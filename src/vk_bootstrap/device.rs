use ash::{Instance, vk};
use anyhow::{anyhow, Result};
use ash::vk::BaseOutStructure;

pub fn pick_physical_device(instance : &Instance, surface : vk::SurfaceKHR) -> Result<vk::PhysicalDevice> {
    let physical_devices = unsafe {
        instance.enumerate_physical_devices()?
    };
    log::debug!("{} devices (GPU) found with vulkan support.", physical_devices.len());
    let mut result = None;
    for &physical_device in physical_devices.iter() {
        if is_physical_device_suitable(instance, surface, physical_device) {
            if result.is_none() {
                result = Some(physical_device);
            }
        }
    }
    match result {
        None => Err(anyhow!("Failed to find a suitable GPU!")),
        Some(physical_device) => Ok(physical_device)
    }
}

fn is_physical_device_suitable(instance : &Instance, surface : vk::SurfaceKHR, physical_device : vk::PhysicalDevice) -> bool {
    features_supported(instance, physical_device) && surface_supported(instance, surface, physical_device)
}

fn features_supported(instance : &Instance, physical_device : vk::PhysicalDevice) -> bool {
    let mut features : vk::PhysicalDeviceFeatures2 = vk::PhysicalDeviceFeatures2::default();
    unsafe {instance.get_physical_device_features2(physical_device, &mut features)};
    let mut next = features.p_next as *BaseOutStructure;
    while(! next.is_null()){
        match unsafe {(*next).s_type} {
            vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_FEATURES => {
                let features12 = unsafe {*(next as *vk::PhysicalDeviceVulkan12Features)};
                if (! features12.buffer_device_address || ! features12.descriptor_indexing) {return false;}
                next = features12.p_next as *BaseOutStructure;
            },
            vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_3_FEATURES => {
                let features13 = unsafe {*(next as *vk::PhysicalDeviceVulkan13Features)};
                if (! features13.dynamic_rendering || ! features13.synchronization2) {return false;}
                next = features13.p_next as *BaseOutStructure;
            },
            _ => {
                next = unsafe {(*next).p_next};
            }
        }
    }
    true
}

fn surface_supported(instance : &Instance, surface : vk::SurfaceKHR, physical_device : vk::PhysicalDevice) -> bool {
    true
}