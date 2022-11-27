mod automata;
mod obj;
mod polar;

use std::f32::consts::PI;

use automata::{Automata, AutomataRenderer};
use rand::random;
use std::time::{Duration, Instant};

use glam::{u32::UVec3, Mat4, Quat, Vec3};

use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use obj::{MeshInstance, MeshInstances, MeshRenderState};
use polar::Polar;

const FRAME_DELAY: Duration = Duration::new(0, 100000000);

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

    let mesh_render_state = MeshRenderState::create(&device, &bind_group_layout, swapchain_format);

    let mut last_draw = Instant::now();
    let mut automata = Automata::new(&UVec3::new(40, 40, 40), &device);
    let mut automata_renderer =
        AutomataRenderer::new(&device, &bind_group_layout, swapchain_format, automata);

    let mut since_last_update = FRAME_DELAY;

    let mut polar = Polar::new(20., 0., PI / 10.);

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
                println!("Resized");
                // Reconfigure the surface with the new size
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let elapsed = now - last_draw;
                polar.update(elapsed.as_secs_f32());

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
                    let frame = automata_renderer.automata.update(&device, &queue);
                    // TODO: Don't actually pull the frame back to the CPU unless
                    // we really need to.
                    //println!("{:?}", frame);
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
                        depth_stencil_attachment: None,
                    });

                    let projection = glam::Mat4::perspective_rh(
                        90. * (std::f32::consts::PI / 180.),
                        config.width as f32 / config.height as f32,
                        1.,
                        100.,
                    );

                    let (x, y) = polar.position();
                    //println!("{} {}", x, y);
                    let view =
                        Mat4::look_at_rh(Vec3::new(x, 10., y), Vec3::new(20., 20., 20.), Vec3::Z);

                    let projection_by_view = projection * view;

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
