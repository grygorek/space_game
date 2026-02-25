use crate::drawing::draw_sprite;
use crate::entities::{beam::Beam, enemy::Enemy, particle::Particle, ship::Ship, Collidable, Sprite};
use crate::waves::{classic::ClassicWave, swoop::SwoopWave, WaveType};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;

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
static SFX_SHOT: &[u8] = include_bytes!("../sfx/laser1.wav");
static SFX_EXPLOSION: &[u8] = include_bytes!("../sfx/explosion.wav");

pub struct App {
    pub window: Window,
    pub pixels: Pixels,
    pub size: PhysicalSize<u32>,
    pub input: InputState,
    pub score: u32,

    // Entities
    ship: Ship,
    enemies: Vec<Enemy>,
    current_wave: WaveType,
    wave_count: u32,
    beams: Vec<Beam>,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    rng: SimpleRng,
    sprites: Vec<Sprite>,

    // Audio handle must stay alive for the duration of the app
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,

    // We store the raw bytes of the sounds to "play" them instantly
    sfx_shot: &'static [u8],
    sfx_explosion: &'static [u8],
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
            sprites.push(Sprite { width: img.width(), height: img.height(), pixels: img.into_raw() });
        }

        let ship = Ship {
            x: (size.width / 2) - (sprites[0].width / 2),
            y: size.height - (size.height / 5),
            speed: 600.0,
            remain_x: 0.0,
            remain_y: 0.0,
            sprite_idx: 0,
            active: true,
        };

        let wave_count = 1;
        let current_wave = WaveType::Classic(ClassicWave::new(wave_count));
        let enemies = current_wave.deploy(size.width);

        let stars = generate_stars(&mut rng, size);
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            window,
            pixels,
            size,
            rng,
            input: InputState::new(),
            score: 0,
            ship,
            enemies,
            current_wave,
            wave_count,
            sprites,
            beams: Vec::new(),
            particles: Vec::new(),
            stars,
            _stream: stream,
            stream_handle,
            sfx_shot: SFX_SHOT,
            sfx_explosion: SFX_EXPLOSION,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.input.was_key_pressed(VirtualKeyCode::R) {
            self.reset();
            return;
        }

        update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        if self.ship.is_active() {
            let s_img = &self.sprites[self.ship.sprite_idx];
            self.ship.update(&self.input, self.size, s_img.width, s_img.height, dt);

            if self.input.was_key_pressed(VirtualKeyCode::Space) {
                self.fire_beam();
            }
        }

        self.update_enemies(dt);
        self.update_beams(dt);
        self.update_particles(dt);
        self.process_collisions();

        // Wave Transition Logic
        if self.current_wave.is_extinct(&self.enemies) {
            self.transition_wave();
        }

        self.input.clear_just_pressed();
    }

    fn transition_wave(&mut self) {
        self.wave_count += 1;

        // Cycle behavior: Every 3rd wave is a Swoop
        self.current_wave = if self.wave_count % 3 == 0 {
            WaveType::Swoop(SwoopWave::new())
        } else {
            WaveType::Classic(ClassicWave::new(self.wave_count))
        };

        self.enemies = self.current_wave.deploy(self.size.width);
    }

    fn update_enemies(&mut self, dt: f32) {
        if self.enemies.is_empty() {
            return;
        }

        self.current_wave.update(&mut self.enemies, dt, self.size.width, &self.sprites[2]);
    }

    fn process_collisions(&mut self) {
        let (s_w, s_h) = (self.sprites[0].width, self.sprites[0].height);
        let (e_w, e_h) = (self.sprites[2].width, self.sprites[2].height);

        let mut play_explosion = false;
        let mut beam_explosions = Vec::new();

        for beam in self.beams.iter_mut() {
            for enemy in self.enemies.iter_mut().filter(|e| e.active) {
                if beam.collides_with(enemy, &self.sprites[1], &self.sprites[2]) {
                    enemy.active = false;

                    self.score += 1;

                    beam.y = -1000;
                    beam_explosions.push((enemy.x + e_w / 2, enemy.y + e_h / 2));
                    play_explosion = true;

                    // Type-safe property reset using 'if let'
                    if let WaveType::Classic(ref mut w) = self.current_wave {
                        w.idle_timer = 0.0;
                    }
                    break;
                }
            }
        }

        if self.ship.active {
            let (sx, sy) = (self.ship.x, self.ship.y);
            for enemy in self.enemies.iter().filter(|e| e.active) {
                if enemy.collides_with(&self.ship, &self.sprites[2], &self.sprites[0]) {
                    self.ship.active = false;
                    self.spawn_explosion(sx + s_w / 2, sy + s_h / 2);
                    self.play_sfx(self.sfx_explosion);
                    break;
                }
            }
        }

        for (hx, hy) in beam_explosions {
            self.spawn_explosion(hx, hy);
        }
        if play_explosion {
            self.play_sfx(self.sfx_explosion);
        }
    }

    pub fn reset(&mut self) {
        self.ship.active = true;
        self.ship.x = (self.size.width / 2) - (self.sprites[0].width / 2);
        self.ship.y = self.size.height - (self.size.height / 5);

        self.wave_count = 1;
        self.current_wave = WaveType::Classic(ClassicWave::new(self.wave_count));
        self.enemies = self.current_wave.deploy(self.size.width);

        self.beams.clear();
        self.particles.clear();

        self.score = 0;
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

    pub fn draw(&mut self) {
        let width = self.size.width;
        let height = self.size.height;
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        for s in &self.stars {
            draw_star(frame, width, height, s);
        }

        Self::draw_enemies(frame, width, height, &self.enemies, &self.sprites[2]);
        Self::draw_beams(frame, width, height, &self.beams, &self.sprites[1]);
        Self::draw_particles(frame, width, height, &self.particles);

        let score_text = format!("SCORE: {}", self.score);
        crate::drawing::draw_text(frame, width, height, 20, 20, &score_text, 3, crate::drawing::COLOR_WHITE);

        if self.ship.is_active() {
            let s = &self.sprites[self.ship.sprite_idx];
            draw_sprite(frame, width, height, self.ship.x as i32, self.ship.y as i32, &s.pixels, s.width, s.height);
        } else {
            crate::drawing::draw_text_centered(frame, width, height, "GAMEOVER", 10, crate::drawing::COLOR_RED);
        }
        self.pixels.render().unwrap();
    }

    fn draw_enemies(frame: &mut [u8], width: u32, height: u32, enemies: &[Enemy], sprite: &Sprite) {
        for e in enemies.iter().filter(|e| e.active) {
            draw_sprite(frame, width, height, e.x as i32, e.y as i32, &sprite.pixels, sprite.width, sprite.height);
        }
    }

    fn draw_beams(frame: &mut [u8], width: u32, height: u32, beams: &[Beam], sprite: &Sprite) {
        for b in beams {
            draw_sprite(frame, width, height, b.x as i32, b.y, &sprite.pixels, sprite.width, sprite.height);
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

        self.play_sfx(self.sfx_shot);
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

    fn play_sfx(&self, data: &'static [u8]) {
        let cursor = Cursor::new(data);
        let source = Decoder::new(cursor).unwrap();
        // Use play_raw to avoid creating a new Sink every time
        let _ = self.stream_handle.play_raw(source.convert_samples());
    }
}
