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