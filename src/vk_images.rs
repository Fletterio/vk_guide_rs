use std::slice;
use ash::{Device, vk};
use crate::vk_init;

//using sync2 pipeline barrier to transition image layouts
pub fn transition_image(device :  &Device, cmd : vk::CommandBuffer, image : vk::Image, current_layout : vk::ImageLayout, new_layout : vk::ImageLayout) {
    let mut image_barrier = vk::ImageMemoryBarrier2::builder()
        .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
        .old_layout(current_layout)
        .new_layout(new_layout);

    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {vk::ImageAspectFlags::DEPTH} else {vk::ImageAspectFlags::COLOR};
    image_barrier = image_barrier.subresource_range(vk_init::image_subresource_range(aspect_mask))
        .image(image);

    let dependency_info = vk::DependencyInfo::builder()
        .image_memory_barriers(slice::from_ref(&image_barrier))
        .build();

    unsafe {device.cmd_pipeline_barrier2(cmd, &dependency_info)};
}