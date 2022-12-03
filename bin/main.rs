use std::f32::consts::PI;

use automata_lib::{Automata, AutomataRenderer};
use std::time::{Duration, Instant};

use glam::{u32::UVec3, Mat4, Vec3};

use wgpu::{util::DeviceExt, BindGroupLayout, Device, Texture, TextureFormat, TextureView};
use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const FRAME_DELAY: Duration = Duration::new(0, 100000000);

fn fresh_automata(
    device: &Device,
    bind_group_layout: &BindGroupLayout,
    swapchain_format: TextureFormat,
    dim: UVec3,
    p: f32,
) -> AutomataRenderer {
    AutomataRenderer::new(
        &device,
        &bind_group_layout,
        swapchain_format,
        Automata::new(&dim, p, &device),
    )
}

fn generate_depth_buffer(
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

async fn run(event_loop: EventLoop<()>, window: Window) {
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

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    let swapchain_format = surface.get_supported_formats(&adapter)[0];

    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
    };

    surface.configure(&device, &config);

    // TODO: Move this bind group logic to its own home
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

    let mx_total = Mat4::IDENTITY;
    let mx_ref: &[f32; 16] = mx_total.as_ref();

    let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Projection Matrix"),
        contents: bytemuck::cast_slice(mx_ref),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buf.as_entire_binding(),
        }],
        label: None,
    });

    let (_, mut draw_depth_buffer_view) = generate_depth_buffer(&device, &config);

    let mut last_draw = Instant::now();
    let half_dim = 60;
    let automata_dim = UVec3::new(half_dim * 2, half_dim * 2, 3);
    let automata_p = 0.01;
    let mut automata_renderer = fresh_automata(
        &device,
        &bind_group_layout,
        swapchain_format,
        automata_dim,
        automata_p,
    );

    let mut since_last_update = FRAME_DELAY;
    let mut x_rotation = 0.;

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // `event_loop.run` never returns, therefore we must do this to ensure
        // the resources are properly cleaned up.
        let _ = (&instance, &adapter, &pipeline_layout);

        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);

                (_, draw_depth_buffer_view) = generate_depth_buffer(&device, &config);

                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::R),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // On 'R' reset the automata
                automata_renderer = fresh_automata(
                    &device,
                    &bind_group_layout,
                    swapchain_format,
                    automata_dim,
                    automata_p,
                );
            }
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let elapsed = now - last_draw;
                //x_rotation += PI * (elapsed.as_secs_f32() / 10.);

                let frame = surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // If update delay has passed then use a compute
                // pipeline to update the automata
                since_last_update += elapsed;

                if since_last_update >= FRAME_DELAY {
                    since_last_update = Duration::new(0, 0);
                    automata_renderer.automata.update(&device, &queue);
                }

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &draw_depth_buffer_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    let projection = glam::Mat4::perspective_rh(
                        90. * (std::f32::consts::PI / 180.),
                        config.width as f32 / config.height as f32,
                        0.1,
                        150.,
                    );

                    let view = Mat4::from_translation(Vec3::new(0., 0., -60.));

                    let rotation = Mat4::from_rotation_y(x_rotation);

                    let projection_by_view = projection * view * rotation;

                    queue.write_buffer(
                        &uniform_buf,
                        0,
                        bytemuck::cast_slice(projection_by_view.as_ref()),
                    );

                    rpass.set_bind_group(0, &bind_group, &[]);
                    automata_renderer.draw(&mut rpass);
                }

                queue.submit(Some(encoder.finish()));
                frame.present();

                last_draw = now;
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        // Temporarily avoid srgb formats for the swapchain on the web
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
        // On wasm, append the canvas to the document body
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        wasm_bindgen_futures::spawn_local(run(event_loop, window));
    }
}