mod vk_bootstrap;
mod vk_initializers;
mod vk_engine;
mod vk_types;

use vk_engine::{VulkanEngine};

fn main() {
    let mut engine = VulkanEngine::init();
    engine.run();
    engine.cleanup();
}
