use ash::{vk, Device};
use gpu_allocator::vulkan::{AllocationCreateDesc, AllocationScheme};
use std::cell::OnceCell;

pub struct AllocatedBuffer {
    pub buffer: vk::Buffer,
    pub allocation: OnceCell<gpu_allocator::vulkan::Allocation>,
}
pub fn create_buffer(
    device: &Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    allocation_size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    memory_location: gpu_allocator::MemoryLocation,
) -> AllocatedBuffer {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(allocation_size)
        .usage(usage)
        .build();

    let buffer = unsafe { device.create_buffer(&buffer_info, None).unwrap() };
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator
        .allocate(&AllocationCreateDesc {
            name: "buffer_allocation",
            requirements,
            location: memory_location,
            linear: true,
            allocation_scheme: AllocationScheme::DedicatedBuffer(buffer),
        })
        .unwrap();

    unsafe {
        device
            .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
            .unwrap()
    };

    AllocatedBuffer {
        buffer,
        allocation: allocation.into(),
    }
}

pub fn destroy_buffer(
    device: &Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    buffer: &mut AllocatedBuffer,
) {
    allocator.free(buffer.allocation.take().unwrap()).unwrap();
    unsafe { device.destroy_buffer(buffer.buffer, None) };
}
