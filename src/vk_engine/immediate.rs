use core::slice;
use std::sync::{Arc, Mutex};
use ash::{Device, Instance, vk};
use ash::vk::{DescriptorPoolSize, PhysicalDevice};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use imgui_rs_vulkan_renderer::{DynamicRendering, Options, Renderer};
use crate::{vk_engine, vk_init, VulkanEngine};
impl<'a> VulkanEngine<'a> {
    #[allow(dead_code)]
    pub fn immediate_submit<Callback: Fn(vk::CommandBuffer) + 'static>(&mut self, callback: Callback) {
        unsafe {self.device.reset_fences(slice::from_ref(&self.immediate_fence)).unwrap()};
        unsafe {self.device.reset_command_buffer(self.immediate_command_buffer, vk::CommandBufferResetFlags::empty()).unwrap()};

        let cmd_begin_info = vk_init::command_buffer_begin_info(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {self.device.begin_command_buffer(self.immediate_command_buffer, &cmd_begin_info).unwrap()};

        callback(self.immediate_command_buffer);

        unsafe {self.device.end_command_buffer(self.immediate_command_buffer).unwrap()};

        let cmd_submit_info = vk_init::command_buffer_submit_info(self.immediate_command_buffer);
        let submit_info = vk_init::submit_info(&cmd_submit_info, None, None);

        // submit command buffer to the queue and execute it.
        // immediate_fence will now block until the GUI commands finish execution on the graphics queue
        unsafe {self.device.queue_submit2(self.graphics_queue, slice::from_ref(&submit_info), self.immediate_fence).unwrap()};

        unsafe {self.device.wait_for_fences(slice::from_ref(&self.immediate_fence), true, 9999999999).unwrap()};

    }
}

//WARNING: extremely oversized
pub fn init_imgui(instance: &Instance, device: &Device, physical_device: PhysicalDevice, queue: vk::Queue, command_pool: vk::CommandPool, swapchain_format: vk::Format, window: &sdl2::video::Window) -> (imgui::Context, imgui_sdl2::ImguiSdl2, vk::DescriptorPool, imgui_rs_vulkan_renderer::Renderer){
    let pool_sizes : [vk::DescriptorPoolSize; 11] = [
        DescriptorPoolSize {ty : vk::DescriptorType::SAMPLER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::COMBINED_IMAGE_SAMPLER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::SAMPLED_IMAGE, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::STORAGE_IMAGE, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::UNIFORM_TEXEL_BUFFER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::STORAGE_TEXEL_BUFFER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::UNIFORM_BUFFER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::STORAGE_BUFFER, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::STORAGE_BUFFER_DYNAMIC, descriptor_count: 1000},
        DescriptorPoolSize {ty : vk::DescriptorType::INPUT_ATTACHMENT, descriptor_count: 1000},
    ];

    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
        .max_sets(1000)
        .pool_sizes(&pool_sizes)
        .build();

    let imgui_pool = unsafe {device.create_descriptor_pool(&pool_info, None).unwrap()};

    //Initialize imgui
    //Initialize core structures
    let mut imgui_context = imgui::Context::create();

    //Initialize additional structures for SDL
    let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui_context, window);

    let mut renderer = {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        }).unwrap();

        Renderer::with_gpu_allocator(
            Arc::new(Mutex::new(allocator)),
            device.clone(),
            queue,
            command_pool,
            DynamicRendering {color_attachment_format: swapchain_format, depth_attachment_format: None},
            &mut imgui_context,
            Some(Options {
                in_flight_frames: vk_engine::frame_data::FRAME_OVERLAP,
                ..Default::default()
            }),
        ).unwrap()
    };

    renderer.update_fonts_texture(queue, command_pool, &mut imgui_context).unwrap();

    (imgui_context, imgui_sdl2, imgui_pool, renderer)
}