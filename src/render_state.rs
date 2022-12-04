use glam::Mat4;
use wgpu::{
    BindGroup, BindGroupLayout, Buffer, Device, Instance, Queue, Surface, SurfaceConfiguration,
    Texture, TextureFormat, TextureView,
};
use winit::window::Window;
pub struct RenderState {
    pub instance: Instance,
    pub surface: Surface,
    pub config: SurfaceConfiguration,
    pub swapchain_format: TextureFormat,
    pub queue: Queue,
    pub device: Device,
    pub general_bind_group_layout: BindGroupLayout,
    pub general_bind_group: BindGroup,
    pub projection_buffer: Buffer,
    pub depth_buffer: Texture,
    pub depth_buffer_view: TextureView,
}

impl RenderState {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let swapchain_format = surface.get_supported_formats(&adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let general_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let projection_buffer = crate::util::mat4_identity(&device);

        let general_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &general_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: projection_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let (depth_buffer, depth_buffer_view) =
            crate::util::generate_depth_buffer(&device, &config);

        RenderState {
            instance,
            surface,
            swapchain_format,
            config,
            queue,
            device,
            depth_buffer,
            depth_buffer_view,
            general_bind_group_layout,
            general_bind_group,
            projection_buffer,
        }
    }

    pub fn regenerate_depth_buffer(&mut self) {
        let (depth_buffer, depth_buffer_view) =
            crate::util::generate_depth_buffer(&self.device, &self.config);
        self.depth_buffer = depth_buffer;
        self.depth_buffer_view = depth_buffer_view;
    }

    pub fn set_projection(&self, projection_matrix: Mat4) {
        self.queue.write_buffer(
            &self.projection_buffer,
            0,
            bytemuck::cast_slice(projection_matrix.as_ref()),
        );
    }

    pub fn reconfigure(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.regenerate_depth_buffer();
    }
}
