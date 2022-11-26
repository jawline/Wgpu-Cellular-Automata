use log::debug;
use std::error::Error;
use std::mem;

use std::borrow::Cow;
use std::ops::Range;
use std::time::Duration;

use failure::format_err;

use glam::{Mat4, Quat, Vec3};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, RenderPipeline,
    TextureFormat, VertexAttribute, VertexBufferLayout,
};

use super::data::Data;
use super::mesh_vertex::Vertex;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

pub struct MeshRenderState {
    pub pipeline: RenderPipeline,
    pub swapchain_format: TextureFormat,
}

impl MeshRenderState {
    pub fn create(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        swapchain_format: TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../shaders/mesh.wgsl"
            ))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        debug!("Vertex buffer stride: {:?}", mem::size_of::<Vertex>());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Mesh::desc(), MeshInstances::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: /* TODO: Add a depth buffer */ None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            swapchain_format,
        }
    }
}

pub struct Face {}

impl Face {
    pub fn of_data(data: &Data, index: usize) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        let face = &data.faces[index];

        for element in face {
            let vertex = Vertex::new(
                &data.vertices[element.vertex - 1],
                &element
                    .texture_coordinate
                    .map(|index| data.texture_coordinates[index - 1]),
                &element.normal.map(|index| data.vertex_normals[index - 1]),
            );
            vertices.push(vertex);
        }

        vertices
    }
}

pub struct MeshInstance {
    pub position: Vec3,
    pub rotation: Quat,
}

impl MeshInstance {
    fn to_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position)
    }
}

pub struct MeshInstances {
    pub instances: Vec<MeshInstance>,
    buffer: Buffer,
}

impl MeshInstances {
    pub fn new(instances: Vec<MeshInstance>, device: &Device) -> Self {
        let instance_data: Vec<_> = instances
            .iter()
            .map(|x| x.to_matrix().as_ref().clone())
            .collect();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mesh Instance Buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self { instances, buffer }
    }

    pub fn count(&self) -> u32 {
        self.instances.len() as u32
    }

    pub fn update(&self, queue: &Queue) {
        // TODO: Duplication of instance data
        let instance_data: Vec<_> = self
            .instances
            .iter()
            .map(|x| x.to_matrix().as_ref().clone())
            .collect();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&instance_data));
    }

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        const ATTRS: [VertexAttribute; 4] = wgpu::vertex_attr_array![5 => Float32x4 , 6 => Float32x4 , 7 => Float32x4 , 8 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRS,
        }
    }
}

pub struct Mesh {
    pub vertices: Buffer,
    face_indices: Vec<(u32, u32)>,
    total_vertices: usize,
}

impl Mesh {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        const ATTRS: [VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x3, 2 => Float32x3];
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRS,
        }
    }

    pub fn of_file(device: &Device, file: &str) -> Result<Self, Box<dyn Error>> {
        let obj_data = Data::from_file(file)?;
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut face_indices = Vec::new();

        let mut current_range = 0;

        for face in 0..obj_data.faces.len() {
            let face_data = Face::of_data(&obj_data, face);

            face_indices.push((
                current_range as u32,
                (current_range + face_data.len()) as u32,
            ));

            current_range += face_data.len();

            match face_data.len() {
                3 => {
                    vertices.push(face_data[0]);
                    vertices.push(face_data[1]);
                    vertices.push(face_data[2]);
                }
                4 => {
                    vertices.push(face_data[0]);
                    vertices.push(face_data[1]);
                    vertices.push(face_data[2]);

                    vertices.push(face_data[1]);
                    vertices.push(face_data[2]);
                    vertices.push(face_data[3]);
                }
                _ => {
                    return Err(format_err!("{} face data length", face_data.len()).into());
                }
            };
        }

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Ok(Self {
            vertices,
            face_indices,
            total_vertices: current_range,
        })
    }

    pub fn draw<'pass, 'mesh: 'pass>(
        &'mesh self,
        pass: &mut RenderPass<'pass>,
        render_state: &'mesh MeshRenderState,
        instances: &'mesh MeshInstances,
        instance_range: Range<u32>,
    ) {
        pass.set_pipeline(&render_state.pipeline);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_vertex_buffer(1, instances.buffer.slice(..));
        pass.draw(0..self.total_vertices as u32, instance_range);
    }
}
