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

pub struct ClassicWave {
    pub speed: f32,
    pub direction: f32,
    pub drop_dist: u32,
    pub idle_timer: f32,
    pub max_speed: f32,
}

impl ClassicWave {
    pub fn new(wave_count: u32) -> Self {
        Self {
            speed: 200.0 + (wave_count as f32 * 30.0),
            direction: 1.0,
            drop_dist: 20,
            idle_timer: 0.0,
            max_speed: 800.0,
        }
    }

    pub fn update(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
        // 1. Check: Are we still in the "Intro" phase?
        // We filter for active enemies so dead ones don't hold up the parade.
        let is_entering = enemies.iter().filter(|e| e.active).any(|e| e.y < e.target_y);

        if is_entering {
            // --- PHASE A: THE FLY-IN ---
            for enemy in enemies.iter_mut().filter(|e| e.active) {
                if enemy.y < enemy.target_y {
                    enemy.y += 250.0 * dt; // Constant entry speed

                    // Clamp to target to prevent "jittering" past the goal
                    if enemy.y > enemy.target_y {
                        enemy.y = enemy.target_y;
                    }
                }
            }
        } else {
            // --- PHASE B: THE BATTLE ---
            // Only starts once every living enemy is in position.
            self.move_in_formation(enemies, dt, width, sprite);
        }
    }

    pub fn move_in_formation(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
        let mut hit_edge = false;
        let margin = 20.0; // Margin as f32

        // 1. Calculate Dynamic Speed
        // Scale speed based on how many enemies are dead (Classic Arcade style)
        let alive_count = enemies.iter().filter(|e| e.active).count();
        let kill_progress = 1.0 - (alive_count as f32 / enemies.len() as f32);

        self.idle_timer += dt;
        let idle_boost = (self.idle_timer / 5.0).floor() * 0.1;

        let magnitude = (self.speed * (1.0 + kill_progress + idle_boost)).min(self.max_speed);
        let current_velocity = magnitude * self.direction;

        // Check boundaries using float math
        let sprite_w = sprite.width as f32;
        let screen_w = width as f32;

        // 2. Horizontal Movement & Edge Detection
        for enemy in enemies.iter_mut().filter(|e| e.active) {
            enemy.x += current_velocity * dt;

            if (enemy.x <= margin && self.direction < 0.0)
                || (enemy.x + sprite_w >= screen_w - margin && self.direction > 0.0)
            {
                hit_edge = true;
            }
        }

        // 3. Row Advancement (The "Drop")
        if hit_edge {
            self.direction *= -1.0;
            let drop_amount = self.drop_dist as f32;

            for enemy in enemies.iter_mut() {
                enemy.y += drop_amount;

                // Small horizontal "nudge" to ensure they aren't still
                // touching the trigger zone on the next frame.
                enemy.x += self.direction * 5.0;
            }
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let (cols, rows, spacing) = (10, 5, 100);

        // Calculate total width of the formation to center it
        let formation_width = (cols - 1) * spacing;
        let start_x = (width as i32 - formation_width as i32) / 2;

        for row in 0..rows {
            for col in 0..cols {
                let target_y = (row * spacing + 100) as f32;
                enemies.push(Enemy {
                    x: (start_x + (col * spacing) as i32) as f32,
                    y: -100.0 - (row as f32 * 60.0), // Tighter staggered entry
                    target_y,
                    active: true,
                    remain_x: 0.0,
                    sprite_idx: 2,
                });
            }
        }
        enemies
    }
}
