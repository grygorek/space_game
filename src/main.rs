mod stars;

use pixels::{Error, Pixels, SurfaceTexture};
use std::collections::HashSet;
use std::time::{Duration, Instant};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};
use stars::{generate_stars, draw_star, SimpleRng, Star};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FRAME_DURATION: Duration = Duration::from_millis(16); // ~60 FPS

// Embed ship image bytes (png/ship.png)
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");

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
    stars: Vec<Star>,
    rng: SimpleRng,

    // Ship sprite
    ship_pixels: Vec<u8>, // RGBA8 raw pixels
    ship_w: u32,
    ship_h: u32,
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

        // Generate stars
        let mut rng = SimpleRng::seed_from_instant();
        let stars = generate_stars(&mut rng, size);

        // Decode ship PNG
        let img = image::load_from_memory(SHIP_PNG).expect("Failed to load ship.png").to_rgba8();
        let ship_w = img.width();
        let ship_h = img.height();
        let ship_pixels = img.into_raw();

        Ok(Self {
            window,
            pixels,
            size,
            square,
            pressed_keys: HashSet::new(),
            stars,
            rng,
            ship_pixels,
            ship_w,
            ship_h,
        })
    }

    fn handle_event(&mut self, event: Event<()>, control_flow: &mut ControlFlow) {
        // Drive continuous animation at FRAME_DURATION even when no keys are pressed
        *control_flow = ControlFlow::WaitUntil(Instant::now() + FRAME_DURATION);

        match event {
            Event::RedrawRequested(_) => {
                self.on_redraw();
            }
            Event::WindowEvent { event, .. } => {
                self.on_window_event(event, control_flow);
            }
            Event::MainEventsCleared => {
                // Always request redraw to keep stars animating
                self.window.request_redraw();
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self) {
        // Update stars (falling)
        stars::update_stars(&mut self.stars, &mut self.rng, self.size);

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

        // Clear to black (interstellar space)
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 255]);
        }

        // Draw stars
        for star in &self.stars {
            draw_star(frame, self.size.width, star);
        }

        // Draw ship sprite centered on square
        let sx = self.square.x as i32;
        let sy = self.square.y as i32;
        for yy in 0..(self.ship_h as i32) {
            for xx in 0..(self.ship_w as i32) {
                let px = sx + xx;
                let py = sy + yy;
                if px < 0 || py < 0 {
                    continue;
                }
                let px = px as u32;
                let py = py as u32;
                if px >= self.size.width || py >= self.size.height {
                    continue;
                }
                let idx = ((yy as u32 * self.ship_w + xx as u32) * 4) as usize;
                let src = [
                    self.ship_pixels[idx],
                    self.ship_pixels[idx + 1],
                    self.ship_pixels[idx + 2],
                    self.ship_pixels[idx + 3],
                ];
                // blend over background
                stars::blend_pixel(frame, self.size.width, px, py, src);
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
                let _ = self.pixels.resize_surface(self.size.width, self.size.height);
                self.square.x = self.square.x.min(self.size.width.saturating_sub(self.square.size));
                self.square.y = self.square.y.min(self.size.height.saturating_sub(self.square.size));
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.size = *new_inner_size;
                let _ = self.pixels.resize_surface(self.size.width, self.size.height);
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
