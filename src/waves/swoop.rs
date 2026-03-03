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

use crate::entities::enemy::Enemy;
use crate::entities::ship::Ship;
use crate::entities::Sprite;
use crate::rng::SimpleRng;
use crate::waves::Wave;

pub struct SwoopedDiver {
    pub enemy_index: usize,
    pub dive_speed: f32,
    pub is_returning: bool,
    pub target_x: f32,
}

pub struct SwoopWave {
    pub timer: f32,
    pub center_y: f32,
    pub divers: Vec<SwoopedDiver>,
    pub dive_timer: f32,
    pub dive_interval: f32,
    pub rng: SimpleRng,
}

impl SwoopWave {
    pub fn new(wave_count: u32) -> Self {
        Self {
            timer: 0.0,
            center_y: 300.0,
            divers: Vec::new(),
            dive_timer: 0.0,
            // Dive interval gets shorter (faster) as waves progress
            dive_interval: (5.0 - (wave_count as f32 * 0.5)).max(1.5),
            rng: SimpleRng::seed_from_instant(),
        }
    }

    fn launch_diver(&mut self, enemies: &mut Vec<Enemy>, ship_x: f32) {
        let eligible: Vec<usize> =
            enemies.iter().enumerate().filter(|(_, e)| e.active && !e.is_diving).map(|(i, _)| i).collect();

        if !eligible.is_empty() {
            let idx = eligible[self.rng.next_range(0, eligible.len())];
            enemies[idx].is_diving = true;

            self.divers.push(SwoopedDiver {
                enemy_index: idx,
                dive_speed: 650.0,
                is_returning: false,
                target_x: ship_x,
            });
        }
    }

    fn update_divers(&mut self, enemies: &mut Vec<Enemy>, dt: f32, height: u32) {
        self.divers.retain_mut(|diver| {
            let enemy = &mut enemies[diver.enemy_index];
            if !enemy.active {
                return false;
            }

            if !diver.is_returning {
                // Kamikaze Phase
                enemy.y += diver.dive_speed * dt;
                // Aggressive horizontal tracking
                let dx = diver.target_x - enemy.x;
                enemy.x += dx * dt * 2.5;

                if enemy.y > height as f32 + 50.0 {
                    diver.is_returning = true;
                    enemy.y = -100.0; // Reset above screen to "fly back" into formation
                }
                true
            } else {
                // Return Phase: once back at the top, let formation math take over
                enemy.is_diving = false;
                false // Remove from divers list
            }
        });
    }

    fn move_in_formation(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, height: u32) {
        self.timer += dt * 0.8;

        let stop_limit = height as f32 * 0.65;
        if self.center_y < stop_limit {
            self.center_y += dt * 20.0;
        }

        let breathing_factor = 0.85 + (self.timer * 0.4).sin() * 0.15;
        let base_x_radius = (width as f32 * 0.45) * breathing_factor;
        let y_radius = height as f32 * 0.35;
        let phase_tilt = (self.timer * 0.3).cos() * 0.6;

        for (i, enemy) in enemies.iter_mut().enumerate() {
            if !enemy.active || enemy.is_diving {
                continue;
            }

            let offset = i as f32 * 0.18;
            let t = self.timer + offset;

            // Vertical position ratio (-1.0 top to 1.0 bottom)
            let swoop_intensity = (t * 2.0 + phase_tilt).sin();

            // 1. EARLY SPREAD LOGIC:
            // We remove .powi(2) so the spread is linear with depth.
            // We also use a "max" to ensure the spread starts as soon as swoop_intensity > 0.
            let spread_magnitude = if swoop_intensity > 0.0 {
                swoop_intensity * 75.0 // Increased multiplier and linear growth
            } else {
                // Even at the top, we give a tiny bit of separation (5.0)
                // so they never perfectly overlap in a single-file line.
                swoop_intensity.abs() * 5.0
            };

            // Center the spread around the middle of the pack (index 7.5)
            let individual_spread = (i as f32 - 7.5) * spread_magnitude;

            // 2. TURBULENCE:
            let wobble_x = (t * 3.0 + i as f32).sin() * 25.0;
            let wobble_y = (t * 2.5 + i as f32).cos() * 15.0;

            // 3. APPLY POSITION:
            enemy.x = (width as f32 / 2.0) + (t.cos() * base_x_radius) + individual_spread + wobble_x;
            enemy.y = self.center_y + (swoop_intensity * y_radius) + wobble_y;
        }
    }
}

impl Wave for SwoopWave {
    fn deploy(&self, width: u32, _height: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        for _ in 0..15 {
            enemies.push(Enemy {
                x: width as f32 / 2.0,
                y: -100.0,
                target_y: 100.0,
                active: true,
                sprite_idx: 2,
                is_diving: false,
            });
        }
        enemies
    }

    fn update(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, height: u32, _sprite: &Sprite, ship_x: f32) {
        // 1. Formation Movement
        self.move_in_formation(enemies, dt, width, height);

        // 2. Diver Timing
        self.dive_timer += dt;
        if self.dive_timer >= self.dive_interval {
            self.launch_diver(enemies, ship_x);
            self.dive_timer = 0.0;
        }

        // 3. Process active Kamikazes
        self.update_divers(enemies, dt, height);
    }

    fn check_player_collision(&mut self, _ship: &Ship, _p_sprite: &Sprite, _s_sprite: &Sprite) -> bool {
        false // Projectile collisions handled elsewhere if added
    }

    fn draw_projectiles(&self, _frame: &mut [u8], _width: u32, _height: u32, _sprite: &Sprite) {}

    fn on_enemy_killed(&mut self) {}

    fn is_extinct(&self, enemies: &[Enemy]) -> bool {
        enemies.iter().all(|e| !e.active)
    }
}
