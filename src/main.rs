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

static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut app = App::new(&event_loop)?;
    let mut last_time = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            app.window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            let now = Instant::now();
            let dt = now.duration_since(last_time).as_secs_f32().min(0.1);
            last_time = now;

            app.update(dt);
            app.draw();
        }
        Event::WindowEvent { event, .. } => {
            app.handle_window_event(event, control_flow);
        }
        _ => {}
    });
}

struct Beam {
    x: u32,
    y: i32,
    remain_y: f32, // Accumulator for smooth vertical movement
}

struct Ship {
    x: u32,
    y: u32,
    speed: f32,
    remain_x: f32,
    remain_y: f32,
}

struct App {
    window: Window,
    pixels: Pixels,
    size: PhysicalSize<u32>,
    ship: Ship,
    pressed_keys: HashSet<VirtualKeyCode>,
    stars: Vec<Star>,
    rng: SimpleRng,
    // Sprites
    ship_pixels: Vec<u8>,
    ship_w: u32,
    ship_h: u32,
    beams: Vec<Beam>,
    beam_pixels: Vec<u8>,
    beam_w: u32,
    beam_h: u32,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title("Starship Command")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .build(event_loop)
            .unwrap();

        try_set_fullscreen(event_loop, &window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        // Load Ship
        let ship_img = image::load_from_memory(SHIP_PNG)
            .expect("Failed to load ship.png")
            .to_rgba8();
        let (ship_w, ship_h) = (ship_img.width(), ship_img.height());

        // Load Beam
        let beam_img = image::load_from_memory(BEAM_PNG)
            .expect("Failed to load beam.png")
            .to_rgba8();
        let (beam_w, beam_h) = (beam_img.width(), beam_img.height());

        let ship = Ship {
            x: size.width / 2,
            y: size.height - 150, // Start near the bottom
            speed: 500.0,
            remain_x: 0.0,
            remain_y: 0.0,
        };

        let mut rng = SimpleRng::seed_from_instant();
        let stars = generate_stars(&mut rng, size);

        Ok(Self {
            window,
            pixels,
            size,
            ship,
            pressed_keys: HashSet::new(),
            stars,
            rng,
            ship_pixels: ship_img.into_raw(),
            ship_w,
            ship_h,
            beams: Vec::new(),
            beam_pixels: beam_img.into_raw(),
            beam_w,
            beam_h,
        })
    }

    fn update(&mut self, dt: f32) {
        // 1. Update stars
        stars::update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        // 2. Update ship movement (Integer + Accumulator)
        let mut move_x = 0.0;
        let mut move_y = 0.0;
        let speed_this_frame = self.ship.speed * dt;

        if self.pressed_keys.contains(&VirtualKeyCode::Left) {
            move_x -= speed_this_frame;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Right) {
            move_x += speed_this_frame;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Up) {
            move_y -= speed_this_frame;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Down) {
            move_y += speed_this_frame;
        }

        self.ship.remain_x += move_x;
        self.ship.remain_y += move_y;

        let dx = self.ship.remain_x as i32;
        let dy = self.ship.remain_y as i32;

        self.ship.x = (self.ship.x as i32 + dx)
            .clamp(0, (self.size.width.saturating_sub(self.ship_w)) as i32)
            as u32;
        self.ship.y = (self.ship.y as i32 + dy)
            .clamp(0, (self.size.height.saturating_sub(self.ship_h)) as i32)
            as u32;

        self.ship.remain_x -= dx as f32;
        self.ship.remain_y -= dy as f32;

        // 3. Update beam movement
        let beam_speed = 900.0;
        for beam in self.beams.iter_mut() {
            beam.remain_y -= beam_speed * dt;
            let b_dy = beam.remain_y as i32;
            beam.y += b_dy;
            beam.remain_y -= b_dy as f32;
        }

        // 4. Cull beams that left the screen
        self.beams.retain(|b| b.y + (self.beam_h as i32) > 0);
    }

    fn draw(&mut self) {
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        // Layers: Stars -> Beams -> Ship
        for star in &self.stars {
            draw_star(frame, self.size.width, star);
        }

        for beam in &self.beams {
            draw_sprite(
                frame,
                self.size.width,
                beam.x as i32,
                beam.y,
                &self.beam_pixels,
                self.beam_w,
                self.beam_h,
            );
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

        let _ = self.pixels.render();
    }

    fn handle_window_event(&mut self, event: WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(key) = input.virtual_keycode {
                    match input.state {
                        ElementState::Pressed => {
                            // Check for single-press Space fire
                            if key == VirtualKeyCode::Space
                                && !self.pressed_keys.contains(&VirtualKeyCode::Space)
                            {
                                self.fire_beam();
                            }

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
            _ => {}
        }
    }

    fn fire_beam(&mut self) {
        // Center the beam horizontally based on the ship's position
        let spawn_x = self.ship.x + (self.ship_w / 2) - (self.beam_w / 2);
        let spawn_y = self.ship.y as i32 - (self.beam_h as i32);

        self.beams.push(Beam {
            x: spawn_x,
            y: spawn_y,
            remain_y: 0.0,
        });
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
