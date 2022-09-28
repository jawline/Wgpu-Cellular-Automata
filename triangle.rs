use std::borrow::Cow;
use std::time::Duration;

use glam::{Mat4, Vec3};
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, RenderPipeline,
    TextureFormat,
};

pub struct TriangleRenderState {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub swapchain_format: TextureFormat,
}

impl TriangleRenderState {
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

pub struct Triangle {
    pub pos: Vec3,
    pub velocity: Vec3,
    pos_matrix: Buffer,
    pub bind_group: BindGroup,
}

impl Triangle {
    pub fn create(
        device: &Device,
        pos: Vec3,
        velocity: Vec3,
        render_state: &TriangleRenderState,
    ) -> Self {
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

        Self {
            pos,
            velocity,
            pos_matrix: uniform_buf,
            bind_group,
        }
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
