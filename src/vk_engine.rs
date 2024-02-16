mod destructors;
pub mod frame_data;
mod immediate;

use std::cell::OnceCell;
use crate::vk_bootstrap;
use crate::vk_types::AllocatedImage;
use crate::{vk_images, vk_init};
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::Handle;
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use frame_data::{FrameData, FRAME_OVERLAP};
use sdl2::event::{Event, WindowEvent};
use sdl2::sys::VkInstance;
use sdl2::EventPump;
use std::marker::PhantomData;
use std::slice;
use gpu_allocator::{AllocationSizes, AllocatorDebugSettings};
use crate::vk_descriptors::DescriptorAllocator;
use imgui_sdl2;

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
    pub draw_extent: vk::Extent2D,
    //descriptor stuff
    pub global_descriptor_allocator: DescriptorAllocator,
    pub draw_image_descriptors: vk::DescriptorSet,
    pub draw_image_descriptor_layout: vk::DescriptorSetLayout,
    //Pipelines
    pub gradient_pipeline: vk::Pipeline,
    pub gradient_pipeline_layout: vk::PipelineLayout,
    //ImGUI stuff - Immediate
    pub immediate_fence: vk::Fence,
    pub immediate_command_pool: vk::CommandPool,
    pub immediate_command_buffer : vk::CommandBuffer,
    //ImGUI specific structs
    pub imgui_context: imgui::Context,
    pub imgui_sdl2: imgui_sdl2::ImguiSdl2,
    pub imgui_pool: vk::DescriptorPool,
    pub renderer: OnceCell<imgui_rs_vulkan_renderer::Renderer>,
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
        let (frames, immediate_command_pool, immediate_command_buffer, immediate_fence) = vk_bootstrap::init_frames(&device, graphics_queue_family);

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
        ) = vk_bootstrap::create_swapchain(
            &instance,
            &device,
            physical_device,
            &surface_loader,
            surface,
            window_extent,
            &mut allocator,
        );
        let (global_descriptor_allocator, draw_image_descriptors, draw_image_descriptor_layout) = vk_bootstrap::init_descriptors(&device, draw_image.image_view);
        let (gradient_pipeline, gradient_pipeline_layout) = vk_bootstrap::init_pipelines(&device, draw_image_descriptor_layout);
        //No need to add to deletion queue, drop method takes care of it
        let (imgui_context, imgui_sdl2, imgui_pool, renderer) = immediate::init_imgui(&instance, &device, physical_device, graphics_queue, immediate_command_pool, swapchain_image_format.format, &window);
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
            draw_extent: window_extent,
            global_descriptor_allocator,
            draw_image_descriptors,
            draw_image_descriptor_layout,
            gradient_pipeline,
            gradient_pipeline_layout,
            immediate_fence,
            immediate_command_pool,
            immediate_command_buffer,
            imgui_context,
            imgui_sdl2,
            imgui_pool,
            renderer: renderer.into()
        })
    }
    pub fn run(&mut self) {
        let mut b_quit = false;
        // main loop
        while !b_quit {
            // Handle events on queue
            for event in self.event_pump.poll_iter() {
                self.imgui_sdl2.handle_event(&mut self.imgui_context, &event);
                if self.imgui_sdl2.ignore_event(&event) { continue; }
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
            self.imgui_sdl2.prepare_frame(self.imgui_context.io_mut(), &self.window, &self.event_pump.mouse_state());
            let ui = self.imgui_context.new_frame();
            //call this immediately before UI rendering code
            self.imgui_sdl2.prepare_render(&ui, &self.window);
            //UI rendering code
            //------------------

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
        vk_images::transition_image(
            &self.device,
            cmd,
            self.draw_image.image,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        self.draw_background(cmd);

        //transition the draw image and the swapchain image into their correct transfer layouts

        //set the draw image to be the source of a copy command
        vk_images::transition_image(
            &self.device,
            cmd,
            self.draw_image.image,
            vk::ImageLayout::GENERAL,
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

        //careful about this code: the only reason we don't need to sync is because transitioning images is currently
        //written as to block all the pipeline
        unsafe {self.device.begin_command_buffer(self.immediate_command_buffer, &cmd_begin_info).unwrap()};

        //draw ImGUI directly into swapchain image
        self.draw_imgui(self.swapchain_image_views[swapchain_image_index as usize]);

        unsafe {self.device.end_command_buffer(self.immediate_command_buffer).unwrap()};

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

            //Renderer must be destroyed early, because it needs access to the device
            self.renderer.take();

            self.destroy_immediate_handles();

            self.destroy_pipelines();

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
            self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.gradient_pipeline);
            self.device.cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::COMPUTE, self.gradient_pipeline_layout, 0, slice::from_ref(&self.draw_image_descriptors), &[]);
            self.device.cmd_dispatch(cmd, (self.draw_extent.width as f32 / 16f32).ceil() as u32, (self.draw_extent.height as f32 / 16f32).ceil() as u32, 1);
        }
    }

    fn draw_imgui(&mut self, target_image_view: vk::ImageView) {
        let color_attachment = vk_init::attachment_info(target_image_view, None, vk::ImageLayout::GENERAL);
        let render_info = vk_init::rendering_info(self.swapchain_extent, color_attachment, None);

        unsafe {self.device.cmd_begin_rendering(self.immediate_command_buffer, &render_info)};

        self.renderer.get_mut().unwrap().cmd_draw(self.immediate_command_buffer, self.imgui_context.render()).unwrap();

        unsafe {self.device.cmd_end_rendering(self.immediate_command_buffer)};

    }
}
