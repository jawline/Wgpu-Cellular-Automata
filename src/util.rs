use glam::{Mat4, UVec3};
use wgpu::{util::DeviceExt, Buffer, Device, Texture, TextureView};

pub fn generate_depth_buffer(
    device: &Device,
    config: &wgpu::SurfaceConfiguration,
) -> (Texture, TextureView) {
    let texture_extent = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };

    let draw_depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Buffer"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::RENDER_ATTACHMENT,
    });
    let draw_depth_buffer_view =
        draw_depth_buffer.create_view(&wgpu::TextureViewDescriptor::default());
    (draw_depth_buffer, draw_depth_buffer_view)
}

pub fn uvec_buffer(device: &Device, initial: &UVec3) -> Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Generated UVec"),
        contents: bytemuck::cast_slice(initial.as_ref()),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn mat4_identity(device: &Device) -> Buffer {
    let mx_total = Mat4::IDENTITY;
    let mx_ref: &[f32; 16] = mx_total.as_ref();

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Projection Matrix"),
        contents: bytemuck::cast_slice(mx_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}
