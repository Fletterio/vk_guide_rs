use crate::vk_init;
use ash::{vk, Device};
use std::slice;
use ash::vk::{ImageSubresourceLayers, Offset3D};

//using sync2 pipeline barrier to transition image layouts
pub fn transition_image(
    device: &Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    current_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
) {
    let mut image_barrier = vk::ImageMemoryBarrier2::builder()
        .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
        .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
        .old_layout(current_layout)
        .new_layout(new_layout);

    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
        vk::ImageAspectFlags::DEPTH
    } else {
        vk::ImageAspectFlags::COLOR
    };
    image_barrier = image_barrier
        .subresource_range(vk_init::image_subresource_range(aspect_mask))
        .image(image);

    let dependency_info = vk::DependencyInfo::builder()
        .image_memory_barriers(slice::from_ref(&image_barrier))
        .build();

    unsafe { device.cmd_pipeline_barrier2(cmd, &dependency_info) };
}

pub fn copy_image_to_image(device : &Device, cmd: vk::CommandBuffer, source: vk::Image, destination: vk::Image, src_size: vk::Extent2D, dst_size: vk::Extent2D) {
    let blit_region = vk::ImageBlit2::builder()
        .src_offsets([Default::default(), Offset3D{x : src_size.width as i32, y : src_size.height as i32, z : 1}])
        .dst_offsets([Default::default(), Offset3D{x : dst_size.width as i32, y : dst_size.height as i32, z : 1}])
        .src_subresource(ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .mip_level(0)
            .build())
        .dst_subresource(ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .mip_level(0)
            .build())
        .build();

    let blit_info = vk::BlitImageInfo2::builder()
        .dst_image(destination)
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_image(source)
        .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .filter(vk::Filter::LINEAR)
        .regions(slice::from_ref(&blit_region))
        .build();

    unsafe {device.cmd_blit_image2(cmd, &blit_info)};
}