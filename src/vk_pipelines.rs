use std::path::Path;
use ash::{Device, vk};

pub fn load_shader_module(file_path: impl AsRef<Path>, device: &Device) -> vk::ShaderModule {
    let mut file = std::fs::File::open(file_path).unwrap();

    let byte_code_aligned = ash::util::read_spv(&mut file).unwrap();
    let shader_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&byte_code_aligned)
        .build();

    unsafe {device.create_shader_module(&shader_create_info, None).unwrap()}

}