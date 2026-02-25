pub mod app;
pub mod drawing;
pub mod entities;
pub mod input;
pub mod stars;
pub mod waves;

use app::App;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    // Resolution constants
    let width: u32 = 1920;
    let height: u32 = 1080;

    // Determine the monitor for Fullscreen
    let monitor = event_loop.primary_monitor();

    let window = WindowBuilder::new()
        .with_title("Rust Space Invaders")
        .with_inner_size(LogicalSize::new(width as f64, height as f64))
        // Enable Borderless Fullscreen
        .with_fullscreen(Some(Fullscreen::Borderless(monitor)))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    let size = PhysicalSize::new(width, height);
    let mut app = App::new(window, size);
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            // Close window or press Escape to quit
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input: winit::event::KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), .. },
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                let _ = app.pixels.resize_surface(new_size.width, new_size.height);
            }

            Event::MainEventsCleared => {
                let now = std::time::Instant::now();
                let dt = now.duration_since(last_time).as_secs_f32();
                last_time = now;

                app.update(dt);
                app.window.request_redraw();
            }

            Event::RedrawRequested(_) => {
                app.draw();
            }

            Event::WindowEvent { event, .. } => {
                app.input.update(&event);
            }
            _ => {}
        }
    });
}
