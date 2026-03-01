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
    pub start_x: f32, // Relative offset for return to formation
    pub pivot_x: f32,
    pub pivot_y: f32,
    pub radius: f32,
    pub current_angle: f32,
    pub angular_velocity: f32,
    pub bomb_dropped: bool,
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
    pub bombs: Vec<(f32, f32, f32)>, // (x, y, vx)
    pub rng: SimpleRng,
}

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
            bombs: Vec::new(),
            rng: SimpleRng::seed_from_instant(),
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let (cols, rows, spacing) = (10, 5, 100);
        let start_x = (width as i32 - ((cols - 1) * spacing) as i32) / 2;

        for row in 0..rows {
            for col in 0..cols {
                enemies.push(Enemy {
                    x: (start_x + (col * spacing) as i32) as f32,
                    y: -100.0 - (row as f32 * 60.0),
                    target_y: (row * spacing + 100) as f32,
                    active: true,
                    sprite_idx: 2,
                    is_diving: false,
                });
            }
        }
        enemies
    }

    pub fn update(
        &mut self,
        enemies: &mut Vec<Enemy>,
        dt: f32,
        width: u32,
        sprite: &Sprite,
        ship_x: f32,
    ) -> Vec<(f32, f32, f32)> {
        // Entry movement
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
            let max_divers = ((self.speed / 200.0).floor() as usize).max(1);

            if self.dive_timer >= self.dive_interval && self.divers.len() < max_divers {
                self.launch_diver(enemies, ship_x);
                self.dive_timer = 0.0;
            }

            self.move_in_formation(enemies, dt, width, sprite);
            self.update_divers(enemies, dt, width);
        }

        // Bomb Physics
        self.bombs.retain_mut(|(x, y, vx)| {
            *x += *vx * dt;
            *y += 450.0 * dt;
            *y < 950.0 && *x > -50.0 && *x < width as f32 + 50.0
        });

        self.bombs.clone()
    }
    fn launch_diver(&mut self, enemies: &mut Vec<Enemy>, ship_x: f32) {
        let eligible: Vec<usize> = enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.active && !e.is_diving && e.y >= e.target_y)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = eligible.get(self.rng.next_range(0, eligible.len())) {
            let formation_center_x = self.get_formation_center(enemies);

            let enemy = &mut enemies[idx];
            enemy.is_diving = true;

            let target_y = 800.0;
            let dx = ship_x - enemy.x;
            let dy = target_y - enemy.y;

            let mid_x = (enemy.x + ship_x) / 2.0;
            let mid_y = (enemy.y + target_y) / 2.0;

            // INCREASED CURVE INTENSITY & ADDED Y-OFFSET
            // This pushes the pivot point significantly further away and LOWER.
            // A lower pivot means the "bottom" of the circle is off-screen.
            let curve_intensity = 0.8;
            let p_x = mid_x - dy * curve_intensity;
            let p_y = (mid_y + dx * curve_intensity) + 200.0; // Force pivot down

            let r = ((enemy.x - p_x).powi(2) + (enemy.y - p_y).powi(2)).sqrt();
            let start_angle = (enemy.y - p_y).atan2(enemy.x - p_x);
            let target_angle = (target_y - p_y).atan2(ship_x - p_x);

            let mut diff = target_angle - start_angle;
            if diff > std::f32::consts::PI {
                diff -= 2.0 * std::f32::consts::PI;
            }
            if diff < -std::f32::consts::PI {
                diff += 2.0 * std::f32::consts::PI;
            }
            let direction = if diff > 0.0 { 1.0 } else { -1.0 };

            self.divers.push(Diver {
                enemy_index: idx,
                start_x: enemy.x - formation_center_x,
                pivot_x: p_x,
                pivot_y: p_y,
                radius: r,
                current_angle: start_angle,
                // Ensure speed is high enough to feel like a dive
                angular_velocity: direction * (400.0 / r).clamp(0.8, 2.5),
                bomb_dropped: false,
            });
        }
    }

    pub fn update_divers(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32) {
        let formation_center_x = self.get_formation_center(enemies);

        self.divers.retain_mut(|diver| {
            let enemy = &mut enemies[diver.enemy_index];
            if !enemy.active {
                return false;
            }

            diver.current_angle += diver.angular_velocity * dt;
            enemy.x = diver.pivot_x + diver.current_angle.cos() * diver.radius;
            enemy.y = diver.pivot_y + diver.current_angle.sin() * diver.radius;

            if !diver.bomb_dropped && enemy.y > 400.0 {
                let vx = -diver.radius * diver.angular_velocity * diver.current_angle.sin();
                self.bombs.push((enemy.x + 16.0, enemy.y + 20.0, vx * 0.4));
                diver.bomb_dropped = true;
            }

            // RIGOROUS SCREEN BOUNDARY CHECK
            // If the ship is outside this box, it's gone.
            let is_off_screen = enemy.y > 910.0 ||      // Below screen
                                enemy.x < -100.0 ||     // Left of screen
                                enemy.x > width as f32 + 100.0 || // Right of screen
                                (enemy.y < -100.0 && diver.angular_velocity.abs() > 0.0); // Back at top

            if is_off_screen {
                enemy.is_diving = false;
                enemy.y = -200.0; // Reset way up high
                enemy.x = formation_center_x + diver.start_x;
                return false;
            }
            true
        });
    }

    fn get_formation_center(&self, enemies: &[Enemy]) -> f32 {
        let active_in_formation: Vec<&Enemy> = enemies.iter().filter(|e| e.active && !e.is_diving).collect();
        if !active_in_formation.is_empty() {
            active_in_formation.iter().map(|e| e.x).sum::<f32>() / active_in_formation.len() as f32
        } else {
            400.0
        }
    }

    pub fn move_in_formation(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
        let mut hit_edge = false;
        let vel = self.speed * self.direction;

        for enemy in enemies.iter_mut().filter(|e| e.active && !e.is_diving) {
            enemy.x += vel * dt;
            if (enemy.x <= 20.0 && self.direction < 0.0)
                || (enemy.x + sprite.width as f32 >= width as f32 - 20.0 && self.direction > 0.0)
            {
                hit_edge = true;
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
}
