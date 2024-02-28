pub mod buffers;
pub mod gpu_draw_push_constants;
pub mod gpu_mesh_buffers;
pub mod vertex;

use ash::{vk, Device};
use std::cell::OnceCell;

pub struct AllocatedImage {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub allocation: OnceCell<gpu_allocator::vulkan::Allocation>,
    pub image_extent: vk::Extent3D,
    pub image_format: vk::Format,
}

impl AllocatedImage {
    pub unsafe fn dealloc(
        &mut self,
        device: &Device,
        allocator: &mut gpu_allocator::vulkan::Allocator,
    ) {
        unsafe { device.destroy_image_view(self.image_view, None) };
        allocator.free(self.allocation.take().unwrap()).unwrap();
        unsafe { device.destroy_image(self.image, None) };
    }
}
