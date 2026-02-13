use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

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
    let size = window.inner_size();
    let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
    let mut pixels = Pixels::new(size.width, size.height, surface_texture)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
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
                set_pixel(frame, 0, 0, [255, 255, 0, 255]);
                set_pixel(frame, size.width - 1, 0, [255, 255, 0, 255]);
                set_pixel(frame, 0, size.height - 1, [255, 255, 0, 255]);
                set_pixel(frame, size.width - 1, size.height - 1, [255, 255, 0, 255]);

                // Red 20x20 square in center
                let square_size = 20u32;
                let start_x = (size.width / 2).saturating_sub(square_size / 2);
                let start_y = (size.height / 2).saturating_sub(square_size / 2);
                for y in start_y..start_y + square_size {
                    for x in start_x..start_x + square_size {
                        set_pixel(frame, x, y, [255, 0, 0, 255]);
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
                    if input.state == ElementState::Pressed {
                        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    let size = window.inner_size();
                    pixels.resize_surface(size.width, size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });

    Ok(())
}

fn set_pixel(frame: &mut [u8], x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * WIDTH + x) * 4) as usize;
    if idx + 3 < frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}
