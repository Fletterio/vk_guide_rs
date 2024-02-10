use ash::extensions::khr::Surface;
use ash::vk::{BaseOutStructure, QueueFamilyProperties};
use ash::{vk, Instance};
use cgmath::Zero;

pub fn pick_physical_device_and_queue(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
) -> (vk::PhysicalDevice, u32) {
    let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
    log::debug!(
        "{} devices (GPU) found with vulkan support.",
        physical_devices.len()
    );
    let mut result = None;
    for &physical_device in physical_devices.iter() {
        match is_physical_device_suitable(instance, surface_loader, surface, physical_device) {
            Some(queue_family_index) => {
                if result.is_none() {
                    result = Some((physical_device, queue_family_index));
                    break;
                }
            }
            None => {}
        }
    }
    match result {
        Some(device_queue_pair) => device_queue_pair,
        None => {
            log::error!("Failed to find a suitable GPU!");
            panic!();
            },
    }
}

fn is_physical_device_suitable(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> Option<u32> {
    let queue_family_index = surface_supported(instance, surface_loader, surface, physical_device);
    if features_supported(instance, physical_device) {
        Some(queue_family_index)
    } else {
        None
    }
}

fn features_supported(instance: &Instance, physical_device: vk::PhysicalDevice) -> bool {
    let mut features: vk::PhysicalDeviceFeatures2 = vk::PhysicalDeviceFeatures2::default();
    unsafe { instance.get_physical_device_features2(physical_device, &mut features) };
    let mut next = features.p_next as *const BaseOutStructure;
    while !next.is_null() {
        match unsafe { (*next).s_type } {
            vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_2_FEATURES => {
                let features12 = unsafe { *(next as *const vk::PhysicalDeviceVulkan12Features) };
                if features12.buffer_device_address.is_zero()
                    || features12.descriptor_indexing.is_zero()
                {
                    return false;
                }
                next = features12.p_next as *const BaseOutStructure;
            }
            vk::StructureType::PHYSICAL_DEVICE_VULKAN_1_3_FEATURES => {
                let features13 = unsafe { *(next as *const vk::PhysicalDeviceVulkan13Features) };
                if features13.dynamic_rendering.is_zero() || features13.synchronization2.is_zero() {
                    return false;
                }
                next = features13.p_next as *const BaseOutStructure;
            }
            _ => {
                next = unsafe { (*next).p_next };
            }
        }
    }
    true
}

fn surface_supported(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
    physical_device: vk::PhysicalDevice,
) -> u32 {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(physical_device)
            .iter()
            .enumerate()
            .find(|(queue_family_index, qfp): &(usize, &QueueFamilyProperties)| -> bool {
                qfp.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                    && surface_loader
                        .get_physical_device_surface_support(
                            physical_device,
                            *queue_family_index as u32,
                            surface,
                        )
                        .unwrap()
            })
            .unwrap()
            .0 as u32
    }
}
