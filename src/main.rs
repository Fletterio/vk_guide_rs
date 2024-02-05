mod vk_bootstrap;
mod vk_engine;
mod vk_types;
mod vk_debug;

use vk_engine::{VulkanEngine};

fn main() {
    pretty_env_logger::init();
    let mut engine = VulkanEngine::init();
    engine.run();
    //no cleanup, it's in the engine's drop
}
