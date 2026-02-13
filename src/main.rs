use pixels::{Error, Pixels, SurfaceTexture};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Full HD Graphics")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)
        .unwrap();

    // Try to switch the monitor to an exclusive Full HD video mode if available.
    if let Some(primary_monitor) = event_loop.primary_monitor() {
        // Find a matching video mode at Full HD.
        if let Some(vm) = primary_monitor
            .video_modes()
            .find(|vm| vm.size().width == WIDTH && vm.size().height == HEIGHT)
        {
            window.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            // Fall back to borderless fullscreen on the primary monitor.
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(primary_monitor))));
        }
    }

    // Use the actual window inner size for the pixel surface.
    let mut size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(size.width, size.height, surface_texture)?;

    // Square state (position). Start centered.
    let square_size: u32 = 20;
    let mut square_x = (size.width / 2).saturating_sub(square_size / 2);
    let mut square_y = (size.height / 2).saturating_sub(square_size / 2);

    // Track currently pressed keys for multi-key support
    let mut pressed_keys: HashSet<VirtualKeyCode> = HashSet::new();

    event_loop.run(move |event, _, control_flow| {
        // Schedule next wake depending on whether we need continuous updates
        if pressed_keys.is_empty() {
            *control_flow = ControlFlow::Wait;
        } else {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION);
        }

        match event {
            Event::RedrawRequested(_) => {
                // Apply movement based on keys held down
                let moved = apply_movement(&pressed_keys, &mut square_x, &mut square_y, 15u32, square_size, size);
                if moved {
                    // request a redraw so movement is continuous
                    window.request_redraw();
                }

                // `pixels.frame()` returns an immutable slice in this pixels version; create a
                // mutable view into the same memory so we can modify pixels.
                let frame_immutable = pixels.frame();
                let frame_len = frame_immutable.len();
                let frame = unsafe {
                    std::slice::from_raw_parts_mut(frame_immutable.as_ptr() as *mut u8, frame_len)
                };

                // Clear to black
                for pixel in frame.chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[0, 0, 0, 255]);
                }

                // Yellow pixels at corners
                set_pixel(frame, size.width, 0, 0, [255, 255, 0, 255]);
                set_pixel(frame, size.width, size.width - 1, 0, [255, 255, 0, 255]);
                set_pixel(frame, size.width, 0, size.height - 1, [255, 255, 0, 255]);
                set_pixel(frame, size.width, size.width - 1, size.height - 1, [255, 255, 0, 255]);

                // Red square at current position
                for y in square_y..(square_y + square_size) {
                    for x in square_x..(square_x + square_size) {
                        set_pixel(frame, size.width, x, y, [255, 0, 0, 255]);
                    }
                }

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    // Update pressed keys set; handle escape immediately
                    update_pressed_keys(&input, &mut pressed_keys);
                    if input.state == ElementState::Pressed {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    size = new_size;
                    pixels.resize_surface(size.width, size.height);
                    // Clamp square position to new bounds
                    square_x = square_x.min(size.width.saturating_sub(square_size));
                    square_y = square_y.min(size.height.saturating_sub(square_size));
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    size = *new_inner_size;
                    pixels.resize_surface(size.width, size.height);
                    square_x = square_x.min(size.width.saturating_sub(square_size));
                    square_y = square_y.min(size.height.saturating_sub(square_size));
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // Keep redrawing while keys are pressed for smooth movement
                if !pressed_keys.is_empty() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    });

    Ok(())
}

fn update_pressed_keys(input: &winit::event::KeyboardInput, pressed_keys: &mut HashSet<VirtualKeyCode>) {
    match (input.virtual_keycode, input.state) {
        (Some(key), ElementState::Pressed) => {
            pressed_keys.insert(key);
        }
        (Some(key), ElementState::Released) => {
            pressed_keys.remove(&key);
        }
        _ => {}
    }
}

/// Apply movement based on currently pressed keys. Returns true if position changed.
fn apply_movement(
    pressed_keys: &HashSet<VirtualKeyCode>,
    square_x: &mut u32,
    square_y: &mut u32,
    step: u32,
    square_size: u32,
    size: PhysicalSize<u32>,
) -> bool {
    let mut moved = false;

    if pressed_keys.contains(&VirtualKeyCode::Left) {
        *square_x = square_x.saturating_sub(step);
        moved = true;
    }
    if pressed_keys.contains(&VirtualKeyCode::Right) {
        *square_x = (*square_x + step).min(size.width.saturating_sub(square_size));
        moved = true;
    }
    if pressed_keys.contains(&VirtualKeyCode::Up) {
        *square_y = square_y.saturating_sub(step);
        moved = true;
    }
    if pressed_keys.contains(&VirtualKeyCode::Down) {
        *square_y = (*square_y + step).min(size.height.saturating_sub(square_size));
        moved = true;
    }

    moved
}

fn set_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, color: [u8; 4]) {
    // compute index using the provided frame width
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 < frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}
