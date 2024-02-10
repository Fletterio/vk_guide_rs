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