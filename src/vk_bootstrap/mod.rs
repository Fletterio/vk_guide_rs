mod device;

use std::ffi::CString;
use std::os::raw::c_char;
use ash::{Device, Entry, Instance, vk};
use sdl2::video::Window;
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::vk::DebugUtilsMessengerEXT;
use crate::vk_debug::vulkan_debug_callback;

//-----------------------------INSTANCE-------------------------------
pub fn create_instance(entry : &Entry, window : &Window) -> Result<Instance> {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(CString::new("Vulkan Application")?.as_c_str())
        .application_version(vk::make_api_version(0,0,1,0))
        .engine_name(CString::new("No Engine")?.as_c_str())
        .engine_version(vk::make_api_version(0,0,1,0))
        .api_version(vk::make_api_version(0,1,3,0))
        .build();

    let mut extension_names = ash_window::enumerate_required_extensions(window.raw_display_handle())?
        .to_vec();
    #[cfg(debug_assertions)]
    extension_names.push(DebugUtils::NAME_as_ptr());

    #[cfg(debug_assertions)]
    let layer_properties = entry.enumerate_instance_layer_properties()?;
    let mut layer_names : Vec<*const c_char> = Vec::new();
    #[cfg(debug_assertions)]
    layer_names = layer_properties.iter().map(|lp| {lp.layer_name.as_ptr()}).collect();

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&layer_names);

    unsafe {Ok(entry.create_instance(&instance_create_info, None)?)}
}
//---------------------------------------DEBUG-----------------------------------------
#[cfg(debug_assertions)]
pub fn create_debug_messenger(debug_utils_loader : &DebugUtils) -> Result<DebugUtilsMessengerEXT>{
    let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
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
        .pfn_user_callback(Some(vulkan_debug_callback));

    unsafe {Ok(debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?)}
}

//----------------------------DEVICE------------------------------------
pub fn create_device(instance : &Instance, surface : vk::SurfaceKHR) -> Result<(Device, vk::PhysicalDevice)> {

}