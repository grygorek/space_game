use crate::entities::enemy::Enemy;
use crate::entities::Sprite;

pub struct Wave {
    pub count: u32,
    pub direction: f32, // 1.0 for right, -1.0 for left
    pub move_speed: f32,
    pub drop_distance: i32,
    pub idle_timer: f32,
}

impl Wave {
    pub fn new() -> Self {
        Self {
            count: 0,
            direction: 1.0,
            move_speed: 200.0,
            drop_distance: 20,
            idle_timer: 0.0,
        }
    }

    pub fn deploy(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        enemy_sprite: &Sprite,
    ) -> Vec<Enemy> {
        self.count += 1;
        self.direction = 1.0; // Reset direction for new wave
        self.idle_timer = 0.0;

        // Base speed increases with each wave
        self.move_speed = 200.0 + (self.count as f32 * 30.0);

        let cols = 8;
        let rows = (2 + self.count).min(6);

        let gap_x = (enemy_sprite.width as f32 * 1.5) as u32;
        let gap_y = (enemy_sprite.height as f32 * 1.5) as u32;

        let grid_w = (cols * enemy_sprite.width) + ((cols - 1) * gap_x);
        let start_x = (screen_width.saturating_sub(grid_w)) / 2;

        let mut enemies = Vec::with_capacity((rows * cols) as usize);
        for r in 0..rows {
            for c in 0..cols {
                enemies.push(Enemy {
                    x: start_x + (c * (enemy_sprite.width + gap_x)),
                    y: (screen_height / 8) + (r * (enemy_sprite.height + gap_y)),
                    remain_x: 0.0,
                    active: true,
                    sprite_idx: 2,
                });
            }
        }
        enemies
    }

    pub fn is_extinct(&self, enemies: &[Enemy]) -> bool {
        !enemies.iter().any(|e| e.active)
    }
}
