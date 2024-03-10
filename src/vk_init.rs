use ash::vk;
use ash::vk::ClearValue;
use std::slice;

pub fn command_pool_create_info(
    queue_family_index: u32,
    flags: vk::CommandPoolCreateFlags,
) -> vk::CommandPoolCreateInfo {
    vk::CommandPoolCreateInfo::builder()
        .flags(flags)
        .queue_family_index(queue_family_index)
        .build()
}

pub fn command_buffer_allocate_info(
    pool: vk::CommandPool,
    count: u32,
) -> vk::CommandBufferAllocateInfo {
    vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .command_buffer_count(count)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build()
}

pub fn fence_create_info(flags: vk::FenceCreateFlags) -> vk::FenceCreateInfo {
    vk::FenceCreateInfo::builder().flags(flags).build()
}

pub fn semaphore_create_info(flags: vk::SemaphoreCreateFlags) -> vk::SemaphoreCreateInfo {
    vk::SemaphoreCreateInfo::builder().flags(flags).build()
}

pub fn command_buffer_begin_info(flags: vk::CommandBufferUsageFlags) -> vk::CommandBufferBeginInfo {
    vk::CommandBufferBeginInfo::builder().flags(flags).build()
}

pub fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .base_array_layer(0)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
        .build()
}

pub fn semaphore_submit_info(
    stage_mask: vk::PipelineStageFlags2,
    semaphore: vk::Semaphore,
) -> vk::SemaphoreSubmitInfo {
    vk::SemaphoreSubmitInfo::builder()
        .semaphore(semaphore)
        .stage_mask(stage_mask)
        .device_index(0)
        .value(1)
        .build()
}

pub fn command_buffer_submit_info(cmd: vk::CommandBuffer) -> vk::CommandBufferSubmitInfo {
    vk::CommandBufferSubmitInfo::builder()
        .command_buffer(cmd)
        .device_mask(0)
        .build()
}

pub fn submit_info(
    cmd_submit_info: &vk::CommandBufferSubmitInfo,
    signal_semaphore_info: Option<&vk::SemaphoreSubmitInfo>,
    wait_semaphore_info: Option<&vk::SemaphoreSubmitInfo>,
) -> vk::SubmitInfo2 {
    vk::SubmitInfo2::builder()
        .wait_semaphore_infos(match wait_semaphore_info {
            Some(info) => slice::from_ref(&info),
            None => &[],
        })
        .signal_semaphore_infos(match signal_semaphore_info {
            Some(info) => slice::from_ref(&info),
            None => &[],
        })
        .command_buffer_infos(slice::from_ref(cmd_submit_info))
        .build()
}

pub fn image_create_info(
    format: vk::Format,
    usage_flags: vk::ImageUsageFlags,
    extent: vk::Extent3D,
) -> vk::ImageCreateInfo {
    vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(extent)
        .mip_levels(1)
        .array_layers(1)
        //for MSAA. we will not be using it by default, so default it to 1 sample per pixel.
        .samples(vk::SampleCountFlags::TYPE_1)
        //optimal tiling, which means the image is stored on the best gpu format
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage_flags)
        .build()
}

pub fn image_view_create_info(
    format: vk::Format,
    image: vk::Image,
    aspect_flags: vk::ImageAspectFlags,
) -> vk::ImageViewCreateInfo {
    vk::ImageViewCreateInfo::builder()
        .view_type(vk::ImageViewType::TYPE_2D)
        .image(image)
        .format(format)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .aspect_mask(aspect_flags)
                .build(),
        )
        .build()
}

pub fn attachment_info(
    view: vk::ImageView,
    clear_value: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
) -> vk::RenderingAttachmentInfo {
    vk::RenderingAttachmentInfo::builder()
        .image_view(view)
        .image_layout(layout)
        .load_op(match clear_value {
            Some(_) => vk::AttachmentLoadOp::CLEAR,
            None => vk::AttachmentLoadOp::LOAD,
        })
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(clear_value.unwrap_or_else(|| ClearValue::default()))
        .build()
}

pub fn rendering_info(
    render_extent: vk::Extent2D,
    color_attachment: vk::RenderingAttachmentInfo,
    depth_attachment: Option<&vk::RenderingAttachmentInfo>,
) -> vk::RenderingInfo {
    let mut rendering_info = vk::RenderingInfo::builder()
        .render_area(vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: render_extent,
        })
        .layer_count(1)
        .color_attachments(slice::from_ref(&color_attachment));

    if let Some(depth) = depth_attachment {
        rendering_info = rendering_info.depth_attachment(depth);
    }

    rendering_info.build()
}

pub fn depth_attachment_info(view: vk::ImageView, layout: vk::ImageLayout) -> vk::RenderingAttachmentInfo {
    vk::RenderingAttachmentInfo::builder()
        .image_view(view)
        .image_layout(layout)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .clear_value(ClearValue {depth_stencil: Default::default()})
        .build()
}