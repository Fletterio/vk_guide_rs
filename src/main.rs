mod vk_bootstrap;
mod vk_engine;
mod vk_types;
mod vk_debug;

use vk_engine::{VulkanEngine};

fn main() {
    pretty_env_logger::init();
    let mut engine = VulkanEngine::init();
    engine.run();
    engine.cleanup();
}
