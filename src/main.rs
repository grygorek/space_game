use pixels::{Error, Pixels, SurfaceTexture};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let mut app = App::new(&event_loop)?;

    event_loop.run(move |event, _, control_flow| {
        // Delegate all event handling to the application instance.
        app.handle_event(event, control_flow);
    });
}

struct Square {
    x: u32,
    y: u32,
    size: u32,
}

struct App {
    window: Window,
    pixels: Pixels,
    size: PhysicalSize<u32>,
    square: Square,
    pressed_keys: HashSet<VirtualKeyCode>,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title("Full HD Graphics")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .build(event_loop)
            .unwrap();

        // Try to set fullscreen modes
        try_set_fullscreen(event_loop, &window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        let square_size = 20u32;
        let square = Square {
            x: (size.width / 2).saturating_sub(square_size / 2),
            y: (size.height / 2).saturating_sub(square_size / 2),
            size: square_size,
        };

        Ok(Self {
            window,
            pixels,
            size,
            square,
            pressed_keys: HashSet::new(),
        })
    }

    fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        // Determine desired wake behavior based on pressed keys
        if self.pressed_keys.is_empty() {
            *control_flow = ControlFlow::Wait;
        } else {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION);
        }

        match event {
            Event::RedrawRequested(_) => {
                self.on_redraw();
            }
            Event::WindowEvent { event, .. } => {
                self.on_window_event(event, control_flow);
            }
            Event::MainEventsCleared => {
                if !self.pressed_keys.is_empty() {
                    self.window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self) {
        // Apply movement based on keys held down
        let moved = self.apply_movement(15u32);
        if moved {
            // request another redraw for continuous movement
            self.window.request_redraw();
        }

        // Acquire a mutable frame buffer
        let frame_immutable = self.pixels.frame();
        let frame_len = frame_immutable.len();
        let frame = unsafe {
            std::slice::from_raw_parts_mut(frame_immutable.as_ptr() as *mut u8, frame_len)
        };

        // Clear to black
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }

        // Draw corner yellow pixels
        set_pixel(frame, self.size.width, 0, 0, [255, 255, 0, 255]);
        set_pixel(frame, self.size.width, self.size.width - 1, 0, [255, 255, 0, 255]);
        set_pixel(frame, self.size.width, 0, self.size.height - 1, [255, 255, 0, 255]);
        set_pixel(frame, self.size.width, self.size.width - 1, self.size.height - 1, [255, 255, 0, 255]);

        // Draw square
        for y in self.square.y..(self.square.y + self.square.size) {
            for x in self.square.x..(self.square.x + self.square.size) {
                set_pixel(frame, self.size.width, x, y, [255, 0, 0, 255]);
            }
        }

        if self.pixels.render().is_err() {
            // On render failure, just exit the event loop on next opportunity
            // (can't return an error from this callback).
        }
    }

    fn on_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                self.update_pressed_keys(&input);
                if input.state == ElementState::Pressed {
                    if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                self.size = new_size;
                self.pixels.resize_surface(self.size.width, self.size.height);
                self.square.x = self.square.x.min(self.size.width.saturating_sub(self.square.size));
                self.square.y = self.square.y.min(self.size.height.saturating_sub(self.square.size));
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.size = *new_inner_size;
                self.pixels.resize_surface(self.size.width, self.size.height);
                self.square.x = self.square.x.min(self.size.width.saturating_sub(self.square.size));
                self.square.y = self.square.y.min(self.size.height.saturating_sub(self.square.size));
            }
            _ => {}
        }
    }

    fn update_pressed_keys(&mut self, input: &winit::event::KeyboardInput) {
        match (input.virtual_keycode, input.state) {
            (Some(key), ElementState::Pressed) => {
                self.pressed_keys.insert(key);
            }
            (Some(key), ElementState::Released) => {
                self.pressed_keys.remove(&key);
            }
            _ => {}
        }
    }

    fn apply_movement(&mut self, step: u32) -> bool {
        let mut moved = false;
        if self.pressed_keys.contains(&VirtualKeyCode::Left) {
            self.square.x = self.square.x.saturating_sub(step);
            moved = true;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Right) {
            self.square.x = (self.square.x + step).min(self.size.width.saturating_sub(self.square.size));
            moved = true;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Up) {
            self.square.y = self.square.y.saturating_sub(step);
            moved = true;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Down) {
            self.square.y = (self.square.y + step).min(self.size.height.saturating_sub(self.square.size));
            moved = true;
        }
        moved
    }
}

fn try_set_fullscreen(event_loop: &EventLoop<()>, window: &Window) {
    if let Some(primary_monitor) = event_loop.primary_monitor() {
        if let Some(vm) = primary_monitor
            .video_modes()
            .find(|vm| vm.size().width == WIDTH && vm.size().height == HEIGHT)
        {
            window.set_fullscreen(Some(Fullscreen::Exclusive(vm)));
        } else {
            window.set_fullscreen(Some(Fullscreen::Borderless(Some(primary_monitor))));
        }
    }
}

fn set_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 < frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}
