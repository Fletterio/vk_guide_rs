use std::slice;
use ash::vk;

pub fn command_pool_create_info(queue_family_index : u32, flags : vk::CommandPoolCreateFlags) -> vk::CommandPoolCreateInfo {
    vk::CommandPoolCreateInfo::builder()
        .flags(flags)
        .queue_family_index(queue_family_index)
        .build()
}

pub fn command_buffer_allocate_info(pool : vk::CommandPool, count : u32) -> vk::CommandBufferAllocateInfo {
    vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .command_buffer_count(count)
        .level(vk::CommandBufferLevel::PRIMARY)
        .build()
}

pub fn fence_create_info(flags : vk::FenceCreateFlags) -> vk::FenceCreateInfo {
    vk::FenceCreateInfo::builder()
        .flags(flags)
        .build()
}

pub fn semaphore_create_info(flags : vk::SemaphoreCreateFlags) -> vk::SemaphoreCreateInfo {
    vk::SemaphoreCreateInfo::builder()
        .flags(flags)
        .build()
}

pub fn command_buffer_begin_info(flags : vk::CommandBufferUsageFlags) -> vk::CommandBufferBeginInfo {
    vk::CommandBufferBeginInfo::builder()
        .flags(flags)
        .build()
}

pub fn image_subresource_range(aspect_mask : vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .base_array_layer(0)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
        .build()
}

pub fn semaphore_submit_info(stage_mask : vk::PipelineStageFlags2, semaphore : vk::Semaphore) -> vk::SemaphoreSubmitInfo {
    vk::SemaphoreSubmitInfo::builder()
        .semaphore(semaphore)
        .stage_mask(stage_mask)
        .device_index(0)
        .value(1)
        .build()
}

pub fn command_buffer_submit_info(cmd : vk::CommandBuffer) -> vk::CommandBufferSubmitInfo {
    vk::CommandBufferSubmitInfo::builder()
        .command_buffer(cmd)
        .device_mask(0)
        .build()
}

pub fn submit_info(cmd_submit_info : &vk::CommandBufferSubmitInfo, signal_semaphore_info : Option<&vk::SemaphoreSubmitInfo>, wait_semaphore_info : Option<&vk::SemaphoreSubmitInfo>) -> vk::SubmitInfo2 {
    vk::SubmitInfo2::builder()
        .wait_semaphore_infos(match wait_semaphore_info {
            Some(info) => {slice::from_ref(&info)},
            None => &[]
        })
        .signal_semaphore_infos(match signal_semaphore_info {
            Some(info) => {slice::from_ref(&info)},
            None => &[]
        })
        .command_buffer_infos(slice::from_ref(cmd_submit_info))
        .build()
}