use core::slice;
use std::ffi::CStr;
use std::path::Path;
use std::ptr::null;
use ash::{Device, vk};

pub fn load_shader_module(file_path: impl AsRef<Path>, device: &Device) -> vk::ShaderModule {
    let mut file = std::fs::File::open(file_path).unwrap();

    let byte_code_aligned = ash::util::read_spv(&mut file).unwrap();
    let shader_create_info = vk::ShaderModuleCreateInfo::builder()
        .code(&byte_code_aligned)
        .build();

    unsafe {device.create_shader_module(&shader_create_info, None).unwrap()}

}

#[derive(Default)]
pub struct PipelineBuilder {
    pub shader_stages: Vec<vk::PipelineShaderStageCreateInfo>,
    pub input_assembly: vk::PipelineInputAssemblyStateCreateInfo,
    pub rasterizer: vk::PipelineRasterizationStateCreateInfo,
    pub color_blend_attachment: vk::PipelineColorBlendAttachmentState,
    pub multisampling: vk::PipelineMultisampleStateCreateInfo,
    pub pipeline_layout: vk::PipelineLayout,
    pub depth_stencil: vk::PipelineDepthStencilStateCreateInfo,
    pub render_info: vk::PipelineRenderingCreateInfo,
    pub color_attachment_format: vk::Format
}


impl PipelineBuilder {
    pub fn build_pipeline(mut self, device: &Device) -> vk::Pipeline {
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1)
            .build();

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(slice::from_ref(&self.color_blend_attachment));

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let pipeline_info_builder = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut self.render_info)
            .stages(&self.shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&self.input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&self.rasterizer)
            .multisample_state(&self.multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&self.depth_stencil)
            .layout(self.pipeline_layout);

        //dynamic state setup
        let state = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&state)
            .build();

        let pipeline_info = pipeline_info_builder
            .dynamic_state(&dynamic_info)
            .build();

        unsafe {device.create_graphics_pipelines(vk::PipelineCache::null(), slice::from_ref(&pipeline_info), None).unwrap()[0]}
    }

    pub fn set_shaders(&mut self, vertex_shader: vk::ShaderModule, fragment_shader: vk::ShaderModule, vertex_entry: &CStr, fragment_entry: &CStr) {
        self.shader_stages.clear();
        self.shader_stages.push(vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader)
            .name(vertex_entry)
            .build());
        self.shader_stages.push(vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragment_shader)
            .name(fragment_entry)
            .build());
    }

    pub fn set_input_topology(&mut self, topology: vk::PrimitiveTopology) {
        self.input_assembly.topology = topology;
        self.input_assembly.primitive_restart_enable = vk::FALSE;
    }

    pub fn set_polygon_mode(&mut self, mode: vk::PolygonMode) {
        self.rasterizer.polygon_mode = mode;
        self.rasterizer.line_width = 1f32;
    }

    pub fn set_cull_mode(&mut self, cull_mode: vk::CullModeFlags, front_face: vk::FrontFace) {
        self.rasterizer.cull_mode = cull_mode;
        self.rasterizer.front_face = front_face;
    }

    pub fn set_multisampling_none(&mut self) {
        self.multisampling.sample_shading_enable = vk::FALSE;
        //defaults to no multisampling
        self.multisampling.rasterization_samples = vk::SampleCountFlags::TYPE_1;
        self.multisampling.min_sample_shading = 1f32;
        self.multisampling.p_sample_mask = null();
        //no alpha to coverage either
        self.multisampling.alpha_to_coverage_enable = vk::FALSE;
        self.multisampling.alpha_to_one_enable = vk::FALSE;
    }

    pub fn disable_blending(&mut self) {
        //default write mask
        self.color_blend_attachment.color_write_mask = vk::ColorComponentFlags::RGBA;
        //no blending
        self.color_blend_attachment.blend_enable = vk::FALSE;
    }

    pub fn set_color_attachment_format<'a>(&'a mut self, format: &'a vk::Format) {
        self.color_attachment_format = *format;
        //connect format to renderInfo
        self.render_info.color_attachment_count = 1;
        self.render_info.p_color_attachment_formats = format as *const vk::Format;
    }

    pub fn set_depth_format(&mut self, format: vk::Format) {
        self.render_info.depth_attachment_format = format;
    }

    pub fn disable_depth_test(&mut self) {
        self.depth_stencil.depth_test_enable = vk::FALSE;
        self.depth_stencil.depth_write_enable = vk::FALSE;
        self.depth_stencil.depth_compare_op = vk::CompareOp::NEVER;
        self.depth_stencil.depth_bounds_test_enable = vk::FALSE;
        self.depth_stencil.stencil_test_enable = vk::FALSE;
        self.depth_stencil.front = vk::StencilOpState::default();
        self.depth_stencil.back = vk::StencilOpState::default();
        self.depth_stencil.min_depth_bounds = 0f32;
        self.depth_stencil.max_depth_bounds = 1f32;
    }
}