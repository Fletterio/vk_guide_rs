use crate::vk_types::buffers::{destroy_buffer, AllocatedBuffer};
use crate::{immediate_submit, vk_types};
use ash::vk::DeviceSize;
use ash::{vk, Device};
use std::mem::size_of;
use std::slice;
use vk_types::buffers::create_buffer;
use crate::vk_types::vertex::Vertex;

// holds the resources needed for a mesh
pub struct GPUMeshBuffers {
    pub index_buffer: AllocatedBuffer,
    pub vertex_buffer: AllocatedBuffer,
    pub vertex_buffer_address: vk::DeviceAddress,
}

pub fn upload_mesh(
    device: &Device,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    indices: &[u32],
    vertices: &[Vertex],
    immediate_command_buffer: vk::CommandBuffer,
    immediate_fence: vk::Fence,
    immediate_queue: vk::Queue,
) -> GPUMeshBuffers {
    let vertex_buffer_size: usize = vertices.len() * size_of::<Vertex>();
    let index_buffer_size: usize = indices.len() * size_of::<u32>();

    //create vertex buffer
    let vertex_buffer = create_buffer(
        device,
        allocator,
        vertex_buffer_size as DeviceSize,
        vk::BufferUsageFlags::STORAGE_BUFFER
            | vk::BufferUsageFlags::TRANSFER_DST
            | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS,
        gpu_allocator::MemoryLocation::GpuOnly,
    );

    //find the address of the vertex buffer
    let device_address_info = vk::BufferDeviceAddressInfo::builder()
        .buffer(vertex_buffer.buffer)
        .build();
    let vertex_buffer_address = unsafe { device.get_buffer_device_address(&device_address_info) };

    //create index buffer
    let index_buffer = create_buffer(
        device,
        allocator,
        index_buffer_size as DeviceSize,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuOnly,
    );

    //upload buffer
    let mut staging = create_buffer(
        device,
        allocator,
        (vertex_buffer_size + index_buffer_size) as DeviceSize,
        vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    presser::copy_from_slice_to_offset(vertices, staging.allocation.get_mut().unwrap(), 0).unwrap();
    presser::copy_from_slice_to_offset(
        indices,
        staging.allocation.get_mut().unwrap(),
        vertex_buffer_size,
    )
    .unwrap();

    let upload_helper = |cmd: vk::CommandBuffer| {
        let vertex_copy = vk::BufferCopy::builder()
            .dst_offset(0)
            .src_offset(0)
            .size(vertex_buffer_size as DeviceSize)
            .build();
        unsafe {
            device.cmd_copy_buffer(
                cmd,
                staging.buffer,
                vertex_buffer.buffer,
                slice::from_ref(&vertex_copy),
            )
        };

        let index_copy = vk::BufferCopy::builder()
            .dst_offset(0)
            .src_offset(vertex_buffer_size as DeviceSize)
            .size(index_buffer_size as DeviceSize)
            .build();
        unsafe {
            device.cmd_copy_buffer(
                cmd,
                staging.buffer,
                index_buffer.buffer,
                slice::from_ref(&index_copy),
            )
        };
    };
    immediate_submit!(
        device,
        immediate_command_buffer,
        immediate_fence,
        immediate_queue,
        upload_helper,
        immediate_command_buffer
    );
    destroy_buffer(device, allocator, &mut staging);
    GPUMeshBuffers {
        index_buffer,
        vertex_buffer,
        vertex_buffer_address,
    }
}

impl GPUMeshBuffers {
    pub fn dealloc(&mut self, device: &Device, allocator: &mut gpu_allocator::vulkan::Allocator) {
        destroy_buffer(device, allocator, &mut self.vertex_buffer);
        destroy_buffer(device, allocator, &mut self.index_buffer);
    }
}