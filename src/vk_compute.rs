use std::cell::RefCell;
use ash::vk;

pub struct ComputePushConstants {
    pub data1: cgmath::Vector4<f32>,
    pub data2: cgmath::Vector4<f32>,
    pub data3: cgmath::Vector4<f32>,
    pub data4: cgmath::Vector4<f32>,
}

impl Default for ComputePushConstants {
    fn default() -> Self {
        ComputePushConstants {
            data1: cgmath::Vector4::<f32>::new(0f32, 0f32, 0f32, 0f32),
            data2: cgmath::Vector4::<f32>::new(0f32, 0f32, 0f32, 0f32),
            data3: cgmath::Vector4::<f32>::new(0f32, 0f32, 0f32, 0f32),
            data4: cgmath::Vector4::<f32>::new(0f32, 0f32, 0f32, 0f32),
        }
    }
}

pub struct ComputeEffect {
    pub name: String,
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub data: RefCell<ComputePushConstants>
}
