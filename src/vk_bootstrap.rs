mod device;

#[cfg(debug_assertions)]
use crate::vk_debug::vulkan_debug_callback;
use anyhow::Result;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{DebugUtilsMessengerEXT, SurfaceFormatKHR, SwapchainKHR};
use ash::{vk, Device, Entry, Instance};
use sdl2::video::Window;
use std::ffi::{c_char, CString};

//-----------------------------INSTANCE-------------------------------
pub fn create_instance(entry: &Entry, window: &Window) -> Instance {
    let app_info = vk::ApplicationInfo::builder()
        .application_name(CString::new("Vulkan Application").unwrap().as_c_str())
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(CString::new("No Engine").unwrap().as_c_str())
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 3, 0))
        .build();

    let mut extension_names: Vec<*const c_char> = window
        .vulkan_instance_extensions()
        .unwrap()
        .iter()
        .map(|name| -> *const c_char { name.as_ptr() as *const c_char })
        .collect();
    #[cfg(debug_assertions)]
    extension_names.push(DebugUtils::name().as_ptr());
    let mut instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names);
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)]{
            let _layer_names = [b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const c_char];
            instance_create_info = instance_create_info.enabled_layer_names(&_layer_names);
        }
    }
    unsafe {
        entry
            .create_instance(&(instance_create_info.build()), None)
            .unwrap()
    }
}
//---------------------------------------DEBUG-----------------------------------------
#[cfg(debug_assertions)]
pub fn create_debug_messenger(debug_utils_loader: &DebugUtils) -> DebugUtilsMessengerEXT {
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

    unsafe { debug_utils_loader.create_debug_utils_messenger(&debug_info, None).unwrap() }
}

//----------------------------DEVICE------------------------------------
pub fn create_device(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
) -> (Device, vk::PhysicalDevice){
    let (physical_device, queue_index) = device::pick_physical_device_and_queue(instance, surface_loader, surface);
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
    unsafe {
        (
            instance.create_device(physical_device, &device_create_info, None).unwrap(),
            physical_device,
        )
    }
}

//-------------------SWAPCHAIN-----------------------
pub fn create_swapchain(instance : &Instance, device : &Device, physical_device : vk::PhysicalDevice, surface_loader : &Surface, surface: vk::SurfaceKHR, extent : vk::Extent2D) -> (Swapchain, vk::SwapchainKHR, vk::SurfaceFormatKHR, Vec<vk::Image>, Vec<vk::ImageView>, vk::Extent2D){
    let surface_format = SurfaceFormatKHR{format : vk::Format::B8G8R8A8_UNORM, color_space : vk::ColorSpaceKHR::SRGB_NONLINEAR};
    let surface_capabilities = unsafe {surface_loader.get_physical_device_surface_capabilities(physical_device, surface).unwrap()};
    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0
        && desired_image_count > surface_capabilities.max_image_count
    {
        desired_image_count = surface_capabilities.max_image_count;
    }
    let surface_extent = match surface_capabilities.current_extent.width {
        u32::MAX => extent,
        _ => surface_capabilities.current_extent,
    };
    let pre_transform = if surface_capabilities
        .supported_transforms
        .contains(vk::SurfaceTransformFlagsKHR::IDENTITY)
    {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        surface_capabilities.current_transform
    };

    let swapchain_loader = Swapchain::new(instance, device);

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .image_format(surface_format.format)
        .image_color_space(surface_format.color_space)
        .present_mode(vk::PresentModeKHR::FIFO)
        .image_extent(extent)
        .image_usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .min_image_count(desired_image_count)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .clipped(true)
        .image_array_layers(1)
        .build();
    let swapchain = unsafe {swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap()};
    let swapchain_images = unsafe {swapchain_loader.get_swapchain_images(swapchain).unwrap()};
    let swapchain_image_views : Vec<vk::ImageView> = swapchain_images
        .iter()
        .map(|&image| {
            let create_view_info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image);
            unsafe {device.create_image_view(&create_view_info, None).unwrap()}
        })
        .collect();

    (swapchain_loader, swapchain, surface_format, swapchain_images, swapchain_image_views, surface_extent)
}