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
const MAX_STARS: usize = 20;

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

struct Star {
    x: u32,
    y: u32,
    diameter: u32,
    color: [u8; 4], // RGBA
}

struct App {
    window: Window,
    pixels: Pixels,
    size: PhysicalSize<u32>,
    square: Square,
    pressed_keys: HashSet<VirtualKeyCode>,
    stars: Vec<Star>,
    rng: SimpleRng,
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

        Ok(Self {
            window,
            pixels,
            size,
            square,
            pressed_keys: HashSet::new(),
            stars,
            rng,
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
        self.update_stars();

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

    fn update_stars(&mut self) {
        for star in &mut self.stars {
            // speed depends on diameter: larger stars fall slightly faster
            let speed = (star.diameter / 2).max(1);
            star.y = star.y.saturating_add(speed);
            if star.y > self.size.height {
                // reset to top with new random properties
                star.diameter = (self.rng.next_u32() % 5) + 1;
                let radius = star.diameter / 2;
                // avoid underflow if diameter > width
                if self.size.width > star.diameter {
                    star.x = (self.rng.next_u32() % (self.size.width.saturating_sub(star.diameter) + 1)) + radius;
                } else {
                    star.x = 0;
                }
                star.y = 0;
                let choice = self.rng.next_u32() % 4;
                let alpha = 100 + (self.rng.next_u32() % 156) as u8; // 100..255
                star.color = match choice {
                    0 => [150, 180, 255, alpha], // bluish
                    1 => [200, 200, 220, alpha], // grayish
                    2 => [255, 230, 160, alpha], // yellowish
                    3 => [255, 150, 150, alpha], // reddish
                    _ => [200, 200, 200, alpha],
                };
            }
        }
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

fn generate_stars(rng: &mut SimpleRng, size: PhysicalSize<u32>) -> Vec<Star> {
    let count = (rng.next_u32() as usize % (MAX_STARS + 1)).max(5); // between 5 and MAX_STARS
    let mut stars = Vec::with_capacity(count);
    for _ in 0..count {
        let diameter = (rng.next_u32() % 5) + 1; // 1..5
        let radius = diameter / 2;
        let x = (rng.next_u32() % (size.width.saturating_sub(diameter) + 1)) + radius;
        let y = (rng.next_u32() % (size.height.saturating_sub(diameter) + 1)) + radius;

        // Color palettes: bluish, gray, yellowish, reddish
        let choice = rng.next_u32() % 4;
        let alpha = 100 + (rng.next_u32() % 156) as u8; // 100..255
        let color = match choice {
            0 => [150, 180, 255, alpha], // bluish
            1 => [200, 200, 220, alpha], // grayish
            2 => [255, 230, 160, alpha], // yellowish
            3 => [255, 150, 150, alpha], // reddish
            _ => [200, 200, 200, alpha],
        };

        stars.push(Star { x, y, diameter, color });
    }
    stars
}

fn draw_star(frame: &mut [u8], frame_width: u32, star: &Star) {
    let r = (star.diameter as i32) / 2;
    let cx = star.x as i32;
    let cy = star.y as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let sx = cx + dx;
            let sy = cy + dy;
            if sx < 0 || sy < 0 {
                continue;
            }
            // circle mask
            if dx * dx + dy * dy <= r * r {
                set_pixel(frame, frame_width, sx as u32, sy as u32, star.color);
            }
        }
    }
}

// Simple xorshift RNG to avoid adding dependencies
struct SimpleRng(u64);
impl SimpleRng {
    fn seed_from_instant() -> Self {
        // Use system time since UNIX_EPOCH to create a seed
        let seed = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_nanos() as u64,
            Err(_) => 0u64,
        };
        SimpleRng(seed.wrapping_add(0x9E3779B97F4A7C15))
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() & 0xFFFF_FFFF) as u32
    }
}

fn set_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 < frame.len() {
        // simple overwrite; alpha channel is stored but we don't blend
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}
