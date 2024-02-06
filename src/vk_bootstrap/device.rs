use ash::{Instance, vk};
use anyhow::{anyhow, Result};
use ash::extensions::khr::Surface;
use ash::vk::BaseOutStructure;

pub fn pick_physical_device_and_queue(instance : &Instance, surface_loader : &Surface, surface : vk::SurfaceKHR) -> Result<(vk::PhysicalDevice, u32)> {
    let physical_devices = unsafe {
        instance.enumerate_physical_devices()?
    };
    log::debug!("{} devices (GPU) found with vulkan support.", physical_devices.len());
    let mut result = None;
    for &physical_device in physical_devices.iter() {
        match is_physical_device_suitable(instance, surface_loader, surface, physical_device)? {
            Some(index) => {if result.is_none() {
                result = Some((physical_device, index));
                break;
            }},
            None => {}
        }
    }
    match result {
        None => Err(anyhow!("Failed to find a suitable GPU!")),
        Some(device_queue_pair) => Ok(device_queue_pair)
    }
}

fn is_physical_device_suitable(instance : &Instance, surface_loader : &Surface, surface : vk::SurfaceKHR, physical_device : vk::PhysicalDevice) -> Result<Option<u32>> {
    let index = surface_supported(instance, surface_loader, surface, physical_device)?;
    if features_supported(instance, physical_device) {
        Ok(Some(index))
    }
    else {
        Ok(None)
    }
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

fn surface_supported(instance : &Instance, surface_loader : &Surface, surface : vk::SurfaceKHR, physical_device : vk::PhysicalDevice) -> Result<u32> {
    unsafe {Ok(instance.get_physical_device_queue_family_properties(physical_device).iter().enumerate()
        .find(|(index, qfp)| -> bool {
            qfp.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            && surface_loader.get_physical_device_surface_support(physical_device, *index as u32, surface)?
        })?.0 as u32)}
}