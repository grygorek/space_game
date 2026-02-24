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
        let mut hit_edge = false;
        let margin = 20;

        // Calculate speed scaling based on remaining enemies
        let alive_count = enemies.iter().filter(|e| e.active).count();
        let kill_progress = 1.0 - (alive_count as f32 / enemies.len() as f32);
        
        self.idle_timer += dt;
        let idle_boost = (self.idle_timer / 5.0).floor() * 0.1;
        
        let magnitude = (self.speed * (1.0 + kill_progress + idle_boost)).min(self.max_speed);
        let current_speed = magnitude * self.direction;

        // Move and Check Edges
        for enemy in enemies.iter_mut().filter(|e| e.active) {
            enemy.remain_x += current_speed * dt;
            let move_x = enemy.remain_x as i32;
            enemy.x = (enemy.x as i32 + move_x).max(0) as u32;
            enemy.remain_x -= move_x as f32;

            if (enemy.x <= margin && self.direction < 0.0) || 
               (enemy.x + sprite.width >= width - margin && self.direction > 0.0) {
                hit_edge = true;
            }
        }

        if hit_edge {
            self.direction *= -1.0;
            for enemy in enemies.iter_mut() {
                enemy.y += self.drop_dist;
                // Tiny push to prevent edge-snagging
                let push = (self.direction * 5.0) as i32;
                enemy.x = (enemy.x as i32 + push).max(0) as u32;
                enemy.remain_x = 0.0;
            }
        }
    }

    pub fn deploy(&self, _width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let (cols, rows, spacing) = (10, 5, 100);
        for row in 0..rows {
            for col in 0..cols {
                enemies.push(Enemy {
                    x: col * spacing + 100,
                    y: row * spacing + 100,
                    active: true,
                    remain_x: 0.0,
                    sprite_idx: 2,
                });
            }
        }
        enemies
    }
}