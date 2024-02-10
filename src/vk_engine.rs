mod destructors;

use crate::vk_bootstrap;
use anyhow::Result;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::Handle;
use ash::{vk, Entry};
pub use ash::{Device, Instance};
use sdl2::event::{Event, WindowEvent};
use sdl2::sys::VkInstance;
use sdl2::EventPump;

const WINDOW_TITLE: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 1700;
const WINDOW_HEIGHT: u32 = 900;

pub struct VulkanEngine {
    pub is_initialized: bool,
    pub entry: Entry,
    pub frame_number: i32,
    pub stop_rendering: bool,
    pub window_extent: vk::Extent2D,
    pub window: sdl2::video::Window,
    pub instance: Instance,
    #[cfg(debug_assertions)]
    pub debug_utils_loader: DebugUtils,
    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    pub surface_loader: Surface,
    pub surface: vk::SurfaceKHR,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    //swapchainStuff
    pub swapchain_extent : vk::Extent2D,
    pub swapchain_loader : Swapchain,
    pub swapchain : vk::SwapchainKHR,
    pub swapchain_image_format : vk::SurfaceFormatKHR,
    pub swapchain_images : Vec<vk::Image>,
    pub swapchain_image_views : Vec<vk::ImageView>,
    //window event pump
    pub event_pump: EventPump,
}

// Main loop functions
impl VulkanEngine {
    pub fn init() -> Result<Self> {
        let window_extent = vk::Extent2D {
            width: 1700,
            height: 900,
        };
        //SDL initialization
        //todo: not killed
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
            .position_centered()
            .vulkan()
            .build().unwrap();

        //Vulkan initialization
        let entry = Entry::linked();
        let instance = vk_bootstrap::create_instance(&entry, &window);
        //Debug Utils initialization
        #[cfg(debug_assertions)]
            let debug_utils_loader = DebugUtils::new(&entry, &instance);
        #[cfg(debug_assertions)]
            let debug_messenger = vk_bootstrap::create_debug_messenger(&debug_utils_loader);
        //Surface initialization
        let surface_loader = Surface::new(&entry, &instance);
        let instance_handle = instance.handle().as_raw();
        let surface = vk::SurfaceKHR::from_raw(
            window
                .vulkan_create_surface(instance_handle as VkInstance)
                .unwrap(),
        );
        let (device, physical_device) = vk_bootstrap::create_device(&instance, &surface_loader, surface);
        let event_pump = sdl_context.event_pump().unwrap();
        let (swapchain_loader, swapchain, swapchain_image_format, swapchain_images, swapchain_image_views, swapchain_extent) = vk_bootstrap::create_swapchain(&instance, &device, physical_device, &surface_loader, surface, window_extent);
        Ok(VulkanEngine {
            is_initialized: true,
            entry,
            frame_number: 0,
            stop_rendering: false,
            window_extent,
            window,
            instance,
            #[cfg(debug_assertions)]
            debug_utils_loader,
            #[cfg(debug_assertions)]
            debug_messenger,
            surface_loader,
            surface,
            device,
            physical_device,
            swapchain_extent,
            swapchain_loader,
            swapchain,
            swapchain_image_format,
            swapchain_images,
            swapchain_image_views,
            event_pump,
        })
    }
    pub fn run(&mut self) {
        let mut b_quit = false;
        // main loop
        while !b_quit {
            // Handle events on queue
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => {
                        b_quit = true;
                    }
                    Event::Window { win_event, .. } => {
                        match win_event {
                            WindowEvent::Minimized => self.stop_rendering = true,
                            WindowEvent::Restored => self.stop_rendering = false,
                            _ => {}
                        };
                    }
                    _ => {}
                };
            }
            //do not draw if we are minimized
            if self.stop_rendering {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            self.draw();
        }
    }
    pub fn draw(&mut self) {
        todo!()
    }
}

impl Drop for VulkanEngine {
    fn drop(&mut self) {
        if(self.is_initialized){
            self.destroy_swapchain();

            unsafe {
                self.surface_loader.destroy_surface(self.surface, None);
                self.device.destroy_device(None);

                self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
                self.instance.destroy_instance(None);

            };
        }
    }
}
