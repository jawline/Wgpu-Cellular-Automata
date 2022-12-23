use automata_lib::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use glam::u32::UVec3;

use winit::{
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

const FRAME_DELAY: Duration = Duration::new(0, 50000000);

async fn run(event_loop: EventLoop<()>, window: Window) {
    let render_state = Rc::new(RefCell::new(RenderState::new(&window).await));
    let mut last_draw = Instant::now();
    let automata_dim = UVec3::new(500, 500, 3);
    let automata_p = 0.02;
    let automata_rules = conways_game_of_life();

    let render_ref = render_state.clone();
    let fresh_automata = move || {
        let render_ref = render_ref.borrow();
        AutomataRenderer::new(
            &render_ref.device,
            &render_ref.general_bind_group_layout,
            render_ref.swapchain_format,
            Automata::new(
                &automata_dim,
                automata_p,
                automata_rules.clone(),
                &render_ref.device,
            ),
        )
    };

    let mut automata_renderer = fresh_automata();

    let mut since_last_update = FRAME_DELAY;
    let mut camera = SimpleCamera::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_state
                    .borrow_mut()
                    .reconfigure(size.width, size.height);
                window.request_redraw();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                use VirtualKeyCode::*;
                match keycode {
                    R => {
                        // On 'R' reset the automata
                        automata_renderer = fresh_automata();
                    }
                    _ => {}
                }
                camera.key(keycode, state);
            }
            Event::RedrawRequested(_) => {
                let render_state = render_state.borrow();
                let now = Instant::now();
                let elapsed = now - last_draw;

                camera.update(elapsed.as_secs_f32());

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

                    let view = camera.view();
                    render_state.set_projection(projection * view);

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
