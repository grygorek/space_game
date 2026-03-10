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
use crate::waves::{classic::ClassicWave, swoop::SwoopWave, Wave};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;

use crate::input::InputState;
use crate::rng::SimpleRng;
use crate::stars::{draw_star, generate_stars, update_stars, Star};
use pixels::Pixels;
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;

// Asset Constants
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");
static BOMB_PNG: &[u8] = include_bytes!("../png/bomb.png");
static SFX_SHOT: &[u8] = include_bytes!("../sfx/laser1.wav");
static SFX_EXPLOSION: &[u8] = include_bytes!("../sfx/explosion.wav");
static SFX_OVERHEAT: &[u8] = include_bytes!("../sfx/Metal_Click.wav");

#[derive(PartialEq)]
pub enum GameState {
    StartScreen,
    Playing,
    GameOver,
}

pub struct App {
    pub pixels: Pixels,
    pub size: PhysicalSize<u32>,
    pub input: InputState,
    pub score: u32,
    pub state: GameState,

    // Entities
    ship: Ship,
    enemies: Vec<Enemy>,
    current_wave: Box<dyn Wave>,
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
    pub fn new(pixels: Pixels, size: PhysicalSize<u32>) -> Self {
        let mut rng = SimpleRng::seed_from_instant();

        let image_data = [SHIP_PNG, BEAM_PNG, ENEMY1_PNG, BOMB_PNG];
        let mut sprites = Vec::new();

        for (i, data) in image_data.iter().enumerate() {
            let img = image::load_from_memory(data).unwrap();
            let (w, h, pix) = if i == 3 {
                let res = img.resize(img.width() / 20, img.height() / 20, image::imageops::FilterType::Nearest);
                (res.width(), res.height(), res.to_rgba8().into_raw())
            } else {
                (img.width(), img.height(), img.to_rgba8().into_raw())
            };
            sprites.push(Sprite { width: w, height: h, pixels: pix });
        }

        let ship = Ship {
            x: (size.width / 2 - sprites[0].width / 2) as f32,
            y: (size.height - size.height / 5) as f32,
            speed: 600.0,
            sprite_idx: 0,
            active: true,
            heat: 0.0,
            is_overheated: false,
        };

        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            pixels,
            size,
            rng,
            input: InputState::new(),
            score: 0,
            state: GameState::StartScreen,
            ship,
            enemies: Vec::new(),
            current_wave: Box::new(ClassicWave::new(1)),
            wave_count: 1,
            sprites,
            beams: Vec::new(),
            particles: Vec::new(),
            stars: generate_stars(&mut rng, size),
            _stream: stream,
            stream_handle,
            sfx_shot: SFX_SHOT,
            sfx_explosion: SFX_EXPLOSION,
            sfx_overheat: SFX_OVERHEAT,
        }
    }

    // --- MAIN LOOP ---

    pub fn update(&mut self, dt: f32) {
        update_stars(&mut self.stars, &mut self.rng, self.size, dt);

        match self.state {
            GameState::StartScreen => self.update_start_screen(),
            GameState::Playing => self.update_playing(dt),
            GameState::GameOver => self.update_game_over(),
        }

        self.input.clear_just_pressed();
    }

    pub fn draw(&mut self) {
        let (w, h) = (self.size.width, self.size.height);
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        // Draw background stars
        Self::draw_background(frame, w, h, &self.stars);

        match self.state {
            GameState::StartScreen => {
                Self::draw_start_menu(frame, w, h, &self.ship, &self.sprites);
            }
            GameState::Playing | GameState::GameOver => {
                Self::draw_gameplay_entities(
                    frame,
                    w,
                    h,
                    &self.enemies,
                    &self.beams,
                    &self.particles,
                    &self.ship,
                    &self.sprites,
                    &*self.current_wave,
                );

                Self::draw_hud(frame, w, h, self.ship.heat, self.ship.is_overheated, self.score);

                if self.state == GameState::GameOver {
                    Self::draw_game_over_overlay(frame, w, h);
                }
            }
        }

        self.pixels.render().unwrap();
    }

    // --- UPDATE HELPERS ---

    fn update_start_screen(&mut self) {
        if self.input.was_key_pressed(VirtualKeyCode::Space) {
            self.reset();
            self.state = GameState::Playing;
        }
    }

    fn update_playing(&mut self, dt: f32) {
        if self.input.was_key_pressed(VirtualKeyCode::R) {
            self.reset();
            return;
        }

        self.handle_player_input(dt);
        self.update_enemies(dt);
        self.update_beams(dt);
        self.update_particles(dt);
        self.process_collisions();

        if self.current_wave.is_extinct(&self.enemies) {
            self.transition_wave();
        }

        if !self.ship.is_active() {
            self.state = GameState::GameOver;
        }
    }

    fn update_game_over(&mut self) {
        if self.input.was_key_pressed(VirtualKeyCode::R) {
            self.reset();
            self.state = GameState::Playing;
        }
    }

    fn handle_player_input(&mut self, dt: f32) {
        if !self.ship.active {
            return;
        }
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

    // --- DRAW HELPERS (Associated Functions to avoid Borrow Checker issues) ---

    fn draw_background(frame: &mut [u8], w: u32, h: u32, stars: &[Star]) {
        for s in stars {
            draw_star(frame, w, h, s);
        }
    }

    fn draw_start_menu(frame: &mut [u8], w: u32, h: u32, ship: &Ship, sprites: &[Sprite]) {
        let s = &sprites[ship.sprite_idx];
        draw_sprite(frame, w, h, ship.x as i32, ship.y as i32, &s.pixels, s.width, s.height);

        draw_text_centered(frame, w, h, "SPACE GAME", 8, COLOR_SCORE_GOLD);
        draw_text(frame, w, h, (w / 2) - 150, (h / 2) + 100, "PRESS SPACE TO START", 2, COLOR_WHITE);
    }

    fn draw_gameplay_entities(
        frame: &mut [u8],
        w: u32,
        h: u32,
        enemies: &[Enemy],
        beams: &[Beam],
        particles: &[Particle],
        ship: &Ship,
        sprites: &[Sprite],
        wave: &dyn Wave,
    ) {
        Self::draw_enemies(frame, w, h, enemies, &sprites[2]);
        wave.draw_projectiles(frame, w, h, &sprites[3]);
        Self::draw_beams(frame, w, h, beams, &sprites[1]);
        Self::draw_particles(frame, w, h, particles);

        if ship.active {
            let s = &sprites[ship.sprite_idx];
            draw_sprite(frame, w, h, ship.x as i32, ship.y as i32, &s.pixels, s.width, s.height);
        }
    }

    fn draw_hud(frame: &mut [u8], w: u32, h: u32, heat: f32, is_overheated: bool, score: u32) {
        Self::draw_ui(frame, w, h, heat, is_overheated, score);
    }

    fn draw_game_over_overlay(frame: &mut [u8], w: u32, h: u32) {
        draw_text_centered(frame, w, h, "GAMEOVER", 10, COLOR_RED);
        draw_text(frame, w, h, (w / 2) - 120, (h / 2) + 80, "PRESS R TO RESTART", 2, COLOR_WHITE);
    }

    // --- LOGIC HELPER FUNCTIONS ---

    fn transition_wave(&mut self) {
        self.wave_count += 1;

        // Cycle behavior: Every 3rd wave is a Swoop
        self.current_wave = if self.wave_count % 3 == 0 {
            Box::new(SwoopWave::new(self.wave_count))
        } else {
            Box::new(ClassicWave::new(self.wave_count))
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
            self.ship.x,
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

                    beam.y = -1000.0;
                    beam_explosions.push(((enemy.x + e_w as f32 / 2.0) as u32, (enemy.y + e_h as f32 / 2.0) as u32));
                    play_explosion = true;

                    self.current_wave.on_enemy_killed();
                    break;
                }
            }
        }

        if self.ship.active {
            if self.current_wave.check_player_collision(&self.ship, &self.sprites[3], &self.sprites[0]) {
                self.destroy_ship(s_w, s_h);
            }
            for enemy in self.enemies.iter().filter(|e| e.active) {
                if enemy.collides_with(&self.ship, &self.sprites[2], &self.sprites[0]) {
                    self.destroy_ship(s_w, s_h);
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

    fn destroy_ship(&mut self, s_w: u32, s_h: u32) {
        self.ship.active = false;
        self.spawn_explosion((self.ship.x + s_w as f32 / 2.0) as u32, (self.ship.y + s_h as f32 / 2.0) as u32);
        self.play_sfx(self.sfx_explosion);
    }

    pub fn reset(&mut self) {
        self.ship.active = true;
        self.ship.x = (self.size.width / 2 - self.sprites[0].width / 2) as f32;
        self.ship.y = (self.size.height - self.size.height / 5) as f32;
        self.ship.heat = 0.0;
        self.ship.is_overheated = false;

        self.wave_count = 1;
        self.current_wave = Box::new(ClassicWave::new(self.wave_count));
        self.enemies = self.current_wave.deploy(self.size.width, self.size.height);
        self.beams.clear();
        self.particles.clear();

        self.score = 0;

        self.state = GameState::Playing;
    }

    fn update_beams(&mut self, dt: f32) {
        let b_h = self.sprites[1].height;
        for beam in self.beams.iter_mut() {
            beam.y -= 1000.0 * dt;
        }
        self.beams.retain(|b| b.y + b_h as f32 > 0.0);
    }

    fn update_particles(&mut self, dt: f32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
    }

    fn draw_ui(frame: &mut [u8], width: u32, height: u32, heat: f32, is_overheated: bool, score: u32) {
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
        draw_text(frame, width, height, 20, 20, &format!("SCORE: {}", score), 3, COLOR_WHITE);
    }

    fn draw_enemies(frame: &mut [u8], width: u32, height: u32, enemies: &[Enemy], sprite: &Sprite) {
        for e in enemies.iter().filter(|e| e.active) {
            draw_sprite(frame, width, height, e.x as i32, e.y as i32, &sprite.pixels, sprite.width, sprite.height);
        }
    }

    fn draw_beams(frame: &mut [u8], width: u32, height: u32, beams: &[Beam], sprite: &Sprite) {
        for b in beams {
            draw_sprite(frame, width, height, b.x as i32, b.y as i32, &sprite.pixels, sprite.width, sprite.height);
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
            x: self.ship.x + s.width as f32 / 2.0 - b.width as f32 / 2.0,
            y: self.ship.y - b.height as f32,
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
        let source = Decoder::new(Cursor::new(data)).unwrap();
        let _ = self.stream_handle.play_raw(source.convert_samples());
    }
}
