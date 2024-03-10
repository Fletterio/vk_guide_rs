use std::cell::RefCell;
use std::fmt::Display;
use std::path::Path;
use std::rc::Rc;
use ash::{Device, vk};
use cfg_if::cfg_if;
use gltf::Semantic;
use crate::vk_types::gpu_mesh_buffers::{GPUMeshBuffers, upload_mesh};
use crate::vk_types::vertex::Vertex;

#[derive(Default)]
pub struct GeoSurface {
    pub start_index: u32,
    pub count: u32
}

pub struct MeshAsset {
    pub name: String,
    pub surfaces: Vec<GeoSurface>,
    pub mesh_buffers: GPUMeshBuffers
}

pub fn load_gltf_meshes<P: AsRef<Path> + ?Sized + Display>(device: &Device,
                        allocator: &mut gpu_allocator::vulkan::Allocator,
                        immediate_command_buffer: vk::CommandBuffer,
                        immediate_fence: vk::Fence,
                        immediate_queue: vk::Queue,
                        file_path: &P) -> Option<Vec<Rc<RefCell<MeshAsset>>>> {
    println!("Loading GLTF: {}", file_path);
    let import = gltf::import(file_path);
    if let Err(e) = import {
        println!("Failed to load glTF: {}", e);
        return None
    }
    let (gltf, buffers, _) = import.unwrap();
    let mut meshes = Vec::<Rc<RefCell<MeshAsset>>>::new();
    // use the same vectors for all meshes so that the memory doesnt reallocate as often
    let mut indices = Vec::<u32>::new();
    let mut vertices = Vec::<Vertex>::new();
    for mesh in gltf.meshes() {
        let name = String::from(mesh.name().unwrap());
        let mut surfaces = Vec::<GeoSurface>::new();
        // clear the mesh arrays each mesh, we dont want to merge them by error
        indices.clear();
        vertices.clear();
        for primitive in mesh.primitives() {
            let new_surface = GeoSurface {
                start_index: indices.len() as u32,
                count: primitive.indices().unwrap().count() as u32
            };
            let initial_vertex = vertices.len();
            //load indexes
            let index_accessor = &primitive.indices().unwrap();
            indices.reserve(index_accessor.count());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let primitive_indices = reader.read_indices().unwrap();
            for index in primitive_indices.into_u32() {
                indices.push(index + initial_vertex as u32);
            }

            //load vertex positions
            let position_accessor = &primitive.get(&Semantic::Positions).unwrap();
            vertices.reserve(position_accessor.count());
            let primitive_positions = reader.read_positions().unwrap();
            for position in primitive_positions {
                vertices.push(Vertex {
                    position: position.into(),
                    ..Default::default()
                })
            }
            //load vertex normals
            if let Some(primitive_normals) = reader.read_normals() {
                for (index, normal) in primitive_normals.enumerate() {
                    vertices[initial_vertex + index].normal = normal.into();
                }
            }
            //load UVs
            if let Some(primitive_uv) = reader.read_tex_coords(0) {
                for (index, uv) in primitive_uv.into_f32().enumerate() {
                    vertices[initial_vertex + index].uv_x = uv[0];
                    vertices[initial_vertex + index].uv_y = uv[1];
                }
            }
            //load vertex colors
            if let Some(primitive_colors) = reader.read_colors(0) {
                for (index, color) in primitive_colors.into_rgba_f32().enumerate() {
                    vertices[initial_vertex + index].color = color.into();
                }
            }
            //add the submesh info to surfaces vector
            surfaces.push(new_surface)
        }
        //painting vertex normals
        cfg_if!{
            if #[cfg(feature="vertex_normals")] {
                for vertex in vertices.iter_mut() {
                    vertex.color = vertex.normal.clone().extend(1f32);
                }
            }
        }
        let mesh_buffers = upload_mesh(device, allocator, &indices, &vertices, immediate_command_buffer, immediate_fence, immediate_queue);
        meshes.push(Rc::new(RefCell::new(MeshAsset {
            name,
            surfaces,
            mesh_buffers,
        })));
    }
    Some(meshes)
}