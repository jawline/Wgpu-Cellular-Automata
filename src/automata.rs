use glam::u32::UVec3;
use std::borrow::Cow;
use std::mem;
use std::ops::Range;
use std::sync::mpsc::channel;
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, ComputePipeline, Device, Queue,
    RenderPass, RenderPipeline, TextureFormat,
};

const NUM_VERTICES_PER_BLOCK: u32 = 24;

pub struct Automata {
    pub dim: UVec3,
    pub size: u32,
    pub pipeline: ComputePipeline,
    pub dim_buffer: Buffer,
    pub buffers: [Buffer; 2],
    pub bind_groups: Vec<BindGroup>,
    pub buffer_idx: usize,
    //pub staging_buffer: Buffer,
}

impl Automata {
    pub fn new(dim: &UVec3, device: &Device) -> Self {
        let initial_state: Vec<u32> = (0..(dim.x * dim.y * dim.z))
            .map(|_| if rand::random::<f32>() <= 0.2 { 1 } else { 0 })
            .collect();

        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/compute_automata.wgsl"
            ))),
        });

        let slice_size = initial_state.len() * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;

        /*let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });*/

        let automata_dim_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Automata Tensor Dimensions"),
            contents: bytemuck::cast_slice(dim.as_ref()),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let automata_buffers = [
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Automata Tensor 1"),
                contents: bytemuck::cast_slice(&initial_state),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            }),
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Automata Tensor 2"),
                contents: bytemuck::cast_slice(&initial_state),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            }),
        ];

        let uvec3_layout = |i| wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(12),
            },
            count: None,
        };

        let tensor_layout = |i| wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(size),
            },
            count: None,
        };

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[uvec3_layout(0), tensor_layout(1), tensor_layout(2)],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("automata pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &cs_module,
            entry_point: "main",
        });

        let bind_group_layout = pipeline.get_bind_group_layout(0);

        let bind_groups: Vec<BindGroup> = (0..2)
            .map(|offset| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: automata_dim_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: automata_buffers[(offset + 1) % 2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: automata_buffers[(offset + 1) % 2].as_entire_binding(),
                        },
                    ],
                })
            })
            .collect();

        Self {
            dim: *dim,
            dim_buffer: automata_dim_buffer,
            buffers: automata_buffers,
            //staging_buffer,
            pipeline,
            bind_groups,
            buffer_idx: 0,
            size: dim.x * dim.y * dim.z,
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) {
        let buffer_idx = self.buffer_idx;
        self.buffer_idx = (self.buffer_idx + 1) % 2;

        let bind_group = &self.bind_groups[buffer_idx];
        let output_buffer = &self.buffers[(buffer_idx + 1) % 2];

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            cpass.insert_debug_marker("A single automata iteration");
            cpass.dispatch_workgroups(self.dim.x as u32, self.dim.y as u32, self.dim.z as u32);
        }

        /*
        encoder.copy_buffer_to_buffer(
            output_buffer,
            0,
            &self.staging_buffer,
            0,
            self.size as u64 * mem::size_of::<u32>() as u64,
        );*/
        queue.submit(Some(encoder.finish()));

        //let buffer_slice = self.staging_buffer.slice(..);
        //let (sender, receiver) = channel();
        //buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // TODO: This is a busy poll for the final result. We don't actually need this on the GPU
        // so maybe just pass the buffer back to a shader instead?
        //device.poll(wgpu::Maintain::Wait);
        //receiver.recv().unwrap().unwrap();

        //let data = buffer_slice.get_mapped_range();
        //let result = bytemuck::cast_slice(&data).to_vec();
        //drop(data);
        //self.staging_buffer.unmap();
        //result
    }
}

pub struct AutomataRenderer {
    pub pipeline: RenderPipeline,
    pub swapchain_format: TextureFormat,
    pub bind_groups: Vec<BindGroup>,
    pub automata: Automata,
}

impl AutomataRenderer {
    pub fn new(
        device: &Device,
        bind_group_layout: &BindGroupLayout,
        swapchain_format: TextureFormat,
        automata: Automata,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/render_automata.wgsl"
            ))),
        });

        let uvec3_layout = |i| wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(12),
            },
            count: None,
        };

        let tensor_layout = |i| wgpu::BindGroupLayoutEntry {
            binding: i,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(automata.size as u64),
            },
            count: None,
        };

        let automata_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[uvec3_layout(0), tensor_layout(1)],
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout, &automata_bind_group_layout],
            push_constant_ranges: &[],
        });

        let bind_groups: Vec<BindGroup> = (0..2)
            .map(|offset| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &automata_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: automata.dim_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: automata.buffers[(offset + 1) % 2].as_entire_binding(),
                        },
                    ],
                })
            })
            .collect();

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
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: /* TODO: Add a depth buffer */ None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            automata,
            pipeline,
            swapchain_format,
            bind_groups,
        }
    }

    pub fn draw<'pass, 'automata: 'pass>(&'automata self, pass: &mut RenderPass<'pass>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(
            1,
            /* TODO: This indexing is pretty awkward */
            &self.bind_groups[self.automata.buffer_idx],
            &[],
        );
        pass.draw(
            0..self.automata.size * NUM_VERTICES_PER_BLOCK, /* TODO: Sub in number of triangles per cube */
            0..1,
        );
    }
}
