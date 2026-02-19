use crate::drawing::draw_sprite;
use crate::entities::{
    beam::Beam, enemy::Enemy, particle::Particle, ship::Ship, Collidable, Sprite,
};
use crate::wave::Wave;

use crate::input::InputState;
use crate::stars::{draw_star, generate_stars, update_stars, SimpleRng, Star};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;
use winit::window::Window;

// Asset Constants
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");

pub struct App {
    pub window: Window,
    pub pixels: Pixels,
    pub size: PhysicalSize<u32>,
    pub input: InputState,

    // Entities
    ship: Ship,
    enemies: Vec<Enemy>,
    wave: Wave,
    beams: Vec<Beam>,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    rng: SimpleRng,
    sprites: Vec<Sprite>,
}

impl App {
    pub fn new(window: Window, size: PhysicalSize<u32>) -> Self {
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();
        let mut rng = SimpleRng::seed_from_instant();

        // Load Sprites
        let mut sprites = Vec::new();
        for data in [SHIP_PNG, BEAM_PNG, ENEMY1_PNG] {
            let img = image::load_from_memory(data).unwrap().to_rgba8();
            sprites.push(Sprite {
                width: img.width(),
                height: img.height(),
                pixels: img.into_raw(),
            });
        }

        // Initialize Ship
        let ship = Ship {
            x: (size.width / 2) - (sprites[0].width / 2),
            y: size.height - (size.height / 5),
            speed: 600.0,
            remain_x: 0.0,
            remain_y: 0.0,
            sprite_idx: 0,
        };

        let mut wave = Wave::new();
        let stars = generate_stars(&mut rng, size);

        Self {
            window,
            pixels,
            size,
            rng,
            input: InputState::new(),
            ship,
            enemies: wave.deploy(size.width, size.height, &sprites[2]),
            wave,
            sprites,
            beams: Vec::new(),
            particles: Vec::new(),
            stars,
        }
    }

    pub fn update(&mut self, dt: f32) {
        update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        let s_img = &self.sprites[self.ship.sprite_idx];
        self.ship
            .update(&self.input, self.size, s_img.width, s_img.height, dt);

        if self.input.was_key_pressed(VirtualKeyCode::Space) {
            self.fire_beam();
        }

        self.update_beams(dt);
        self.update_particles(dt);
        self.process_collisions();

        // When the current squad is extinct, deploy the next wave
        if self.wave.is_extinct(&self.enemies) {
            self.enemies = self
                .wave
                .deploy(self.size.width, self.size.height, &self.sprites[2]);
        }

        self.input.clear_just_pressed();
    }

    fn update_beams(&mut self, dt: f32) {
        let b_h = self.sprites[1].height;
        for beam in self.beams.iter_mut() {
            beam.remain_y -= 1000.0 * dt;
            let dy = beam.remain_y as i32;
            beam.y += dy;
            beam.remain_y -= dy as f32;
        }
        self.beams.retain(|b| b.y + (b_h as i32) > 0);
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn process_collisions(&mut self) {
        let mut explosions = Vec::new();
        let b_s = &self.sprites[1];
        let e_s = &self.sprites[2];

        for beam in self.beams.iter_mut().filter(|b| b.y >= 0) {
            for enemy in self.enemies.iter_mut().filter(|e| e.is_active()) {
                let (ex, ey) = enemy.pos();
                if beam.x < ex + e_s.width
                    && beam.x + b_s.width > ex
                    && beam.y < ey + e_s.height as i32
                    && beam.y + b_s.height as i32 > ey
                {
                    enemy.set_active(false);
                    beam.y = -1000;
                    explosions.push((ex + e_s.width / 2, ey as u32 + e_s.height / 2));
                    break;
                }
            }
        }
        for (hx, hy) in explosions {
            self.spawn_explosion(hx, hy);
        }
    }

    pub fn draw(&mut self) {
        let width = self.size.width;
        let height = self.size.height;
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        for s in &self.stars {
            draw_star(frame, width, height, s);
        }

        Self::draw_enemies(frame, width, height, &self.enemies, &self.sprites[2]);
        Self::draw_particles(frame, width, height, &self.particles);
        Self::draw_beams(frame, width, height, &self.beams, &self.sprites[1]);

        let s = &self.sprites[self.ship.sprite_idx];
        draw_sprite(
            frame,
            width,
            height,
            self.ship.x as i32,
            self.ship.y as i32,
            &s.pixels,
            s.width,
            s.height,
        );

        self.pixels.render().unwrap();
    }

    fn draw_enemies(frame: &mut [u8], width: u32, height: u32, enemies: &[Enemy], sprite: &Sprite) {
        for e in enemies.iter().filter(|e| e.active) {
            draw_sprite(
                frame,
                width,
                height,
                e.x as i32,
                e.y as i32,
                &sprite.pixels,
                sprite.width,
                sprite.height,
            );
        }
    }

    fn draw_beams(frame: &mut [u8], width: u32, height: u32, beams: &[Beam], sprite: &Sprite) {
        for b in beams {
            draw_sprite(
                frame,
                width,
                height,
                b.x as i32,
                b.y,
                &sprite.pixels,
                sprite.width,
                sprite.height,
            );
        }
    }

    fn draw_particles(frame: &mut [u8], width: u32, height: u32, particles: &[Particle]) {
        for p in particles {
            if p.x >= 0.0 && p.x < width as f32 && p.y >= 0.0 && p.y < height as f32 {
                let idx = (p.y as u32 * width + p.x as u32) as usize * 4;
                if idx + 3 < frame.len() {
                    frame[idx] = 255;
                    frame[idx + 1] = (150.0 + (p.life * 200.0)).min(255.0) as u8;
                    frame[idx + 2] = 50;
                    frame[idx + 3] = 255;
                }
            }
        }
    }

    fn fire_beam(&mut self) {
        let s = &self.sprites[0];
        let b = &self.sprites[1];
        self.beams.push(Beam {
            x: self.ship.x + (s.width / 2) - (b.width / 2),
            y: self.ship.y as i32 - b.height as i32,
            remain_y: 0.0,
            sprite_idx: 1,
        });
    }

    fn spawn_explosion(&mut self, x: u32, y: u32) {
        for _ in 0..30 {
            let angle = (self.rng.next_u32() % 360) as f32 * (std::f32::consts::PI / 180.0);
            let speed = (self.rng.next_u32() % 200) as f32 + 50.0;
            self.particles.push(Particle {
                x: x as f32,
                y: y as f32,
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                life: 0.4 + (self.rng.next_u32() % 300) as f32 / 1000.0,
            });
        }
    }
}
