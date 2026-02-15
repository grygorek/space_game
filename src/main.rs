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
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");

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
    remain_y: f32,
}

struct Ship {
    x: u32,
    y: u32,
    speed: f32,
    remain_x: f32,
    remain_y: f32,
}

struct Enemy {
    x: u32,
    y: u32,
    active: bool,
}

struct App {
    window: Window,
    pixels: Pixels,
    size: PhysicalSize<u32>,
    ship: Ship,
    enemies: Vec<Enemy>,
    pressed_keys: HashSet<VirtualKeyCode>,
    stars: Vec<Star>,
    rng: SimpleRng,
    // Sprites
    ship_pixels: Vec<u8>,
    ship_w: u32,
    ship_h: u32,
    enemy_pixels: Vec<u8>,
    enemy_w: u32,
    enemy_h: u32,
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

        window.set_cursor_visible(false);
        try_set_fullscreen(event_loop, &window);

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        // Load Sprites
        let ship_img = image::load_from_memory(SHIP_PNG)
            .expect("ship.png fail")
            .to_rgba8();
        let beam_img = image::load_from_memory(BEAM_PNG)
            .expect("beam.png fail")
            .to_rgba8();
        let enemy_img = image::load_from_memory(ENEMY1_PNG)
            .expect("enemy1.png fail")
            .to_rgba8();

        let (ship_w, ship_h) = (ship_img.width(), ship_img.height());
        let (beam_w, beam_h) = (beam_img.width(), beam_img.height());
        let (enemy_w, enemy_h) = (enemy_img.width(), enemy_img.height());

        // Player Ship: 1/5th from bottom
        let ship = Ship {
            x: (size.width / 2).saturating_sub(ship_w / 2),
            y: size.height - (size.height / 5),
            speed: 600.0,
            remain_x: 0.0,
            remain_y: 0.0,
        };

        // Enemy Grid: 3 rows, 8 columns
        // Use usize for counts to satisfy Vec::with_capacity and iterators
        let rows: usize = 3;
        let cols: usize = 8;
        let spacing_x = enemy_w / 2;
        let spacing_y = enemy_h / 2;

        let grid_width = (cols as u32 * enemy_w) + ((cols as u32 - 1) * spacing_x);
        let start_x = (size.width.saturating_sub(grid_width)) / 2;
        let start_y = size.height / 3;

        let mut enemies = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                enemies.push(Enemy {
                    x: start_x + (c as u32 * (enemy_w + spacing_x)),
                    y: start_y + (r as u32 * (enemy_h + spacing_y)),
                    active: true,
                });
            }
        }

        let mut rng = SimpleRng::seed_from_instant();
        let stars = generate_stars(&mut rng, size);

        Ok(Self {
            window,
            pixels,
            size,
            ship,
            enemies,
            pressed_keys: HashSet::new(),
            stars,
            rng,
            ship_pixels: ship_img.into_raw(),
            ship_w,
            ship_h,
            enemy_pixels: enemy_img.into_raw(),
            enemy_w,
            enemy_h,
            beams: Vec::new(),
            beam_pixels: beam_img.into_raw(),
            beam_w,
            beam_h,
        })
    }

    fn update(&mut self, dt: f32) {
        stars::update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        // Ship movement
        let mut move_x = 0.0;
        let mut move_y = 0.0;
        let speed = self.ship.speed * dt;
        if self.pressed_keys.contains(&VirtualKeyCode::Left) {
            move_x -= speed;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Right) {
            move_x += speed;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Up) {
            move_y -= speed;
        }
        if self.pressed_keys.contains(&VirtualKeyCode::Down) {
            move_y += speed;
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

        // Beams movement
        let beam_speed = 1000.0;
        for beam in self.beams.iter_mut() {
            beam.remain_y -= beam_speed * dt;
            let b_dy = beam.remain_y as i32;
            beam.y += b_dy;
            beam.remain_y -= b_dy as f32;
        }
        self.beams.retain(|b| b.y + (self.beam_h as i32) > 0);
    }

    fn draw(&mut self) {
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        // 1. Background
        for star in &self.stars {
            draw_star(frame, self.size.width, star);
        }

        // 2. Enemies
        for enemy in &self.enemies {
            if enemy.active {
                draw_sprite(
                    frame,
                    self.size.width,
                    enemy.x as i32,
                    enemy.y as i32,
                    &self.enemy_pixels,
                    self.enemy_w,
                    self.enemy_h,
                );
            }
        }

        // 3. Projectiles
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

        // 4. Player
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
