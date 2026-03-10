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
pub mod loaderboard;

use app::App;
use pixels::{Pixels, SurfaceTexture};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();

    // 1. Setup Monitor and Resolution
    let monitor = event_loop.primary_monitor().expect("Failed to find a monitor");
    let native_size = monitor.size();

    println!("Detected native resolution: {}x{}", native_size.width, native_size.height);

    let window = WindowBuilder::new()
        .with_title("Space Game")
        // Use the detected native size for the window
        .with_inner_size(native_size)
        // Enable Borderless Fullscreen on the detected monitor
        .with_fullscreen(Some(Fullscreen::Borderless(Some(monitor))))
        .with_visible(false) // Start hidden to avoid showing the cursor during load
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(true); // Show cursor during load

    let mut app: Option<App> = None;
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

            Event::MainEventsCleared => {
                // LAZY INITIALIZATION
                if app.is_none() {
                    println!("Initializing GPU (Pixels) and Loading Assets...");

                    // Create Pixels using the NATIVE size (game resolution)
                    let surface_texture = SurfaceTexture::new(native_size.width, native_size.height, &window);

                    // This is slow!
                    let pixels = Pixels::new(native_size.width, native_size.height, surface_texture).unwrap();

                    // Create the App (loads PNGs and SFX)
                    app = Some(App::new(pixels, native_size));

                    // TRANSITION TO GAME MODE
                    window.set_decorations(true);
                    window.set_cursor_visible(false);
                    window.set_visible(true); // Show the window after loading is complete

                    last_time = std::time::Instant::now();
                    println!("Load complete. Entering game loop.");
                }

                // NORMAL UPDATE
                if let Some(ref mut game) = app {
                    let now = std::time::Instant::now();
                    let dt = now.duration_since(last_time).as_secs_f32().min(0.05);
                    last_time = now;

                    game.update(dt);
                    window.request_redraw();
                }
            }

            Event::RedrawRequested(_) => {
                if let Some(ref mut game) = app {
                    game.draw();
                }
            }

            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                if let Some(ref mut game) = app {
                    let _ = game.pixels.resize_surface(new_size.width, new_size.height);
                }
            }

            Event::WindowEvent { event, .. } => {
                if let Some(ref mut game) = app {
                    game.input.update(&event);
                }
            }
            _ => {}
        }
    });
}
