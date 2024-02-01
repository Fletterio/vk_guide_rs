use std::ffi::CString;
use ash::{Entry, Instance, vk};
use sdl2::video::Window;
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;

fn create_instance(entry : &Entry, window : &Window) -> Result<Instance> {
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


    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names();

}