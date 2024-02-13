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
        unsafe  {self.draw_image.dealloc(&self.device, &mut self.allocator)};
    }
}