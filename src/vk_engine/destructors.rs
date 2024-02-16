use crate::vk_engine::VulkanEngine;
impl<'a> VulkanEngine<'a> {
    pub fn destroy_swapchain(&mut self) {
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };
        // destroy swapchain resources
        for &view in self.swapchain_image_views.iter() {
            unsafe { self.device.destroy_image_view(view, None) };
        }
        //deallocate the memory for the draw image associated to the swapchain
        unsafe { self.draw_image.dealloc(&self.device, &mut self.allocator) };
    }

    pub fn destroy_pipelines(&mut self){
        unsafe {
            self.device.destroy_pipeline_layout(self.gradient_pipeline_layout, None);
            self.device.destroy_pipeline(self.gradient_pipeline, None);
        }
    }

    pub fn destroy_descriptor_sets(&mut self) {
        unsafe {self.device.destroy_descriptor_set_layout(self.draw_image_descriptor_layout, None)};
        unsafe {self.device.destroy_descriptor_pool(self.global_descriptor_allocator.pool, None)};
    }

    pub fn destroy_frame_data(&mut self) {
        for frame_data in self.frames.iter_mut() {
            unsafe {
                frame_data.dealloc_last_frame();
                self.device
                    .destroy_command_pool(frame_data.command_pool, None);
                self.device.destroy_fence(frame_data.render_fence, None);
                self.device
                    .destroy_semaphore(frame_data.render_semaphore, None);
                self.device
                    .destroy_semaphore(frame_data.swapchain_semaphore, None);
            };
        }
    }

    pub fn destroy_immediate_handles(&mut self) {
        unsafe {self.device.destroy_fence(self.immediate_fence, None);}
        unsafe {self.device.destroy_command_pool(self.immediate_command_pool, None)};
        unsafe {self.device.destroy_descriptor_pool(self.imgui_pool, None)};
    }
}
