mod device;

use std::ffi::{c_char, CString};
use ash::{Device, Entry, Instance, vk};
use sdl2::video::Window;
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::DebugUtilsMessengerEXT;
#[cfg(debug_assertions)]
use crate::vk_debug::vulkan_debug_callback;
//-----------------------------INSTANCE-------------------------------
pub fn create_instance(entry : &Entry, window : &Window) -> Instance {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(CString::new("Vulkan Application").unwrap().as_c_str())
        .application_version(vk::make_api_version(0,0,1,0))
        .engine_name(CString::new("No Engine").unwrap().as_c_str())
        .engine_version(vk::make_api_version(0,0,1,0))
        .api_version(vk::make_api_version(0,1,3,0))
        .build();

    let mut extension_names : Vec<*const c_char> = window.vulkan_instance_extensions().unwrap().iter()
        .map(|name| -> *const c_char {
            name.as_ptr() as *const c_char
        })
        .collect();
    #[cfg(debug_assertions)]
    extension_names.push(DebugUtils::name().as_ptr());
    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);
    cfg_if::cfg_if!{
        if #[cfg(debug_assertions)]{
            let _layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const c_char];
            instance_create_info = instance_create_info.enabled_layer_names(&_layer_names);
        }
    }
    unsafe {entry.create_instance(&(instance_create_info.build()), None).unwrap()}
}
//---------------------------------------DEBUG-----------------------------------------
#[cfg(debug_assertions)]
pub fn create_debug_messenger(debug_utils_loader : &DebugUtils) -> Result<DebugUtilsMessengerEXT>{
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(vulkan_debug_callback))
        .build();

    unsafe {Ok(debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?)}
}

//----------------------------DEVICE------------------------------------
pub fn create_device(instance : &Instance, surface_loader : &Surface, surface : vk::SurfaceKHR) -> Result<(Device, vk::PhysicalDevice)> {
    let (physical_device, queue_index) = device::pick_physical_device_and_queue(instance, surface_loader, surface)?;
    let priorities = [1.0];
    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_index)
        .queue_priorities(&priorities)
        .build()];

    let mut features13 = vk::PhysicalDeviceVulkan13Features::builder()
        .dynamic_rendering(true)
        .synchronization2(true)
        .build();
    let mut features12 = vk::PhysicalDeviceVulkan12Features::builder()
        .buffer_device_address(true)
        .descriptor_indexing(true)
        .build();
    let device_extension_names = [Swapchain::name().as_ptr()];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_info)
        .enabled_extension_names(&device_extension_names)
        .push_next(&mut features13)
        .push_next(&mut features12)
        .build();
    unsafe {Ok((instance.create_device(physical_device, &device_create_info, None)?, physical_device))}
}