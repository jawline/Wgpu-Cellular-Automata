use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;

use std::borrow::Cow;
use std::time::Duration;

use glam::{Mat4, Vec3, Vec4};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, RenderPipeline,
    TextureFormat,
};

pub struct FaceElement {
    pub vertex: usize,
    pub texture_coordinate: Option<usize>,
    pub normal: Option<usize>,
}

impl FaceElement {
    fn of_string(line: &str) -> Result<FaceElement, Box<dyn Error>> {
        let mut parts = line.split('/');

        let vertex = parts
            .next()
            .ok_or("face element is not in the form vertex/texture_coord/normal")?
            .parse::<usize>()?;
        let texture_coordinate = parts.next();
        let normal = parts.next();

        let (texture_coordinate, normal) = match (texture_coordinate, normal) {
            (Some(texture_coordinate), Some(normal)) => {
                let texture_coordinate = match texture_coordinate {
                    "" => None,
                    v => Some(v.parse::<usize>()?),
                };

                let normal = match normal {
                    "" => None,
                    v => Some(v.parse::<usize>()?),
                };
                (texture_coordinate, normal)
            }
            _ => (None, None),
        };

        Ok(Self {
            vertex: vertex,
            texture_coordinate,
            normal,
        })
    }
}

pub struct ObjData {
    pub vertices: Vec<Vec4>,
    pub texture_coordinates: Vec<Vec3>,
    pub vertex_normals: Vec<Vec3>,
    pub faces: Vec<Vec<FaceElement>>,
}

impl ObjData {
    fn read_line(&mut self, line: &str) -> Result<(), Box<dyn Error>> {
        if line == "" {
            return Ok(());
        }
        let mut parts = line.trim().split(' ');

        match parts.next() {
            Some("v") => {
                /* Vertex */
                let mut parts = parts.map(|x| x.parse::<f32>());
                let x = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;
                let y = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;
                let z = parts.next().ok_or::<Box<dyn Error>>(
                    "obj v is not followed by three or four floats".into(),
                )??;

                let w = parts.next().unwrap_or(Ok(1.0))?;

                self.vertices.push(Vec4::new(x, y, z, w));
                Ok(())
            }
            Some("tc") => {
                /* Texture coordinate */
                let mut parts = parts.map(|x| x.parse::<f32>());
                let u = parts.next().ok_or::<Box<dyn Error>>(
                    "obj tc is not followed by one, two or three floats".into(),
                )??;

                let v = parts.next().unwrap_or(Ok(0.0))?;
                let w = parts.next().unwrap_or(Ok(0.0))?;

                self.texture_coordinates.push(Vec3::new(u, v, w));

                Ok(())
            }
            Some("vn") => {
                /* Vertex normal */

                let mut parts = parts.map(|x| x.parse::<f32>());

                let x = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                let y = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                let z = parts
                    .next()
                    .ok_or::<Box<dyn Error>>("obj vn is not followed by three floats".into())??;

                self.vertex_normals.push(Vec3::new(x, y, z));

                Ok(())
            }
            Some("vp") => {
                /* Parameter space vertices (TODO) */
                panic!("I don't know this format");
            }
            Some("f") => {
                /* Face */
                let mut face_elements = Vec::new();
                for part in parts {
                    let new_face = FaceElement::of_string(part)?;
                    face_elements.push(new_face);
                }
                self.faces.push(face_elements);
                Ok(())
            }
            Some("l") => panic!("polylines are unsupported"),
            Some("#") => {
                /* Comment line */
                Ok(())
            }
            Some("mtllib") =>
            /* TODO: Support materials */
            {
                Ok(())
            }
            Some("usemtl") =>
            /* TODO: Support materials */
            {
                Ok(())
            }
            Some("g") =>
            /* TODO: Support groups */
            {
                Ok(())
            }
            Some("s") =>
            /* TODO: Support smooth shading */
            {
                Ok(())
            }
            Some(part) => Err(format!(
                "obj in bad format: {} {:?}",
                part,
                parts.collect::<Vec<&str>>()
            )
            .into()),
            None => {
                /* Empty line */
                Ok(())
            }
        }
    }

    pub fn from_file(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let mut obj_data = ObjData {
            vertices: Vec::new(),
            texture_coordinates: Vec::new(),
            vertex_normals: Vec::new(),
            faces: Vec::new(),
        };
        for line in BufReader::new(File::open(filepath)?).lines() {
            obj_data.read_line(&line?)?;
        }
        Ok(obj_data)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    _pos: [f32; 4],
    _tex_coord: [f32; 3],
    _normal: [f32; 3],
}

pub struct MeshRenderState {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub swapchain_format: TextureFormat,
}

impl MeshRenderState {
    pub fn create(device: &Device, swapchain_format: TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/triangles.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex> as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 4 * 4,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 4 * 7,
                    shader_location: 2,
                },
            ],
        }];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            swapchain_format,
        }
    }
}

pub struct Mesh {
    pub pos: Vec3,
    pub velocity: Vec3,
    pos_matrix: Buffer,
    pub bind_group: BindGroup,
}

impl Mesh {
    pub fn of_file(
        device: &Device,
        pos: Vec3,
        velocity: Vec3,
        render_state: &MeshRenderState,
        file: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let obj_data = ObjData::from_file(file)?;

        let vertices: Vec<Vertex> = obj_data
            .vertices
            .iter()
            .map(|vertex| Vertex {
                _pos: [vertex[0], vertex[1], vertex[2], vertex[3]],
                _tex_coord: [0., 0., 0.],
                _normal: [0., 0., 0.],
            })
            .collect();

        let mut index_data: Vec<usize> = Vec::new();

        for face in obj_data.faces.iter() {
            index_data.extend(face.iter().map(|face| face.vertex));
        }

        println!("{:?} {:?}", vertices, index_data);

        let mx_total = Mat4::IDENTITY;

        let mx_ref: &[f32; 16] = mx_total.as_ref();

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_state.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        Ok(Self {
            pos,
            velocity,
            pos_matrix: uniform_buf,
            bind_group,
        })
    }

    fn to_matrix(&self) -> Mat4 {
        Mat4::IDENTITY + Mat4::from_translation(self.pos)
    }

    pub fn update(&mut self, elapsed: Duration, queue: &Queue) {
        self.pos += self.velocity * elapsed.as_secs_f32();

        if self.pos.x < -1. || self.pos.x > 1. {
            self.velocity.x = -self.velocity.x;
        }

        if self.pos.y < -1. || self.pos.y > 1. {
            self.velocity.y = -self.velocity.y;
        }

        queue.write_buffer(
            &self.pos_matrix,
            0,
            bytemuck::cast_slice(self.to_matrix().as_ref()),
        );
    }

    pub fn draw(&self, pass: &mut RenderPass) {
        pass.draw(0..3, 0..1);
    }
}
