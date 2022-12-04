use automata_lib::*;
use std::time::{Duration, Instant};

use glam::{u32::UVec3, Mat4, Vec3};

use wgpu::{BindGroupLayout, Device, TextureFormat};
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
    dsl: Statement,
) -> AutomataRenderer {
    AutomataRenderer::new(
        &device,
        &bind_group_layout,
        swapchain_format,
        Automata::new(&dim, p, dsl, &device),
    )
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let mut render_state = RenderState::new(&window).await;
    let mut last_draw = Instant::now();
    let automata_dim = UVec3::new(500, 500, 3);
    let automata_p = 0.02;
    let automata_rules = conways_game_of_life();

    let mut automata_renderer = fresh_automata(
        &render_state.device,
        &render_state.general_bind_group_layout,
        render_state.swapchain_format,
        automata_dim,
        automata_p,
        automata_rules.clone(),
    );

    let mut since_last_update = FRAME_DELAY;
    let mut x_off = 0.;
    let mut y_off = 0.;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_state.reconfigure(size.width, size.height);
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                y_off -= 1.;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                y_off += 1.;
            }

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                x_off += 1.;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                x_off -= 1.;
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
                    &render_state.device,
                    &render_state.general_bind_group_layout,
                    render_state.swapchain_format,
                    automata_dim,
                    automata_p,
                    automata_rules.clone(),
                );
            }
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let elapsed = now - last_draw;

                let frame = render_state
                    .surface
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
                    automata_renderer
                        .automata
                        .update(&render_state.device, &render_state.queue);
                }

                let mut encoder = render_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
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
                            view: &render_state.depth_buffer_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    let projection = glam::Mat4::perspective_rh(
                        70. * (std::f32::consts::PI / 180.),
                        render_state.config.width as f32 / render_state.config.height as f32,
                        0.1,
                        1500.,
                    );

                    let view = Mat4::from_translation(Vec3::new(x_off, y_off, -250.));
                    let rotation = Mat4::from_rotation_y(x_rotation);
                    render_state.set_projection(projection * view * rotation);

                    rpass.set_bind_group(0, &render_state.general_bind_group, &[]);
                    automata_renderer.draw(&mut rpass);
                }

                render_state.queue.submit(Some(encoder.finish()));
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
        pollster::block_on(run(event_loop, window));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        use winit::platform::web::WindowExtWebSys;
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
