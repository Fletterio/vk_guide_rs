pub use ash::{vk};

#[derive(Debug)]
pub struct VulkanEngine {
    pub instance : vk::Instance,
    pub chosen_gpu : vk::PhysicalDevice,
    pub device : vk::Device,
    pub surface : vk::SurfaceKHR,
    #[cfg(debug_assertions)]
    pub debug_messenger : vk::DebugUtilsMessengerEXT,
}

// Main loop functions
impl VulkanEngine {
    pub fn init() -> Self {
        todo!()
    }
    pub fn run(&self) {
        todo!()
    }
    pub fn cleanup(&self) {
        todo!()
    }
}



//Internal initialization logic
impl VulkanEngine {
    fn init_vulkan(&mut self) {
        todo!()
    }

    fn init_swapchain(&mut self) {
        todo!()
    }

    fn init_commands(&mut self) {
        todo!()
    }

    fn init_sync_structures(&mut self) {
        todo!()
    }
}