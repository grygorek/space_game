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

use crate::drawing::draw_sprite;
use crate::entities::enemy::Enemy;
use crate::entities::projectile::Projectile;
use crate::entities::ship::Ship;
use crate::entities::Collidable;
use crate::entities::Sprite;
use crate::rng::SimpleRng;
use crate::waves::Wave;

pub struct Diver {
    pub enemy_index: usize,
    pub start_x: f32,
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
    pub idle_timer: f32,
    pub max_speed: f32,
    pub divers: Vec<Diver>,
    pub dive_timer: f32,
    pub dive_interval: f32,
    pub bombs: Vec<Projectile>,
    pub rng: SimpleRng,
}

impl ClassicWave {
    pub fn new(wave_count: u32) -> Self {
        Self {
            speed: 200.0 + (wave_count as f32 * 30.0),
            direction: 1.0,
            idle_timer: 0.0,
            max_speed: 800.0,
            divers: Vec::new(),
            dive_timer: 0.0,
            dive_interval: (4.0 - (wave_count as f32 * 0.5)).max(1.0),
            bombs: Vec::new(),
            rng: SimpleRng::seed_from_instant(),
        }
    }

    fn get_formation_center(&self, enemies: &[Enemy]) -> f32 {
        let active_in_formation: Vec<&Enemy> = enemies.iter().filter(|e| e.active && !e.is_diving).collect();
        if !active_in_formation.is_empty() {
            active_in_formation.iter().map(|e| e.x).sum::<f32>() / active_in_formation.len() as f32
        } else {
            400.0
        }
    }
}

impl Wave for ClassicWave {
    fn deploy(&self, width: u32, height: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let (cols, rows) = (10, 5);
        let spacing_x = (width as f32 * 0.05) as i32;
        let spacing_y = (height as f32 * 0.06) as i32;
        let formation_width = (cols - 1) * spacing_x;
        let start_x = (width as i32 - formation_width) / 2;
        let top_margin = (height as f32 * 0.20) as i32;

        for row in 0..rows {
            for col in 0..cols {
                enemies.push(Enemy {
                    x: (start_x + (col * spacing_x)) as f32,
                    y: -100.0 - (row as f32 * spacing_y as f32),
                    target_y: (top_margin + (row * spacing_y)) as f32,
                    active: true,
                    sprite_idx: 2,
                    is_diving: false,
                });
            }
        }
        enemies
    }

    fn update(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, height: u32, sprite: &Sprite, ship_x: f32) {
        // Show enemies descending into formation
        for enemy in enemies.iter_mut().filter(|e| e.active && !e.is_diving) {
            if enemy.y < enemy.target_y {
                enemy.y += 300.0 * dt;
                if enemy.y > enemy.target_y {
                    enemy.y = enemy.target_y;
                }
            }
        }

        let any_enemy_on_screen = enemies.iter().any(|e| e.active && e.y >= e.target_y);
        if any_enemy_on_screen {
            // Dives
            self.dive_timer += dt;
            let calculated_max = ((self.speed - 120.0) / 100.0).floor() as i32;
            let max_divers = calculated_max.max(1) as usize;

            if self.dive_timer >= self.dive_interval && self.divers.len() < max_divers {
                self.launch_diver(enemies, ship_x, height);
                self.dive_timer = 0.0;
            }
            // Formation & Diver movement
            self.move_in_formation(enemies, dt, width, sprite);
            self.update_divers(enemies, dt, width, height);
        }

        // Updated Bomb Physics using Projectile struct
        self.bombs.retain_mut(|bomb| {
            bomb.x += bomb.vx * dt;
            bomb.y += 450.0 * dt;
            bomb.y < height as f32 + 50.0 && bomb.x > -50.0 && bomb.x < width as f32 + 50.0
        });
    }

    fn check_player_collision(&mut self, ship: &Ship, p_sprite: &Sprite, s_sprite: &Sprite) -> bool {
        let mut hit = false;
        self.bombs.retain(|bomb| {
            if bomb.collides_with(ship, p_sprite, s_sprite) {
                hit = true;
                false
            } else {
                true
            }
        });
        hit
    }

    fn draw_projectiles(&self, frame: &mut [u8], width: u32, height: u32, sprite: &Sprite) {
        for bomb in &self.bombs {
            draw_sprite(
                frame,
                width,
                height,
                bomb.x as i32,
                bomb.y as i32,
                &sprite.pixels,
                sprite.width,
                sprite.height,
            );
        }
    }

    fn on_enemy_killed(&mut self) {
        self.idle_timer = 0.0;
    }

    fn is_extinct(&self, enemies: &[Enemy]) -> bool {
        enemies.iter().all(|e| !e.active)
    }
}

impl ClassicWave {
    fn launch_diver(&mut self, enemies: &mut Vec<Enemy>, ship_x: f32, height: u32) {
        let eligible: Vec<usize> = enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| e.active && !e.is_diving && e.y >= e.target_y)
            .map(|(i, _)| i)
            .collect();

        if let Some(&idx) = eligible.get(self.rng.next_range(0, eligible.len())) {
            let center_x = self.get_formation_center(enemies);
            let enemy = &mut enemies[idx];
            enemy.is_diving = true;

            let side = if ship_x > enemy.x { 1.0 } else { -1.0 };
            let p_x = ((enemy.x + ship_x) / 2.0) - ((height as f32 * 1.25 - enemy.y) * 0.8 * side);
            let p_y =
                ((enemy.y + height as f32 * 1.25) / 2.0) + ((ship_x - enemy.x) * 0.8 * side) + (height as f32 * 0.15);
            let r = ((enemy.x - p_x).powi(2) + (enemy.y - p_y).powi(2)).sqrt();
            let start_angle = (enemy.y - p_y).atan2(enemy.x - p_x);

            self.divers.push(Diver {
                enemy_index: idx,
                start_x: enemy.x - center_x,
                pivot_x: p_x,
                pivot_y: p_y,
                radius: r,
                current_angle: start_angle,
                angular_velocity: side * ((height as f32 * 0.3) / r).clamp(0.5, 1.8),
                bomb_dropped: false,
            });
        }
    }

    fn update_divers(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, height: u32) {
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
                self.bombs.push(Projectile { x: enemy.x + 16.0, y: enemy.y + 20.0, vx: vx * 0.4, active: true });
                diver.bomb_dropped = true;
            }

            if enemy.y > height as f32 || enemy.x < -100.0 || enemy.x > width as f32 + 100.0 {
                enemy.is_diving = false;
                enemy.y = -200.0;
                enemy.x = formation_center_x + diver.start_x;
                return false;
            }
            true
        });
    }

    fn move_in_formation(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
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
            let drop_amount = 30.0;

            for enemy in enemies.iter_mut() {
                enemy.target_y += drop_amount;
            }
        }
    }
}
