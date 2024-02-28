#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: cgmath::Vector3<f32>,
    pub uv_x: f32,
    pub normal: cgmath::Vector3<f32>,
    pub uv_y: f32,
    pub color: cgmath::Vector4<f32>,
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: (0f32, 0f32, 0f32).into(),
            uv_x: 0f32,
            normal: (0f32, 0f32, 0f32).into(),
            uv_y: 0f32,
            color: (0f32, 0f32, 0f32, 1f32).into()
        }
    }
}