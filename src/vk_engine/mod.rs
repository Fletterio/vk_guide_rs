pub use ash::{vk};
use sdl2::event::{Event, WindowEvent};


#[derive(Debug)]
pub struct VulkanEngine {
    pub is_initialized : bool,
    pub frame_number : i32,
    pub stop_rendering : bool,
    pub window_extent : vk::Extent2D,
    pub window : sdl2::video::Window,
    pub instance : vk::Instance,
    pub chosen_gpu : vk::PhysicalDevice,
    pub device : vk::Device,
    pub surface : vk::SurfaceKHR,
    #[cfg(debug_assertions)]
    pub debug_messenger : vk::DebugUtilsMessengerEXT,
}

// Main loop functions
impl VulkanEngine {
    pub fn init() -> Self {
        unsafe {
            let extent = vk::Extent2D {width : 1700, height : 900};
            //todo: not killed
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();
            let window = video_subsystem.window("Vulkan Engine", 1700, 900)
                .position_centered()
                .vulkan()
                .build()
                .unwrap();

            VulkanEngine {
                is_initialized : true,
                frame_number : 0,
                stop_rendering : false,
                window_extent : extent,
                window,
                instance : ,
                chosen_gpu : ,
                device : ,
                surface : ,
                #[cfg(debug_assertions)]
                debug_messenger : ,
            }
        }
    }
    pub fn run(&mut self) {
        let mut bQuit = false;
        let sdl_context = sdl2::init().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();
        // main loop
        while (!bQuit){
            // Handle events on queue
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit => {bQuit = true;},
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
    pub fn cleanup(&mut self) {
        if (self.is_initialized){

        }
    }
}

//Internal initialization logic
impl VulkanEngine {
    fn init_vulkan(&mut self) {
        todo!()
    }

    fn init_swapchain(&mut self) {
        todo!()
    }

    fn init_commands(&mut self) {
        todo!()
    }

    fn init_sync_structures(&mut self) {
        todo!()
    }
}