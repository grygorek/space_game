// Copyright 2026 Piotr Grygorczuk <grygorek@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use crate::drawing::*;
use crate::entities::{beam::Beam, enemy::Enemy, particle::Particle, ship::Ship, Collidable, Sprite};
use crate::waves::{classic::ClassicWave, swoop::SwoopWave, WaveType};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;

use crate::input::InputState;
use crate::rng::SimpleRng;
use crate::stars::{draw_star, generate_stars, update_stars, Star};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;
use winit::window::Window;

// Asset Constants
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");
static BOMB_PNG: &[u8] = include_bytes!("../png/bomb.png");
static SFX_SHOT: &[u8] = include_bytes!("../sfx/laser1.wav");
static SFX_EXPLOSION: &[u8] = include_bytes!("../sfx/explosion.wav");
static SFX_OVERHEAT: &[u8] = include_bytes!("../sfx/Metal_Click.wav");

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
    sfx_overheat: &'static [u8],
}

impl App {
    pub fn new(window: Window, size: PhysicalSize<u32>) -> Self {
        let surface_texture = SurfaceTexture::new(size.width, size.height, &window);
        let pixels = Pixels::new(size.width, size.height, surface_texture).unwrap();
        let mut rng = SimpleRng::seed_from_instant();

        // Load Sprites
        // 1. Update the array to include BOMB_PNG
        let image_data = [SHIP_PNG, BEAM_PNG, ENEMY1_PNG, BOMB_PNG];
        let mut sprites = Vec::new();

        for (i, data) in image_data.iter().enumerate() {
            let img = image::load_from_memory(data).unwrap();

            let (final_width, final_height, final_pixels) = if i == 3 {
                let resized = img.resize(img.width() / 30, img.height() / 30, image::imageops::FilterType::Nearest);
                let rgba = resized.to_rgba8();
                (rgba.width(), rgba.height(), rgba.into_raw())
            } else {
                let rgba = img.to_rgba8();
                (rgba.width(), rgba.height(), rgba.into_raw())
            };

            sprites.push(Sprite { width: final_width, height: final_height, pixels: final_pixels });
        }

        let ship = Ship {
            x: (size.width / 2) - (sprites[0].width / 2),
            y: size.height - (size.height / 5),
            speed: 600.0,
            remain_x: 0.0,
            remain_y: 0.0,
            sprite_idx: 0,
            active: true,
            heat: 0.0,
            is_overheated: false,
        };

        let wave_count = 1;
        let current_wave = WaveType::Classic(ClassicWave::new(wave_count));
        let enemies = current_wave.deploy(size.width, size.height);

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
            sfx_overheat: SFX_OVERHEAT,
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
                if self.ship.try_fire() {
                    self.fire_beam();
                } else {
                    self.play_sfx(self.sfx_overheat);
                }
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

        self.enemies = self.current_wave.deploy(self.size.width, self.size.height);
    }

    fn update_enemies(&mut self, dt: f32) {
        if self.enemies.is_empty() {
            return;
        }

        self.current_wave.update(
            &mut self.enemies,
            dt,
            self.size.width,
            self.size.height,
            &self.sprites[2],
            self.ship.x as f32,
        );
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

                    self.score += if enemy.is_diving { 5 } else { 1 };

                    beam.y = -1000;
                    beam_explosions.push((enemy.x as u32 + e_w / 2, enemy.y as u32 + e_h / 2));
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
            if let WaveType::Classic(ref mut wave) = self.current_wave {
                let s_x = self.ship.x as f32;
                let s_y = self.ship.y as f32;
                let s_w = self.sprites[0].width as f32;
                let s_h = self.sprites[0].height as f32;
                let b_w = self.sprites[3].width as f32;
                let b_h = self.sprites[3].height as f32;

                let mut hit_detected = false;

                // 1. Check for hits and remove the bomb
                wave.bombs.retain(|(bx, by, _)| {
                    let hit = *bx < s_x + s_w && *bx + b_w > s_x && *by < s_y + s_h && *by + b_h > s_y;
                    if hit {
                        hit_detected = true;
                        return false; // Remove the bomb
                    }
                    true // Keep the bomb
                });

                // 2. Now that we are OUTSIDE the retain (and the borrow of wave is done),
                // we can safely modify self (ship, explosion, sfx).
                if hit_detected {
                    self.ship.active = false;
                    let center_x = (s_x + s_w / 2.0) as u32;
                    let center_y = (s_y + s_h / 2.0) as u32;
                    self.spawn_explosion(center_x, center_y);
                    self.play_sfx(self.sfx_explosion);
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
        self.enemies = self.current_wave.deploy(self.size.width, self.size.height);

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

        if let WaveType::Classic(ref wave) = self.current_wave {
            let b_sprite = &self.sprites[3];
            // Note the (bx, by, _vx) pattern here
            for (bx, by, _vx) in &wave.bombs {
                draw_sprite(
                    frame,
                    width,
                    height,
                    *bx as i32,
                    *by as i32,
                    &b_sprite.pixels,
                    b_sprite.width,
                    b_sprite.height,
                );
            }
        }

        Self::draw_beams(frame, width, height, &self.beams, &self.sprites[1]);
        Self::draw_particles(frame, width, height, &self.particles);

        if self.ship.is_active() {
            let s = &self.sprites[self.ship.sprite_idx];
            draw_sprite(frame, width, height, self.ship.x as i32, self.ship.y as i32, &s.pixels, s.width, s.height);
        } else {
            draw_text_centered(frame, width, height, "GAMEOVER", 10, COLOR_RED);
        }

        Self::draw_ui(frame, width, height, self.ship.heat, self.ship.is_overheated, self.score);

        self.pixels.render().unwrap();
    }

    fn draw_ui(frame: &mut [u8], width: u32, height: u32, heat: f32, is_overheated: bool, score: u32) {
        // Layout Constants
        let bar_w = 300;
        let bar_h = 30;
        let x = (width as i32 - bar_w as i32) / 2;
        let y = 20;

        // 1. Draw Outline (Thick 4px to match Scale 3 text)
        let outline_color = if is_overheated { COLOR_OVERHEAT_RED } else { COLOR_GRAY_LIGHT };
        draw_rect_outline(frame, width, height, x - 4, y - 4, bar_w + 8, bar_h + 8, 4, outline_color);

        // 2. Draw Background
        draw_rect(frame, width, height, x, y, bar_w, bar_h, COLOR_GRAY_DARK);

        // 3. Draw Heat Fill
        let fill_w = (heat * bar_w as f32) as u32;
        let fill_color = if is_overheated {
            COLOR_OVERHEAT_RED
        } else if heat > 0.5 {
            COLOR_HEAT_ORANGE
        } else {
            COLOR_HEALTH_GREEN
        };
        draw_rect(frame, width, height, x, y, fill_w, bar_h, fill_color);

        // 5. Draw "HEAT" Label (Centered under bar, Scale 2)
        // "HEAT" at scale 2 is roughly 70px wide
        draw_text(frame, width, height, (width / 2) - 35, (y + bar_h as i32 + 10) as u32, "HEAT", 2, COLOR_WHITE);

        // 6. Draw Score (Scale 3 as requested)
        let score_text = format!("SCORE: {}", score);
        draw_text(frame, width, height, 20, 20, &score_text, 3, COLOR_WHITE);
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
