use crate::vk_engine::{VulkanEngine};
use ash::vk;
use std::fmt::Formatter;

pub const FRAME_OVERLAP: usize = 2;
#[derive(Default)]
pub struct FrameData {
    pub command_pool: vk::CommandPool,
    pub main_command_buffer: vk::CommandBuffer,
    pub swapchain_semaphore: vk::Semaphore,
    pub render_semaphore: vk::Semaphore,
    pub render_fence: vk::Fence,
}

impl<'a> VulkanEngine<'a> {
    pub fn get_current_frame(&self) -> &FrameData {
        &self.frames[(self.frame_number % FRAME_OVERLAP as i32) as usize]
    }
    pub fn get_current_frame_mut(&mut self) -> &mut FrameData {
        &mut self.frames[(self.frame_number % FRAME_OVERLAP as i32) as usize]
    }
}

impl FrameData {
    pub unsafe fn dealloc_last_frame(&mut self) {

    }
}

//Necessary to be able to collect vector in array, somehow
impl std::fmt::Debug for FrameData {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
