// Copyright 2026 Piotr Grygorczuk <grygorek@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

pub mod app;
pub mod drawing;
pub mod entities;
pub mod input;
pub mod rng;
pub mod stars;
pub mod waves;

use app::App;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    // 1. Get the primary monitor
    let monitor = event_loop.primary_monitor().expect("Failed to find a monitor");

    // 2. Detect the native resolution
    // .size() returns PhysicalSize<u32>, which is exactly what we need for the game state
    let native_size = monitor.size();
    let width = native_size.width;
    let height = native_size.height;

    println!("Detected native resolution: {}x{}", width, height);

    let window = WindowBuilder::new()
        .with_title("Rust Space Invaders")
        // Use the detected native size for the window
        .with_inner_size(native_size)
        // Enable Borderless Fullscreen on the detected monitor
        .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    // 3. Initialize your App with the dynamic size instead of hardcoded 1920x1080
    let mut app = App::new(window, native_size);
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
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
                // Keep the pixel buffer in sync with the window size
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
