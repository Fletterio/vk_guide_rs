mod device;

use std::cell::{OnceCell};
#[cfg(debug_assertions)]
use crate::vk_debug::vulkan_debug_callback;
use crate::vk_engine::frame_data::{FrameData, FRAME_OVERLAP};
use crate::vk_init;
#[cfg(debug_assertions)]
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use sdl2::video::Window;
use std::ffi::{c_char, CString};
use gpu_allocator::MemoryLocation;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme};
use crate::vk_types::AllocatedImage;

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
pub fn create_debug_messenger(debug_utils_loader: &DebugUtils) -> vk::DebugUtilsMessengerEXT {
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

    unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&debug_info, None)
            .unwrap()
    }
}

//----------------------------DEVICE------------------------------------
pub fn create_device(
    instance: &Instance,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
) -> (Device, vk::PhysicalDevice, vk::Queue, u32) {
    let (physical_device, queue_family_index) =
        device::pick_physical_device_and_queue(instance, surface_loader, surface);
    let priorities = [1.0];
    let queue_info = [vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
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
    let device: Device = unsafe {
        instance
            .create_device(physical_device, &device_create_info, None)
            .unwrap()
    };
    let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    (device, physical_device, graphics_queue, queue_family_index)
}

//-------------------SWAPCHAIN-----------------------
pub fn create_swapchain(
    instance: &Instance,
    device: &Device,
    physical_device: vk::PhysicalDevice,
    surface_loader: &Surface,
    surface: vk::SurfaceKHR,
    extent: vk::Extent2D,
    allocator : &mut gpu_allocator::vulkan::Allocator,
) -> (
    Swapchain,
    vk::SwapchainKHR,
    vk::SurfaceFormatKHR,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
    vk::Extent2D,
    AllocatedImage
) {
    let surface_format = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_UNORM,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    let surface_capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(physical_device, surface)
            .unwrap()
    };
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
    let swapchain = unsafe {
        swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .unwrap()
    };
    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
    let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
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
            unsafe { device.create_image_view(&create_view_info, None).unwrap() }
        })
        .collect();

    //draw Image stuff
    let draw_image_extent = vk::Extent3D {width : extent.width, height : extent.height, depth : 1};
    //hardcoding the draw format to 32 bit float
    let draw_image_format = vk::Format::R16G16B16A16_SFLOAT;
    let draw_image_usage_flags : vk::ImageUsageFlags = vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::STORAGE
        | vk::ImageUsageFlags::COLOR_ATTACHMENT;
    let draw_image_create_info = vk_init::image_create_info(draw_image_format, draw_image_usage_flags, draw_image_extent);
    let draw_image = unsafe {device.create_image(&draw_image_create_info, None).unwrap()};
    let mut draw_image_requirements = unsafe {device.get_image_memory_requirements(draw_image)};
    //ensure memory is hosted on GPU VRAM. This is likely unnecessary
    draw_image_requirements.memory_type_bits |= vk::MemoryPropertyFlags::DEVICE_LOCAL.as_raw();
    let draw_image_allocation : OnceCell<Allocation> = OnceCell::new();
    draw_image_allocation.set(allocator.allocate(&AllocationCreateDesc {
        name : "draw_image_allocation",
        requirements : draw_image_requirements,
        location : MemoryLocation::GpuOnly,
        linear : false,
        allocation_scheme : AllocationScheme::DedicatedImage(draw_image)
    }).unwrap()).unwrap();
    // Bind memory to the image
    unsafe { device.bind_image_memory(draw_image, draw_image_allocation.get().unwrap().memory(), draw_image_allocation.get().unwrap().offset()).unwrap()};

    //build a image-view for the draw image to use for rendering
    let draw_image_view_create_info = vk_init::image_view_create_info(draw_image_format, draw_image, vk::ImageAspectFlags::COLOR);
    let draw_image_view = unsafe {device.create_image_view(&draw_image_view_create_info, None).unwrap()};

    let allocated_image = AllocatedImage {
        image : draw_image,
        image_view : draw_image_view,
        allocation : draw_image_allocation,
        image_extent : draw_image_extent,
        image_format : draw_image_format
};

    (
        swapchain_loader,
        swapchain,
        surface_format,
        swapchain_images,
        swapchain_image_views,
        surface_extent,
        allocated_image,
    )
}

pub fn init_frames(device: &Device, graphics_queue_family: u32) -> [FrameData; FRAME_OVERLAP] {
    let command_stuff = init_commands(device, graphics_queue_family);
    let sync_structures = init_sync_structures(device);
    let frames = <[FrameData; FRAME_OVERLAP]>::try_from(
        (0..FRAME_OVERLAP)
            .map(|frame| -> FrameData {
                FrameData {
                    command_pool: command_stuff[frame].0,
                    main_command_buffer: command_stuff[frame].1,
                    swapchain_semaphore: sync_structures[frame].0,
                    render_semaphore: sync_structures[frame].1,
                    render_fence: sync_structures[frame].2,
                }
            })
            .collect::<Vec<FrameData>>(),
    );
    frames.expect("Error - weird error going from vec to array")
}

fn init_commands(
    device: &Device,
    graphics_queue_family: u32,
) -> [(vk::CommandPool, vk::CommandBuffer); FRAME_OVERLAP] {
    let command_pool_info = vk_init::command_pool_create_info(
        graphics_queue_family,
        vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
    );
    let commands: [(vk::CommandPool, vk::CommandBuffer); FRAME_OVERLAP] = (0..FRAME_OVERLAP)
        .map(|_| -> (vk::CommandPool, vk::CommandBuffer) {
            let command_pool = unsafe {
                device
                    .create_command_pool(&command_pool_info, None)
                    .unwrap()
            };
            let cmd_alloc_info = vk_init::command_buffer_allocate_info(command_pool, 1);
            let main_command_buffer =
                unsafe { device.allocate_command_buffers(&cmd_alloc_info).unwrap()[0] };
            (command_pool, main_command_buffer)
        })
        .collect::<Vec<(vk::CommandPool, vk::CommandBuffer)>>()
        .try_into()
        .unwrap();
    commands
}

fn init_sync_structures(
    device: &Device,
) -> [(vk::Semaphore, vk::Semaphore, vk::Fence); FRAME_OVERLAP] {
    let fence_create_info = vk_init::fence_create_info(vk::FenceCreateFlags::SIGNALED);
    let semaphore_create_info = vk_init::semaphore_create_info(vk::SemaphoreCreateFlags::empty());

    let structures: [(vk::Semaphore, vk::Semaphore, vk::Fence); FRAME_OVERLAP] = (0..FRAME_OVERLAP)
        .map(|_| -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
            let render_fence = unsafe { device.create_fence(&fence_create_info, None).unwrap() };
            let (swapchain_semaphore, render_semaphore) = unsafe {
                (
                    device
                        .create_semaphore(&semaphore_create_info, None)
                        .unwrap(),
                    device
                        .create_semaphore(&semaphore_create_info, None)
                        .unwrap(),
                )
            };
            (swapchain_semaphore, render_semaphore, render_fence)
        })
        .collect::<Vec<(vk::Semaphore, vk::Semaphore, vk::Fence)>>()
        .try_into()
        .unwrap();
    structures
}
