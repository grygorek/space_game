mod drawing;
mod stars;

use drawing::draw_sprite;
use pixels::{Error, Pixels, SurfaceTexture};
use stars::{draw_star, generate_stars, SimpleRng, Star};
use std::collections::HashSet;
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowBuilder},
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

// Embed ship image bytes (png/ship.png)
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut app = App::new(&event_loop)?;

    let mut last_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            // Only calculate physics right before we draw
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = now.duration_since(last_time).as_secs_f32();
                last_time = now;

                // Cap dt to prevent "teleporting" if the window is dragged
                let dt = dt.min(0.1);

                app.update(dt);
                app.draw();
            }
            Event::MainEventsCleared => {
                app.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.handle_window_event(event, control_flow);
            }
            _ => {}
        }
    });
}

struct Ship {
    x: f32,
    y: f32,
    speed: f32,
}

struct App {
    window: Window,
    pixels: Pixels,
    size: PhysicalSize<u32>,
    ship: Ship,
    pressed_keys: HashSet<VirtualKeyCode>,
    stars: Vec<Star>,
    rng: SimpleRng,
    ship_pixels: Vec<u8>,
    ship_w: u32,
    ship_h: u32,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title("Full HD Starship")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .build(event_loop)
            .unwrap();

        try_set_fullscreen(event_loop, &window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        let ship = Ship {
            x: (size.width / 2) as f32,
            y: (size.height / 2) as f32,
            speed: 300.0,
        };

        let mut rng = SimpleRng::seed_from_instant();
        let stars = generate_stars(&mut rng, size);

        let img = image::load_from_memory(SHIP_PNG)
            .expect("Failed to load ship.png")
            .to_rgba8();
        let ship_w = img.width();
        let ship_h = img.height();
        let ship_pixels = img.into_raw();

        Ok(Self {
            window,
            pixels,
            size,
            ship,
            pressed_keys: HashSet::new(),
            stars,
            rng,
            ship_pixels,
            ship_w,
            ship_h,
        })
    }

    fn update(&mut self, dt: f32) {
        stars::update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        let move_amount = self.ship.speed * dt;

        if self.pressed_keys.contains(&VirtualKeyCode::Left) {
            self.ship.x -= move_amount;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Right) {
            self.ship.x += move_amount;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Up) {
            self.ship.y -= move_amount;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Down) {
            self.ship.y += move_amount;
        }

        // Clamp using current window and ship dimensions
        self.ship.x = self
            .ship
            .x
            .clamp(0.0, (self.size.width.saturating_sub(self.ship_w)) as f32);
        self.ship.y = self
            .ship
            .y
            .clamp(0.0, (self.size.height.saturating_sub(self.ship_h)) as f32);
    }

    fn draw(&mut self) {
        let frame = self.pixels.frame_mut();

        // Optimized screen clear
        frame.fill(0);

        for star in &self.stars {
            draw_star(frame, self.size.width, star);
        }

        draw_sprite(
            frame,
            self.size.width,
            self.ship.x as i32,
            self.ship.y as i32,
            &self.ship_pixels,
            self.ship_w,
            self.ship_h,
        );

        if let Err(err) = self.pixels.render() {
            eprintln!("Pixels render error: {}", err);
        }
    }

    fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                            if key == VirtualKeyCode::Escape {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            WindowEvent::Resized(new_size) => {
                self.size = new_size;
                let _ = self
                    .pixels
                    .resize_surface(self.size.width, self.size.height);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.size = *new_inner_size;
                let _ = self
                    .pixels
                    .resize_surface(self.size.width, self.size.height);
            }
            _ => {}
        }
    }
}

fn try_set_fullscreen(event_loop: &EventLoop<()>, window: &Window) {
    if let Some(primary_monitor) = event_loop.primary_monitor() {
        let video_mode = primary_monitor
            .video_modes()
            .find(|vm| vm.size().width == WIDTH && vm.size().height == HEIGHT);

        match video_mode {
            Some(vm) => window.set_fullscreen(Some(Fullscreen::Exclusive(vm))),
            None => window.set_fullscreen(Some(Fullscreen::Borderless(Some(primary_monitor)))),
        }
    }
}
