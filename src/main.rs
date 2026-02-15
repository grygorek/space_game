mod app;
mod drawing;
mod entities;
mod input;
mod stars;

use crate::app::App;
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Starship Command")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);
    try_set_fullscreen(&event_loop, &window);

    let mut app = App::new(window, PhysicalSize::new(WIDTH, HEIGHT));
    let mut last_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => {
            app.input.update(&event);
            match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::Resized(new_size) => {
                    app.size = new_size;
                    app.pixels
                        .resize_surface(new_size.width, new_size.height)
                        .unwrap();
                }
                _ => {}
            }
        }
        Event::MainEventsCleared => app.window.request_redraw(),
        Event::RedrawRequested(_) => {
            let dt = last_time.elapsed().as_secs_f32().min(0.1);
            last_time = Instant::now();
            app.update(dt);
            app.draw();
        }
        _ => {}
    });
}

fn try_set_fullscreen(event_loop: &EventLoop<()>, window: &winit::window::Window) {
    if let Some(mon) = event_loop.primary_monitor() {
        let mode = mon
            .video_modes()
            .find(|m| m.size().width == WIDTH && m.size().height == HEIGHT);
        window.set_fullscreen(
            mode.map(|m| Fullscreen::Exclusive(m))
                .or(Some(Fullscreen::Borderless(Some(mon)))),
        );
    }
}
