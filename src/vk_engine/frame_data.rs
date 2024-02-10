use ash::{vk};
use crate::vk_engine::VulkanEngine;

pub const FRAME_OVERLAP : usize = 2;
#[derive(Default, Debug)]
pub struct FrameData {
    pub command_pool : vk::CommandPool,
    pub main_command_buffer : vk::CommandBuffer,
    pub swapchain_semaphore : vk::Semaphore,
    pub render_semaphore : vk::Semaphore,
    pub render_fence : vk::Fence
}

impl VulkanEngine {
    pub fn get_current_frame(&mut self) -> &mut FrameData {
        &mut self.frames[(self.frame_number % FRAME_OVERLAP as i32) as usize]
    }
}