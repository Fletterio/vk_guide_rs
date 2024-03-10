mod destructors;
pub mod frame_data;
mod immediate;

use crate::vk_descriptors::DescriptorAllocator;
use crate::vk_types::AllocatedImage;
use crate::{vk_bootstrap, vk_compute};
use crate::{vk_images, vk_init};
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::Handle;
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use frame_data::{FrameData, FRAME_OVERLAP};
use gpu_allocator::{AllocationSizes, AllocatorDebugSettings};
use imgui_sdl2;
use sdl2::event::{Event, WindowEvent};
use sdl2::sys::VkInstance;
use sdl2::EventPump;
use std::cell::{OnceCell, RefCell};
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;
use std::slice;
use cgmath::Deg;
use crate::vk_bootstrap::init_default_data;
use crate::vk_loader::MeshAsset;
use crate::vk_types::gpu_draw_push_constants::GPUDrawPushConstants;

const WINDOW_TITLE: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 1700;
const WINDOW_HEIGHT: u32 = 900;

pub struct VulkanEngine<'a> {
    pub phantom: PhantomData<&'a u32>,
    pub is_initialized: bool,
    pub entry: Entry,
    pub frame_number: i32,
    pub stop_rendering: bool,
    pub window_extent: vk::Extent2D,
    pub window: sdl2::video::Window,
    pub instance: Instance,
    #[cfg(debug_assertions)]
    pub debug_utils_loader: DebugUtils,
    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    //swapchainStuff
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_image_format: vk::SurfaceFormatKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    //window event pump
    pub event_pump: EventPump,
    //frameStuff
    pub frames: [FrameData; FRAME_OVERLAP],
    //queueStuff
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family: u32,
    //memory allocation
    pub allocator: gpu_allocator::vulkan::Allocator,
    //draw resources
    pub draw_image: AllocatedImage,
    pub depth_image: AllocatedImage,
    pub draw_extent: vk::Extent2D,
    //descriptor stuff
    pub global_descriptor_allocator: DescriptorAllocator,
    pub draw_image_descriptors: vk::DescriptorSet,
    pub draw_image_descriptor_layout: vk::DescriptorSetLayout,
    //ImGUI stuff - Immediate
    pub immediate_fence: vk::Fence,
    pub immediate_command_pool: vk::CommandPool,
    pub immediate_command_buffer: vk::CommandBuffer,
    //ImGUI specific structs
    pub imgui_context: imgui::Context,
    pub imgui_sdl2: imgui_sdl2::ImguiSdl2,
    pub imgui_pool: vk::DescriptorPool,
    pub renderer: OnceCell<imgui_rs_vulkan_renderer::Renderer>,
    //compute pipeline effects
    pub background_effects: Vec<vk_compute::ComputeEffect>,
    pub current_background_effect: usize,
    //mesh pipeline
    pub mesh_pipeline_layout: vk::PipelineLayout,
    pub mesh_pipeline: vk::Pipeline,
    //testing meshes
    pub test_meshes: Vec<Rc<RefCell<MeshAsset>>>,

}

// Main loop functions
impl<'a> VulkanEngine<'a> {
    pub fn init() -> Result<Self> {
        let window_extent = vk::Extent2D {
            width: 1700,
            height: 900,
        };
        //SDL initialization
        //todo: not killed
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .vulkan()
            .build()
            .unwrap();

        //Vulkan initialization
        let entry = Entry::linked();
        let instance = vk_bootstrap::create_instance(&entry, &window);
        //Debug Utils initialization
        #[cfg(debug_assertions)]
        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        #[cfg(debug_assertions)]
        let debug_messenger = vk_bootstrap::create_debug_messenger(&debug_utils_loader);
        //Surface initialization
        let surface_loader = Surface::new(&entry, &instance);
        let instance_handle = instance.handle().as_raw();
        let surface = vk::SurfaceKHR::from_raw(
            window
                .vulkan_create_surface(instance_handle as VkInstance)
                .unwrap(),
        );
        //Device creation
        let (device, physical_device, graphics_queue, graphics_queue_family) =
            vk_bootstrap::create_device(&instance, &surface_loader, surface);
        //Event pump
        let event_pump = sdl_context.event_pump().unwrap();
        //FrameData creation
        let (frames, immediate_command_pool, immediate_command_buffer, immediate_fence) =
            vk_bootstrap::init_frames(&device, graphics_queue_family);

        //Allocator creation
        let allocator_create_info = gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: AllocatorDebugSettings::default(),
            buffer_device_address: true,
            allocation_sizes: AllocationSizes::default(),
        };
        let mut allocator = gpu_allocator::vulkan::Allocator::new(&allocator_create_info).unwrap();
        //Swapchain creation
        let (
            swapchain_loader,
            swapchain,
            swapchain_image_format,
            swapchain_images,
            swapchain_image_views,
            swapchain_extent,
            draw_image,
            depth_image
        ) = vk_bootstrap::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_loader,
            surface,
            window_extent,
            &mut allocator,
        );
        let (global_descriptor_allocator, draw_image_descriptors, draw_image_descriptor_layout) =
            vk_bootstrap::init_descriptors(&device, draw_image.image_view);
        let (background_effects,
            (mesh_pipeline, mesh_pipeline_layout)) =
            vk_bootstrap::init_pipelines(
                &device,
                draw_image_descriptor_layout,
                &draw_image.image_format,
                depth_image.image_format
            );
        //No need to add to deletion queue, drop method takes care of it
        let (imgui_context, imgui_sdl2, imgui_pool, renderer) = immediate::init_imgui(
            &instance,
            &device,
            physical_device,
            graphics_queue,
            immediate_command_pool,
            swapchain_image_format.format,
            &window,
        );
        let test_meshes = init_default_data(&device, &mut allocator, immediate_command_buffer, immediate_fence, graphics_queue);
        Ok(VulkanEngine {
            phantom: PhantomData,
            is_initialized: true,
            entry,
            frame_number: 0,
            stop_rendering: false,
            window_extent,
            window,
            instance,
            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_messenger,
            surface_loader,
            surface,
            device,
            physical_device,
            swapchain_extent,
            swapchain_loader,
            swapchain,
            swapchain_image_format,
            swapchain_images,
            swapchain_image_views,
            event_pump,
            frames,
            graphics_queue,
            graphics_queue_family,
            allocator,
            draw_image,
            depth_image,
            draw_extent: window_extent,
            global_descriptor_allocator,
            draw_image_descriptors,
            draw_image_descriptor_layout,
            immediate_fence,
            immediate_command_pool,
            immediate_command_buffer,
            imgui_context,
            imgui_sdl2,
            imgui_pool,
            renderer: renderer.into(),
            background_effects,
            current_background_effect: 0,
            mesh_pipeline_layout,
            mesh_pipeline,
            test_meshes: test_meshes.unwrap(),
        })
    }
    pub fn run(&mut self) {
        let mut b_quit = false;
        // main loop
        while !b_quit {
            // Handle events on queue
            for event in self.event_pump.poll_iter() {
                self.imgui_sdl2
                    .handle_event(&mut self.imgui_context, &event);
                if self.imgui_sdl2.ignore_event(&event) {
                    continue;
                }
                match event {
                    Event::Quit { .. } => {
                        b_quit = true;
                    }
                    Event::Window { win_event, .. } => {
                        match win_event {
                            WindowEvent::Minimized => self.stop_rendering = true,
                            WindowEvent::Restored => self.stop_rendering = false,
                            _ => {}
                        };
                    }
                    _ => {}
                };
            }
            //do not draw if we are minimized
            if self.stop_rendering {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            //must be called before imgui.frame()
            self.imgui_sdl2.prepare_frame(
                self.imgui_context.io_mut(),
                &self.window,
                &self.event_pump.mouse_state(),
            );
            let ui = self.imgui_context.new_frame();
            //UI rendering code

            //ui.show_demo_window(&mut true);
            let effects = &self.background_effects;
            let selected = &effects[self.current_background_effect];
            let shader_selection_window = ui.window("Shader selector");
            let effects_len = effects.len();
            shader_selection_window.build(|| {
                ui.slider(
                    "Effect Index",
                    0,
                    effects_len - 1,
                    &mut self.current_background_effect,
                );

                let mut data = selected.data.borrow_mut();
                ui.input_float4("data1", &mut data.data1).build();
                ui.input_float4("data2", &mut data.data2).build();
                ui.input_float4("data3", &mut data.data3).build();
                ui.input_float4("data4", &mut data.data4).build();
            });

            //call this immediately before rendering
            self.imgui_sdl2.prepare_render(&ui, &self.window);
            self.imgui_context.render();
            self.draw();
        }
    }
    pub fn draw(&mut self) {
        unsafe {
            self.device
                .wait_for_fences(
                    slice::from_ref(&self.get_current_frame().render_fence),
                    true,
                    1000000000,
                )
                .unwrap()
        }
        //delete all objects crated for last draw
        unsafe {
            self.get_current_frame_mut().dealloc_last_frame();
        }

        unsafe {
            self.device
                .reset_fences(slice::from_ref(&self.get_current_frame().render_fence))
                .unwrap()
        }

        let swapchain_image_index = unsafe {
            self.swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    1000000000,
                    self.get_current_frame().swapchain_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
                .0
        };
        let cmd = self.get_current_frame().main_command_buffer;
        //since waiting on the fence means that commands have finished executing, we can reset the buffer
        unsafe {
            self.device
                .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                .unwrap()
        }

        //The command buffer is submitted only once to the GPU
        let cmd_begin_info =
            vk_init::command_buffer_begin_info(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        //set extent for draw image
        self.draw_extent = vk::Extent2D {
            width: self.draw_image.image_extent.width,
            height: self.draw_image.image_extent.height,
        };

        //Begin the command buffer for instruction submmission
        unsafe {
            self.device
                .begin_command_buffer(cmd, &cmd_begin_info)
                .unwrap()
        }

        // transition our main draw image into general layout so we can write into it
        // we will overwrite it all so we dont care about what was the older layout
        // GENERAL is required here by the background compute shader
        vk_images::transition_image(
            &self.device,
            cmd,
            self.draw_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        self.draw_background(cmd);

        //set the draw image to be drawable by graphics commands
        vk_images::transition_image(
            &self.device,
            cmd,
            self.draw_image.image,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );

        //transition depth image for depth testing
        vk_images::transition_image(
            &self.device,
            cmd,
            self.depth_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
        );

        self.draw_geometry(&self.device, cmd);

        //change draw image to be source of a copy command
        vk_images::transition_image(
            &self.device,
            cmd,
            self.draw_image.image,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );

        //Set the swapchain image to be the destination of the same copy command
        vk_images::transition_image(
            &self.device,
            cmd,
            self.swapchain_images[swapchain_image_index as usize],
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        //submit copy from draw image to the current swapchain image to the command buffer
        vk_images::copy_image_to_image(
            &self.device,
            cmd,
            self.draw_image.image,
            self.swapchain_images[swapchain_image_index as usize],
            self.draw_extent,
            self.swapchain_extent,
        );

        // set swapchain image layout to color attachment so we can show draw on it
        vk_images::transition_image(
            &self.device,
            cmd,
            self.swapchain_images[swapchain_image_index as usize],
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        );
        //draw ImGUI directly into swapchain image
        self.draw_imgui(
            cmd,
            self.swapchain_image_views[swapchain_image_index as usize],
        );

        //transition swapchain image to a presentable layout
        vk_images::transition_image(
            &self.device,
            cmd,
            self.swapchain_images[swapchain_image_index as usize],
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        //finalize the command buffer (we can no longer add commands, but it can now be executed)
        unsafe { self.device.end_command_buffer(cmd).unwrap() };

        //prepare the submission to the queue.
        //we want to wait on the _presentSemaphore, as that semaphore is signaled when the swapchain is ready
        //we will signal the _renderSemaphore, to signal that rendering has finished
        let cmd_info = vk_init::command_buffer_submit_info(cmd);

        let wait_info = vk_init::semaphore_submit_info(
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT_KHR,
            self.get_current_frame().swapchain_semaphore,
        );
        let signal_info = vk_init::semaphore_submit_info(
            vk::PipelineStageFlags2::ALL_GRAPHICS,
            self.get_current_frame().render_semaphore,
        );

        let submit = vk_init::submit_info(&cmd_info, Some(&signal_info), Some(&wait_info));

        //submit command buffer to the queue and execute it.
        // render_fence will now block until the graphic commands finish execution
        unsafe {
            self.device
                .queue_submit2(
                    self.graphics_queue,
                    slice::from_ref(&submit),
                    self.get_current_frame().render_fence,
                )
                .unwrap()
        }

        //prepare present
        // this will put the image we just rendered to into the visible window.
        // we want to wait on the render_semaphore for that,
        // as its necessary that drawing commands have finished before the image is displayed to the user
        let present_info = vk::PresentInfoKHR::builder()
            .swapchains(slice::from_ref(&self.swapchain))
            .wait_semaphores(slice::from_ref(&self.get_current_frame().render_semaphore))
            .image_indices(slice::from_ref(&swapchain_image_index))
            .build();
        unsafe {
            self.swapchain_loader
                .queue_present(self.graphics_queue, &present_info)
                .unwrap()
        };
        self.frame_number += 1;
    }
}

impl<'a> Drop for VulkanEngine<'a> {
    fn drop(&mut self) {
        if self.is_initialized {
            unsafe {
                self.device.device_wait_idle().unwrap();
            }
            //---------------------------- deallocations go here ------------------------------------------

            //GPUMeshBuffers in test_meshes need to be deallocated while device and allocator are up
            for mesh in &self.test_meshes {
                mesh.borrow_mut().mesh_buffers.dealloc(&self.device, &mut self.allocator);
            }
            self.renderer.take();

            self.destroy_immediate_handles();

            self.destroy_effects();
            self.destroy_graphics();

            //destroy descriptor sets and their layouts
            self.destroy_descriptor_sets();

            /*for allocated_image in self.allocated_images.iter_mut() {
                unsafe {allocated_image.dealloc(&self.device, &mut self.allocator)}
            }*/

            self.destroy_frame_data();

            self.destroy_swapchain();

            //final cleanup
            unsafe {
                self.surface_loader.destroy_surface(self.surface, None);
                self.device.destroy_device(None);
                #[cfg(debug_assertions)]
                self.debug_utils_loader
                    .destroy_debug_utils_messenger(self.debug_messenger, None);
                self.instance.destroy_instance(None);
            };
        }
    }
}

//Draw commands
impl<'a> VulkanEngine<'a> {
    fn draw_background(&self, cmd: vk::CommandBuffer) {
        unsafe {
            let effects = &self.background_effects;
            let effect = &effects[self.current_background_effect];

            self.device
                .cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, effect.pipeline);
            self.device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::COMPUTE,
                effect.layout,
                0,
                slice::from_ref(&self.draw_image_descriptors),
                &[],
            );

            let push_bytes = slice::from_raw_parts(
                effect.data.as_ptr() as *const vk_compute::ComputePushConstants as *const u8,
                size_of::<vk_compute::ComputePushConstants>(),
            );
            self.device.cmd_push_constants(
                cmd,
                effect.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                push_bytes,
            );
            self.device.cmd_dispatch(
                cmd,
                (self.draw_extent.width as f32 / 16f32).ceil() as u32,
                (self.draw_extent.height as f32 / 16f32).ceil() as u32,
                1,
            );
        }
    }

    fn draw_imgui(&mut self, cmd: vk::CommandBuffer, target_image_view: vk::ImageView) {
        let color_attachment =
            vk_init::attachment_info(target_image_view, None, vk::ImageLayout::GENERAL);
        let render_info = vk_init::rendering_info(self.swapchain_extent, color_attachment, None);

        unsafe { self.device.cmd_begin_rendering(cmd, &render_info) };

        self.renderer
            .get_mut()
            .unwrap()
            .cmd_draw(cmd, self.imgui_context.render())
            .unwrap();

        unsafe { self.device.cmd_end_rendering(cmd) };
    }

    fn draw_geometry(&self, device: &Device, cmd: vk::CommandBuffer) {
        // create necessary drawing info
        let color_attachment =
            vk_init::attachment_info(self.draw_image.image_view, None, vk::ImageLayout::GENERAL);
        let depth_attachment = vk_init::depth_attachment_info(self.depth_image.image_view, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
        let render_info = vk_init::rendering_info(self.draw_extent, color_attachment, Some(&depth_attachment));
        //begin "renderpass"
        unsafe { device.cmd_begin_rendering(cmd, &render_info) };

        //set dynamic viewport and scissor
        let viewport = vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(self.draw_extent.width as f32)
            .height(self.draw_extent.height as f32)
            .min_depth(0f32)
            .max_depth(1f32)
            .build();

        unsafe { device.cmd_set_viewport(cmd, 0, slice::from_ref(&viewport)) };

        let scissor = vk::Rect2D::builder()
            .offset(Default::default())
            .extent(self.draw_extent)
            .build();

        unsafe { device.cmd_set_scissor(cmd, 0, slice::from_ref(&scissor)) };

        //Set up pipeline to render meshes
        unsafe {device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.mesh_pipeline)};

        //world matrix needs to be upside down for gltf meshes since Vulkan uses opposite Y to OpenGL
        //set the monkey 5 units back (using left-handed coordinates it seems)
        let view = cgmath::Matrix4::from_translation((0f32, 0f32, -5f32).into());
        let mut projection = cgmath::perspective(Deg {0: 70f32}, (self.draw_extent.width as f32) / (self.draw_extent.height as f32), 10000f32, 0.1f32);

        // https://matthewwellings.com/blog/the-new-vulkan-coordinate-system/
        // https://github.com/LunarG/VulkanSamples/commit/0dd36179880238014512c0637b0ba9f41febe803
        // invert y-axis to use a left-handed system like OpenGL and change NDC to [0,1] which is what Vulkan uses
        let clip = cgmath::Matrix4::new(1f32, 0f32, 0f32,0f32 ,
                                                    0f32, -1f32, 0f32 ,0f32,
                                                    0f32, 0f32 ,0.5f32, 0f32,
                                                    0f32, 0f32, 0.5f32, 1f32);

        projection = clip * projection;

        //draw a blender monkeyhead
        let monkey_mesh = self.test_meshes[2].borrow();

        let push_constants = GPUDrawPushConstants {
            world_matrix: projection * view,
            vertex_buffer: monkey_mesh.mesh_buffers.vertex_buffer_address
        };

        let push_bytes = unsafe {slice::from_raw_parts(
            &push_constants as *const GPUDrawPushConstants as *const u8,
            size_of::<GPUDrawPushConstants>(),
        )};

        unsafe {device.cmd_push_constants(cmd, self.mesh_pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, push_bytes)};
        unsafe {device.cmd_bind_index_buffer(cmd, monkey_mesh.mesh_buffers.index_buffer.buffer, 0, vk::IndexType::UINT32)};

        unsafe {device.cmd_draw_indexed(cmd, monkey_mesh.surfaces[0].count, 1, monkey_mesh.surfaces[0].start_index, 0, 0)};

        unsafe { device.cmd_end_rendering(cmd) };
    }
}
