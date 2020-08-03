mod key_converter;
mod keylogs;

use crate::keylogs::KeyLogs;
use anyhow::Result;
use clap::{load_yaml, App};
use gfx::{
    format::{Depth, Srgba8},
    Device,
};
use gfx_glyph::{ab_glyph::*, *};
use glutin::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::ControlFlow,
    platform::unix::{WindowBuilderExtUnix, XWindowType},
    window::CursorIcon,
};
use log::{error, info};
use old_school_gfx_glutin_ext::*;
use std::env;

fn main() -> Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "gfx_glyph=warn");
    }

    pretty_env_logger::init();

    if cfg!(target_os = "linux") {
        // winit wayland is currently still wip
        if env::var("WINIT_UNIX_BACKEND").is_err() {
            env::set_var("WINIT_UNIX_BACKEND", "x11");
        }
        // disables vsync sometimes on x11
        if env::var("vblank_mode").is_err() {
            env::set_var("vblank_mode", "0");
        }
    }

    if cfg!(debug_assertions) && env::var("yes_i_really_want_debug_mode").is_err() {
        eprintln!(
            "Note: Release mode will improve performance greatly.\n    \
             e.g. use `cargo run --example depth --release`"
        );
    }

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_transparent(true)
        .with_visible(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(256.0, 128.0))
        .with_x11_window_type(vec![XWindowType::Utility]);

    let (window_ctx, mut device, mut factory, mut main_color, mut main_depth) =
        glutin::ContextBuilder::new()
            .with_gfx_color_depth::<Srgba8, Depth>()
            .build_windowed(window, &event_loop)?
            .init_gfx::<Srgba8, Depth>();

    if let Some(value) = matches.value_of("position") {
        let v: Vec<&str> = value.split('x').collect();
        if v.len() != 2 {
            error!("Invalid argument [--pos]");
        }
        let x: i32 = v[0].parse()?;
        let y: i32 = v[1].parse()?;

        window_ctx
            .window()
            .set_outer_position(glutin::dpi::LogicalPosition::new(x, y));
    }

    let fonts = vec![FontArc::try_from_slice(include_bytes!(
        "Inconsolata-Regular.ttf"
    ))?];

    let mut glyph_brush = GlyphBrushBuilder::using_fonts(fonts)
        .initial_cache_size((512, 512))
        .build(factory.clone());

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    // Render loop
    window_ctx.window().request_redraw();

    let mut keys = KeyLogs::new();
    let mut last_frame_time = std::time::Instant::now();
    let mut size = window_ctx.window().inner_size();
    let mut cursor_state = CursorIcon::Default;

    #[allow(deprecated)]
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                window_ctx.resize(new_size);
                size = new_size;
                window_ctx.update_gfx(&mut main_color, &mut main_depth);
            }
            // Event::WindowEvent {
            //     event: winit::event::WindowEvent::CursorMoved { position: pos, .. },
            //     ..
            // } => {
            //     if cursor_state == CursorIcon::Move {
            //         window.set_outer_position(pos);
            //     }
            // }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                window_ctx.window().set_cursor_icon(CursorIcon::Move);
                cursor_state = CursorIcon::Move;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                window_ctx.window().set_cursor_icon(CursorIcon::Default);
                cursor_state = CursorIcon::Default;
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                if cursor_state == CursorIcon::Move {
                    let p = window_ctx.window().outer_position().unwrap();
                    window_ctx
                        .window()
                        .set_outer_position(glutin::dpi::PhysicalPosition::new(
                            p.x + delta.0 as i32,
                            p.y + delta.1 as i32,
                        ));
                }
            }
            Event::DeviceEvent {
                event:
                    DeviceEvent::Key(KeyboardInput {
                        virtual_keycode: Some(key),
                        state: ElementState::Pressed,
                        modifiers: modifiers_state,
                        ..
                    }),
                ..
            } => {
                info!("press {:?}", key);
                let conv_key = key_converter::convert(key, modifiers_state);
                if !conv_key.is_empty() {
                    keys.push(format!("{}", conv_key));
                    window_ctx.window().request_redraw();
                }
            }
            Event::MainEventsCleared => {}
            Event::RedrawRequested { .. } => {
                encoder.clear(&main_color, [0.0, 0.0, 0.0, 0.0]);
                encoder.clear_depth(&main_depth, 1.0);

                glyph_brush.queue(Section {
                    screen_position: (250.0, 120.0),
                    bounds: (size.width as f32, size.height as f32),
                    text: keys
                        .get_keys_from_last(4)
                        .iter()
                        .map(|x| {
                            vec![
                                Text::new(&x)
                                    .with_color([1.0, 1.0, 1.0, 0.5])
                                    .with_scale(30.0),
                                Text::new("\n")
                                    .with_color([1.0, 1.0, 1.0, 0.5])
                                    .with_scale(30.0),
                            ]
                        })
                        .flatten()
                        .collect(),
                    layout: Layout::default()
                        .h_align(HorizontalAlign::Right)
                        .v_align(VerticalAlign::Bottom),
                    ..Section::default()
                });

                glyph_brush
                    .use_queue()
                    // Enable depth testing with default less-equal drawing and update the depth buffer
                    .depth_target(&main_depth)
                    .draw(&mut encoder, &main_color)
                    .unwrap();

                encoder.flush(&mut device);
                window_ctx.swap_buffers().unwrap();
                device.cleanup();

                last_frame_time = std::time::Instant::now();
            }
            _ => (),
        }
        if std::time::Instant::now() - last_frame_time >= std::time::Duration::from_secs(1) {
            window_ctx.window().request_redraw();
        }
    });
}
