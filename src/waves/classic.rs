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
use crate::entities::Sprite;
use crate::rng::SimpleRng;

pub struct Diver {
    pub enemy_index: usize,
    pub timer: f32,
    pub start_x: f32,
    pub direction: f32,
}

pub struct ClassicWave {
    pub speed: f32,
    pub direction: f32,
    pub drop_dist: u32,
    pub idle_timer: f32,
    pub max_speed: f32,

    pub divers: Vec<Diver>,
    pub dive_timer: f32,
    pub dive_interval: f32,

    pub rng: SimpleRng,
}
// ... (Diver and ClassicWave struct definitions remain the same) ...

impl ClassicWave {
    pub fn new(wave_count: u32) -> Self {
        Self {
            speed: 200.0 + (wave_count as f32 * 30.0),
            direction: 1.0,
            drop_dist: 20,
            idle_timer: 0.0,
            max_speed: 800.0,
            divers: Vec::new(),
            dive_timer: 0.0,
            dive_interval: (4.0 - (wave_count as f32 * 0.5)).max(1.0),
            rng: SimpleRng::seed_from_instant(),
        }
    }

    pub fn update(
        &mut self,
        enemies: &mut Vec<Enemy>,
        dt: f32,
        width: u32,
        sprite: &Sprite,
        ship_x: f32,
    ) -> Vec<(f32, f32)> {
        let mut bombs = Vec::new();

        for enemy in enemies.iter_mut().filter(|e| e.active && !e.is_diving) {
            if enemy.y < enemy.target_y {
                enemy.y += 250.0 * dt;
                if enemy.y > enemy.target_y {
                    enemy.y = enemy.target_y;
                }
            }
        }

        let any_enemy_on_screen = enemies.iter().any(|e| e.active && e.y >= e.target_y);

        if any_enemy_on_screen {
            self.dive_timer += dt;

            let max_divers = ((self.speed / 250.0).floor() as usize).max(1);

            if self.dive_timer >= self.dive_interval {
                if self.divers.len() < max_divers {
                    self.launch_diver(enemies);
                    self.dive_timer = 0.0;
                }
            }

            self.move_in_formation(enemies, dt, width, sprite);
            bombs = self.update_divers(enemies, dt, ship_x);
        }

        bombs
    }

    fn launch_diver(&mut self, enemies: &mut Vec<Enemy>) {
        let eligible_indices: Vec<usize> = enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.active && !e.is_diving && e.y >= e.target_y)
            .map(|(idx, _)| idx)
            .collect();

        if !eligible_indices.is_empty() {
            let random_pick = self.rng.next_range(0, eligible_indices.len());
            let idx = eligible_indices[random_pick];

            let formation_ships: Vec<&Enemy> = enemies.iter().filter(|e| e.active && !e.is_diving).collect();

            // Safety fallback if the last formation ship just started diving
            let center_x = if !formation_ships.is_empty() {
                formation_ships.iter().map(|e| e.x).sum::<f32>() / formation_ships.len() as f32
            } else {
                400.0
            };

            let enemy = &mut enemies[idx];
            enemy.is_diving = true;

            self.divers.push(Diver {
                enemy_index: idx,
                timer: 0.0,
                start_x: enemy.x - center_x,
                direction: if enemy.x > center_x { 1.0 } else { -1.0 },
            });
        }
    }

    pub fn update_divers(&mut self, enemies: &mut Vec<Enemy>, dt: f32, ship_x: f32) -> Vec<(f32, f32)> {
        let mut new_bombs = Vec::new();

        // 1. Calculate the current center of the formation for returning ships
        let formation_ships: Vec<&Enemy> = enemies.iter().filter(|e| e.active && !e.is_diving).collect();
        let formation_center_x = if !formation_ships.is_empty() {
            formation_ships.iter().map(|e| e.x).sum::<f32>() / formation_ships.len() as f32
        } else {
            400.0
        };

        self.divers.retain_mut(|diver| {
            let enemy = &mut enemies[diver.enemy_index];
            if !enemy.active {
                return false;
            }

            diver.timer += dt;

            // Increased from 350.0 to 500.0 for a punchier dive
            enemy.y += 500.0 * dt;

            // --- THE ARC MATH ---
            // flight_progress reaches 1.0 (full hook) at around 0.7 seconds
            let flight_progress = (diver.timer * 1.4).min(1.0);

            // swing_out: Pushes them away from formation center initially
            let swing_out = (1.0 - flight_progress) * 450.0 * diver.direction;

            // pull_to_player: Gradually steers them toward the ship's X position
            let pull_to_player = flight_progress * (ship_x - enemy.x).signum() * 250.0;

            // Combine the forces
            enemy.x += (swing_out + pull_to_player) * dt;

            // --- SCREEN BOUNDARY CHECK ---
            // Prevents the arc from taking them off the 800px wide screen
            enemy.x = enemy.x.clamp(10.0, 790.0);

            // --- WARP / RETURN LOGIC ---
            if enemy.y > 850.0 {
                enemy.y = -100.0;
                // Snap back to their relative spot in the moving grid
                enemy.x = formation_center_x + diver.start_x;
                enemy.is_diving = false;
                return false;
            }
            true
        });

        new_bombs
    }

    pub fn move_in_formation(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
        let mut hit_edge = false;
        let current_velocity = self.speed * self.direction;

        for enemy in enemies.iter_mut().filter(|e| e.active) {
            if !enemy.is_diving {
                enemy.x += current_velocity * dt;
                if (enemy.x <= 20.0 && self.direction < 0.0)
                    || (enemy.x + sprite.width as f32 >= width as f32 - 20.0 && self.direction > 0.0)
                {
                    hit_edge = true;
                }
            }
        }

        if hit_edge {
            self.direction *= -1.0;
            for enemy in enemies.iter_mut() {
                enemy.target_y += self.drop_dist as f32;
                if !enemy.is_diving {
                    enemy.y += self.drop_dist as f32;
                }
            }
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let (cols, rows, spacing) = (10, 5, 100);
        let formation_width = (cols - 1) * spacing;
        let start_x = (width as i32 - formation_width as i32) / 2;

        for row in 0..rows {
            for col in 0..cols {
                let target_y = (row * spacing + 100) as f32;
                enemies.push(Enemy {
                    x: (start_x + (col * spacing) as i32) as f32,
                    y: -100.0 - (row as f32 * 60.0),
                    target_y,
                    active: true,
                    sprite_idx: 2,
                    is_diving: false,
                });
            }
        }
        enemies
    }
}
