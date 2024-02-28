use ash::vk;

// push constants for our mesh object draws
pub struct GPUDrawPushConstants {
    pub world_matrix: cgmath::Matrix4<f32>,
    pub vertex_buffer: vk::DeviceAddress,
}
