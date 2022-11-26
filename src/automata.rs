use glam::u32::UVec3;
use std::borrow::Cow;
use std::sync::mpsc::{channel, Receiver, Sender};
use wgpu::{util::DeviceExt, BindGroup, Buffer, ComputePipeline, Device, Queue};

pub struct Automata {
    pub dim: UVec3,
    pub size: u64,
    pub pipeline: ComputePipeline,
    pub bind_group: BindGroup,
    pub dim_buffer: Buffer,
    pub buffer: Buffer,
    pub staging_buffer: Buffer,
}

impl Automata {
    // TODO: Consider using two buffers to store the cells
    pub fn new(dim: &UVec3, device: &Device) -> Self {
        let size = dim.x * dim.y * dim.z;
        let initial_state: Vec<u32> = (0..(dim.x * dim.y * dim.z))
            .map(|_| if rand::random::<f32>() <= 0.5 { 1 } else { 0 })
            .collect();

        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../shaders/compute_automata.wgsl"
            ))),
        });

        let slice_size = initial_state.len() * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let automata_dim_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Automata Tensor Dimensions"),
            contents: bytemuck::cast_slice(dim.as_ref()),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let automata_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Automata Tensor"),
            contents: bytemuck::cast_slice(&initial_state),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(12),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(size),
                    },
                    count: None,
                },
            ],
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: automata_dim_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: automata_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            dim: *dim,
            dim_buffer: automata_dim_buffer,
            buffer: automata_buffer,
            staging_buffer,
            pipeline,
            bind_group,
            size,
        }
    }

    pub fn update(&mut self, device: &Device, queue: &Queue) -> Vec<u32> {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.insert_debug_marker("iterate automata");
            cpass.dispatch_workgroups(self.size as u32, 1, 1);
        }

        encoder.copy_buffer_to_buffer(&self.buffer, 0, &self.staging_buffer, 0, self.size as u64);
        queue.submit(Some(encoder.finish()));

        let buffer_slice = self.staging_buffer.slice(..);
        let (sender, receiver) = channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        // TODO: This is a busy poll for the final result. We don't actually need this on the GPU
        // so maybe just pass the buffer back to a shader instead?
        device.poll(wgpu::Maintain::Wait);
        receiver.recv().unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        self.staging_buffer.unmap();
        result
    }
}
