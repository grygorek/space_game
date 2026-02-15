use crate::drawing::draw_sprite;
use crate::entities::{Beam, Enemy, Particle, Ship};
use crate::input::InputState;
use crate::stars::{draw_star, generate_stars, update_stars, SimpleRng, Star};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;
use winit::window::Window;

static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");

pub struct App {
    pub window: Window,
    pub pixels: Pixels,
    pub size: PhysicalSize<u32>,
    pub input: InputState,
    ship: Ship,
    enemies: Vec<Enemy>,
    stars: Vec<Star>,
    particles: Vec<Particle>, // New particle container
    rng: SimpleRng,
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
    pub fn new(window: Window, size: PhysicalSize<u32>) -> Self {
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();

        let ship_img = image::load_from_memory(SHIP_PNG).unwrap().to_rgba8();
        let (ship_w, ship_h) = ship_img.dimensions();

        let enemy_img = image::load_from_memory(ENEMY1_PNG).unwrap().to_rgba8();
        let (enemy_w, enemy_h) = enemy_img.dimensions();

        let beam_img = image::load_from_memory(BEAM_PNG).unwrap().to_rgba8();
        let (beam_w, beam_h) = beam_img.dimensions();

        let mut rng = SimpleRng::seed_from_instant();
        let stars = generate_stars(&mut rng, size);

        let ship = Ship {
            x: (size.width / 2) - (ship_w / 2),
            y: size.height - (size.height / 5),
            speed: 600.0,
            remain_x: 0.0,
            remain_y: 0.0,
        };

        let rows = 3;
        let cols = 8;
        let spacing_x = enemy_w / 2;
        let spacing_y = enemy_h / 2;
        let grid_width = (cols as u32 * enemy_w) + ((cols as u32 - 1) * spacing_x);
        let start_x = (size.width.saturating_sub(grid_width)) / 2;

        let mut enemies = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for c in 0..cols {
                enemies.push(Enemy {
                    x: start_x + (c as u32 * (enemy_w + spacing_x)),
                    y: (size.height / 3) + (r as u32 * (enemy_h + spacing_y)),
                    active: true,
                });
            }
        }

        Self {
            window,
            pixels,
            size,
            input: InputState::new(),
            ship,
            enemies,
            stars,
            particles: Vec::new(),
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
        }
    }

    pub fn update(&mut self, dt: f32) {
        update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        if self.input.was_key_pressed(VirtualKeyCode::Space) {
            self.fire_beam();
        }

        // Ship movement
        let mut mx = 0.0;
        let mut my = 0.0;
        if self.input.is_key_down(VirtualKeyCode::Left) {
            mx -= self.ship.speed * dt;
        }
        if self.input.is_key_down(VirtualKeyCode::Right) {
            mx += self.ship.speed * dt;
        }
        if self.input.is_key_down(VirtualKeyCode::Up) {
            my -= self.ship.speed * dt;
        }
        if self.input.is_key_down(VirtualKeyCode::Down) {
            my += self.ship.speed * dt;
        }

        self.ship.remain_x += mx;
        self.ship.remain_y += my;
        let dx = self.ship.remain_x as i32;
        let dy = self.ship.remain_y as i32;

        self.ship.x =
            (self.ship.x as i32 + dx).clamp(0, (self.size.width - self.ship_w) as i32) as u32;
        self.ship.y =
            (self.ship.y as i32 + dy).clamp(0, (self.size.height - self.ship_h) as i32) as u32;
        self.ship.remain_x -= dx as f32;
        self.ship.remain_y -= dy as f32;

        // Particles physics
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);

        // Beam movement
        for beam in self.beams.iter_mut() {
            beam.remain_y -= 1000.0 * dt;
            let bdy = beam.remain_y as i32;
            beam.y += bdy;
            beam.remain_y -= bdy as f32;
        }

        self.check_collisions();
        self.beams.retain(|b| b.y + (self.beam_h as i32) > 0);
        self.input.clear_just_pressed();
    }

    pub fn draw(&mut self) {
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        for s in &self.stars {
            draw_star(frame, self.size.width, s);
        }

        for e in &self.enemies {
            if e.active {
                draw_sprite(
                    frame,
                    self.size.width,
                    e.x as i32,
                    e.y as i32,
                    &self.enemy_pixels,
                    self.enemy_w,
                    self.enemy_h,
                );
            }
        }

        // Draw individual pixel particles
        for p in &self.particles {
            if p.x >= 0.0
                && p.x < self.size.width as f32
                && p.y >= 0.0
                && p.y < self.size.height as f32
            {
                let idx = (p.y as u32 * self.size.width + p.x as u32) as usize * 4;
                if idx + 3 < frame.len() {
                    // Flash yellow/orange based on remaining life
                    frame[idx] = 255;
                    frame[idx + 1] = (150.0 + (p.life * 200.0)).min(255.0) as u8;
                    frame[idx + 2] = 50;
                    frame[idx + 3] = 255;
                }
            }
        }

        for b in &self.beams {
            draw_sprite(
                frame,
                self.size.width,
                b.x as i32,
                b.y,
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

        self.pixels.render().unwrap();
    }

    fn fire_beam(&mut self) {
        self.beams.push(Beam {
            x: self.ship.x + (self.ship_w / 2) - (self.beam_w / 2),
            y: self.ship.y as i32 - self.beam_h as i32,
            remain_y: 0.0,
        });
    }

    fn spawn_explosion(&mut self, x: u32, y: u32) {
        let count = 30;
        for _ in 0..count {
            // Random circle distribution
            let angle = (self.rng.next_u32() % 360) as f32 * (std::f32::consts::PI / 180.0);
            let speed = (self.rng.next_u32() % 200) as f32 + 50.0;

            self.particles.push(Particle {
                x: x as f32 + (self.enemy_w / 2) as f32,
                y: y as f32 + (self.enemy_h / 2) as f32,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.4 + (self.rng.next_u32() % 300) as f32 / 1000.0,
            });
        }
    }

    fn check_collisions(&mut self) {
        // 1. Create a temporary list to store where explosions should happen
        let mut explosions_to_spawn = Vec::new();

        for beam in self.beams.iter_mut() {
            if beam.y < 0 {
                continue;
            }

            for enemy in self.enemies.iter_mut() {
                if !enemy.active {
                    continue;
                }

                let beam_hit = beam.x < enemy.x + self.enemy_w
                    && beam.x + self.beam_w > enemy.x
                    && beam.y < (enemy.y + self.enemy_h) as i32
                    && beam.y + self.beam_h as i32 > enemy.y as i32;

                if beam_hit {
                    enemy.active = false;
                    beam.y = -1000;

                    // 2. Record the location instead of calling self.spawn_explosion
                    explosions_to_spawn.push((enemy.x, enemy.y));

                    break;
                }
            }
        }

        // 3. Now that the mutable borrow of self.beams is over, we can safely use self again
        for (x, y) in explosions_to_spawn {
            self.spawn_explosion(x, y);
        }
    }
}
