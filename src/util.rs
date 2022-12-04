use wgpu::{Device, Texture, TextureView};

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
