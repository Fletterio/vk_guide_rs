mod internal_init;

use ash::{vk, Entry};
pub use ash::{Device, Instance};
use sdl2::event::{Event, WindowEvent};
use anyhow::Result;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;
use ash::vk::Handle;
use sdl2::{EventPump};
use sdl2::sys::VkInstance;
use crate::vk_bootstrap;

const WINDOW_TITLE: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 1700;
const WINDOW_HEIGHT: u32 = 900;

#[derive(Debug)]
pub struct VulkanEngine {
    pub is_initialized : bool,
    pub entry : Entry,
    pub frame_number : i32,
    pub stop_rendering : bool,
    pub window_extent : vk::Extent2D,
    pub window : sdl2::video::Window,
    pub instance : Instance,
    #[cfg(debug_assertions)]
    pub debug_utils_loader : DebugUtils,
    #[cfg(debug_assertions)]
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
    pub surface_loader : Surface,
    pub surface : vk::SurfaceKHR,
    pub device : Device,
    pub chosen_gpu : vk::PhysicalDevice,
    pub event_pump : EventPump,
}

// Main loop functions
impl VulkanEngine {
    pub fn init() -> Result<Self> {
        unsafe {
            let window_extent = vk::Extent2D {width : 1700, height : 900};
            //SDL initialization
            //todo: not killed
            let sdl_context = sdl2::init()?;
            let video_subsystem = sdl_context.video()?;
            let window = video_subsystem.window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
                .position_centered()
                .vulkan()
                .build()?;

            //Vulkan initialization
            let entry = Entry::linked();
            let instance = vk_bootstrap::create_instance(&entry, &window)?;
            //Debug Utils initialization
            #[cfg(debug_assertions)]
            let debug_utils_loader = DebugUtils::new(&entry, &instance);
            #[cfg(debug_assertions)]
            let debug_messenger = vk_bootstrap::create_debug_messenger(&debug_utils_loader)?;
            //Surface initialization
            let surface_loader = Surface::new(&entry, &instance);
            let instance_handle = instance.handle().as_raw();
            let surface = vk::SurfaceKHR::from_raw(window.vulkan_create_surface(instance_handle as VkInstance)?);
            Ok(VulkanEngine {
                is_initialized : true,
                entry,
                frame_number : 0,
                stop_rendering : false,
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
                chosen_gpu,
                event_pump :
            })
        }
    }
    pub fn run(&mut self) {
        let mut b_quit = false;
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        // main loop
        while (!b_quit){
            // Handle events on queue
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit => {b_quit = true;},
                    Event::Window(_,_, win_event) => {
                        match win_event {
                            WindowEvent::Minimized => {self.stop_rendering = true},
                            WindowEvent::Restored => {self.stop_rendering = false},
                            _ => {}
                        };
                    },
                    _ => {}
                };
            };
            //do not draw if we are minimized
            if (self.stop_rendering){
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

impl Drop for VulkanEngine{
    fn drop(&mut self) {
        todo!()
    }
}