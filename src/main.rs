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

struct Ship {
    x: u32,
    y: u32,
    speed: f32,
    remain_x: f32, // Fractional movement storage
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
    ship_pixels: Vec<u8>,
    ship_w: u32,
    ship_h: u32,
}

impl App {
    fn new(event_loop: &EventLoop<()>) -> Result<Self, Error> {
        let window = WindowBuilder::new()
            .with_title("Fixed Pixel Starship")
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .build(event_loop)
            .unwrap();

        try_set_fullscreen(event_loop, &window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        let img = image::load_from_memory(SHIP_PNG)
            .expect("Failed to load ship.png")
            .to_rgba8();
        let ship_w = img.width();
        let ship_h = img.height();

        let ship = Ship {
            x: size.width / 2,
            y: size.height / 2,
            speed: 400.0,
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
            ship_w,
            ship_h,
            ship_pixels: img.into_raw(),
        })
    }

    fn update(&mut self, dt: f32) {
        stars::update_stars(&mut self.stars, &mut self.rng, self.size, dt);

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

        // 1. Add fractional movement to accumulator
        self.ship.remain_x += move_x;
        self.ship.remain_y += move_y;

        // 2. Determine whole pixel steps
        let dx = self.ship.remain_x as i32;
        let dy = self.ship.remain_y as i32;

        // 3. Update integer position with bounds checking
        let new_x = (self.ship.x as i32 + dx)
            .clamp(0, (self.size.width.saturating_sub(self.ship_w)) as i32);
        let new_y = (self.ship.y as i32 + dy)
            .clamp(0, (self.size.height.saturating_sub(self.ship_h)) as i32);

        self.ship.x = new_x as u32;
        self.ship.y = new_y as u32;

        // 4. Subtract used pixels from accumulator
        self.ship.remain_x -= dx as f32;
        self.ship.remain_y -= dy as f32;
    }

    fn draw(&mut self) {
        let frame = self.pixels.frame_mut();
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

        let _ = self.pixels.render();
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
