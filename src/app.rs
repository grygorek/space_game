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

use crate::assets::Assets;
use crate::audio::AudioController;
use crate::input::InputState;
use crate::leaderboard::Leaderboard;
use crate::rng::SimpleRng;
use crate::stars::{draw_star, generate_stars, update_stars, Star};
use crate::ui;
use pixels::Pixels;
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;

#[derive(PartialEq)]
pub enum GameState {
    StartScreen,
    Playing,
    NameEntry,
    GameOver,
}

pub struct App {
    pub pixels: Pixels,
    pub size: PhysicalSize<u32>,
    pub input: InputState,
    pub score: u32,
    pub state: GameState,
    pub name_input: String,
    leaderboard: Leaderboard,

    // Entities
    lives: u32,
    ship: Ship,
    enemies: Vec<Enemy>,
    current_wave: Box<dyn Wave>,
    wave_count: u32,
    beams: Vec<Beam>,
    particles: Vec<Particle>,
    stars: Vec<Star>,
    rng: SimpleRng,

    assets: Assets,
    audio_controller: AudioController,
}

impl App {
    pub fn new(pixels: Pixels, size: PhysicalSize<u32>) -> Self {
        let mut rng = SimpleRng::seed_from_instant();
        let assets = Assets::load();

        let ship = Ship::new(size.width, size.height, assets.ship().width);

        Self {
            pixels,
            size,
            rng,
            input: InputState::new(),
            score: 0,
            state: GameState::StartScreen,
            name_input: String::new(),
            leaderboard: Leaderboard::new(),
            lives: 3,
            ship,
            enemies: Vec::new(),
            current_wave: Box::new(ClassicWave::new(1)),
            wave_count: 1,
            beams: Vec::new(),
            particles: Vec::new(),
            stars: generate_stars(&mut rng, size),
            audio_controller: AudioController::new(),
            assets,
        }
    }

    // --- MAIN LOOP ---

    pub fn update(&mut self, dt: f32) {
        update_stars(&mut self.stars, &mut self.rng, self.size, dt);
        self.update_enemies(dt);
        self.update_beams(dt);
        self.update_particles(dt);
        self.process_collisions();

        match self.state {
            GameState::StartScreen => self.update_start_screen(),
            GameState::Playing => self.update_playing(dt),
            GameState::NameEntry => self.update_name_entry(),
            GameState::GameOver => self.update_game_over(),
        }

        self.input.clear_just_pressed();
    }

    pub fn draw(&mut self) {
        let (w, h) = (self.size.width, self.size.height);
        let frame = self.pixels.frame_mut();
        frame.fill(0);

        // 1. Static Background
        Self::draw_background(frame, w, h, &self.stars);

        // 2. Entities (Always drawn for most states)
        if self.state != GameState::StartScreen {
            Self::draw_gameplay_entities(
                frame,
                w,
                h,
                &self.enemies,
                &self.beams,
                &self.particles,
                &self.ship,
                &self.assets.sprites,
                &*self.current_wave,
            );
            ui::draw_hud(frame, w, h, self.ship.heat, self.ship.is_overheated, self.score, self.lives, &self.assets.ship());
        }

        // 3. State-specific Overlays
        match self.state {
            GameState::StartScreen => {
                let s = self.assets.ship();
                draw_sprite(frame, w, h, self.ship.x as i32, self.ship.y as i32, &s.pixels, s.width, s.height);
                ui::draw_start_menu(frame, w, h);
            }
            GameState::NameEntry => ui::draw_name_entry_overlay(frame, w, h, &self.name_input),
            GameState::GameOver => ui::draw_game_over_overlay(frame, w, h, &self.leaderboard.entries),
            GameState::Playing => {} // HUD already drawn above
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

        if self.current_wave.is_extinct(&self.enemies) {
            self.transition_wave();
        }

        if self.lives == 0 && !self.ship.active {
            let min_high = self.leaderboard.entries.last().map(|(_, s)| *s).unwrap_or(0);
            if self.score > min_high || self.leaderboard.entries.len() < 5 {
                self.name_input.clear();
                self.state = GameState::NameEntry;
            } else {
                self.state = GameState::GameOver;
            }
        }
    }

    fn update_name_entry(&mut self) {
        for key_code in VirtualKeyCode::A as u32..=VirtualKeyCode::Z as u32 {
            let vk: VirtualKeyCode = unsafe { std::mem::transmute(key_code) };
            if self.input.was_key_pressed(vk) {
                if self.name_input.len() < 3 {
                    let letter = (b'A' + (key_code - VirtualKeyCode::A as u32) as u8) as char;
                    self.name_input.push(letter);
                }
            }
        }

        if self.input.was_key_pressed(VirtualKeyCode::Back) {
            self.name_input.pop();
        }

        if self.input.was_key_pressed(VirtualKeyCode::Return) && self.name_input.len() == 3 {
            self.leaderboard.add_entry(self.name_input.clone(), self.score);
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
        let s_img = &self.assets.ship();
        self.ship.update(&self.input, self.size, s_img.width, s_img.height, dt);

        if self.input.was_key_pressed(VirtualKeyCode::Space) {
            if self.ship.try_fire() {
                self.fire_beam();
            } else {
                self.audio_controller.play_sfx(self.assets.sfx_overheat);
            }
        }
    }

    // --- DRAW HELPERS ---

    fn draw_background(frame: &mut [u8], w: u32, h: u32, stars: &[Star]) {
        for s in stars {
            draw_star(frame, w, h, s);
        }
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

        if ship.is_visible() {
            let s = &sprites[ship.sprite_idx];
            draw_sprite(frame, w, h, ship.x as i32, ship.y as i32, &s.pixels, s.width, s.height);
        }
    }

    // --- LOGIC HELPERS ---

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
            &self.assets.enemy(),
            self.ship.x,
        );
    }

    fn process_collisions(&mut self) {
        let (s_w, s_h) = (self.assets.ship().width, self.assets.ship().height);
        let (e_w, e_h) = (self.assets.enemy().width, self.assets.enemy().height);
        let mut play_explosion = false;
        let mut beam_explosions = Vec::new();

        for beam in self.beams.iter_mut() {
            for enemy in self.enemies.iter_mut().filter(|e| e.active) {
                if beam.collides_with(enemy, &self.assets.beam(), &self.assets.enemy()) {
                    enemy.active = false;
                    if self.state == GameState::Playing {
                        self.score += if enemy.is_diving { 5 } else { 1 };
                    }
                    beam.y = -1000.0;
                    beam_explosions.push(((enemy.x + e_w as f32 / 2.0) as u32, (enemy.y + e_h as f32 / 2.0) as u32));
                    play_explosion = true;
                    self.current_wave.on_enemy_killed();
                    break;
                }
            }
        }

        if self.ship.active && !self.ship.is_invincible() {
            if self.current_wave.check_player_collision(&self.ship, &self.assets.bomb(), &self.assets.ship()) {
                self.destroy_ship(s_w, s_h);
            }
            for enemy in self.enemies.iter().filter(|e| e.active) {
                if enemy.collides_with(&self.ship, &self.assets.enemy(), &self.assets.ship()) {
                    self.destroy_ship(s_w, s_h);
                    break;
                }
            }
        }

        for (hx, hy) in beam_explosions {
            self.spawn_explosion(hx, hy);
        }
        if play_explosion {
            self.audio_controller.play_sfx(self.assets.sfx_explosion);
        }
    }

    fn destroy_ship(&mut self, s_w: u32, s_h: u32) {
        self.spawn_explosion((self.ship.x + s_w as f32 / 2.0) as u32, (self.ship.y + s_h as f32 / 2.0) as u32);
        self.audio_controller.play_sfx(self.assets.sfx_explosion);

        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            self.ship.active = false;
        } else {
            self.ship.respawn(self.size.width, self.size.height, s_w);
        }
    }

    pub fn reset(&mut self) {
        self.lives = 3;
        self.ship.respawn(self.size.width, self.size.height, self.assets.ship().width);

        self.wave_count = 1;
        self.current_wave = Box::new(ClassicWave::new(1));
        self.enemies = self.current_wave.deploy(self.size.width, self.size.height);
        self.beams.clear();
        self.particles.clear();

        self.score = 0;

        self.state = GameState::Playing;
    }

    fn update_beams(&mut self, dt: f32) {
        let b_h = self.assets.beam().height;
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
        let s = &self.assets.ship();
        let b = &self.assets.beam();
        self.beams.push(Beam {
            x: self.ship.x + s.width as f32 / 2.0 - b.width as f32 / 2.0,
            y: self.ship.y - b.height as f32,
            sprite_idx: 1,
        });

        self.audio_controller.play_sfx(self.assets.sfx_shot);
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
