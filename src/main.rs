extern crate core;

mod vk_bootstrap;
mod vk_debug;
mod vk_descriptors;
mod vk_engine;
mod vk_images;
mod vk_init;
mod vk_types;
mod vk_pipelines;

use vk_engine::VulkanEngine;

fn main() {
    pretty_env_logger::init();
    let mut engine = VulkanEngine::init().unwrap();
    engine.run();
    //no cleanup, it's in the engine's drop
}
