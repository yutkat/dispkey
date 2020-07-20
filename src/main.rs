mod keylogs;

use crate::keylogs::KeyLogs;
use anyhow::Result;
use log::debug;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::{WindowBuilderExtUnix, XWindowType},
    window::WindowBuilder,
};

fn main() -> Result<()> {
    pretty_env_logger::init();
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_transparent(true)
        .with_visible(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_resizable(true)
        .with_inner_size(winit::dpi::LogicalSize::new(256.0, 128.0))
        .with_x11_window_type(vec![XWindowType::Utility])
        .build(&event_loop)
        .unwrap();

    let surface = wgpu::Surface::create(&window);

    // Initialize GPU
    let (device, queue) = futures::executor::block_on(async {
        let adapter = wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::all(),
        )
        .await
        .expect("Request adapter");

        adapter
            .request_device(&wgpu::DeviceDescriptor {
                extensions: wgpu::Extensions {
                    anisotropic_filtering: false,
                },
                limits: wgpu::Limits { max_bind_groups: 1 },
            })
            .await
    });

    // Prepare swap chain
    let render_format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let mut size = window.inner_size();

    let mut swap_chain = device.create_swap_chain(
        &surface,
        &wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: render_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        },
    );

    // Prepare glyph_brush
    let inconsolata = ab_glyph::FontArc::try_from_slice(include_bytes!("Inconsolata-Regular.ttf"))?;

    let mut glyph_brush = GlyphBrushBuilder::using_font(inconsolata).build(&device, render_format);

    // Render loop
    window.request_redraw();

    let mut keys = KeyLogs::new();
    let mut last_frame_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: winit::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = winit::event_loop::ControlFlow::Exit,
            Event::WindowEvent {
                event: winit::event::WindowEvent::Resized(new_size),
                ..
            } => {
                size = new_size;

                swap_chain = device.create_swap_chain(
                    &surface,
                    &wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                        format: render_format,
                        width: size.width,
                        height: size.height,
                        present_mode: wgpu::PresentMode::Mailbox,
                    },
                );
            }
            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        virtual_keycode: Some(key),
                        state: ElementState::Pressed,
                        ..
                    }),
                ..
            } => {
                debug!("press {:?}", key);
                keys.push(format!("{:?}\n", key));
                window.request_redraw();
            }
            Event::MainEventsCleared => {}
            Event::RedrawRequested { .. } => {
                println!("Redraw");
                glyph_brush.queue(Section {
                    screen_position: (30.0, 10.0),
                    bounds: (size.width as f32, size.height as f32),
                    text: keys
                        .get_keys_from_last(4)
                        .iter()
                        .map(|x| {
                            Text::new(&x)
                                .with_color([0.0, 0.0, 0.0, 1.0])
                                .with_scale(30.0)
                        })
                        .collect(),
                    ..Section::default()
                });
                // Get a command encoder for the current frame
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Redraw"),
                });

                // Get the next frame
                let frame = swap_chain.get_next_texture().expect("Get next frame");

                // Clear frame
                {
                    let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color {
                                r: 0.4,
                                g: 0.4,
                                b: 0.4,
                                a: 1.0,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
                }

                // Draw the text!
                glyph_brush
                    .draw_queued(&device, &mut encoder, &frame.view, size.width, size.height)
                    .expect("Draw queued");

                queue.submit(&[encoder.finish()]);

                last_frame_time = std::time::Instant::now();
            }
            _ => (),
        }
        if std::time::Instant::now() - last_frame_time >= std::time::Duration::from_secs(1) {
            window.request_redraw();
        }
    });
}
