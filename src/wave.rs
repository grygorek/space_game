use crate::entities::enemy::Enemy;
use crate::entities::Sprite;

pub struct Wave {
    pub count: u32,
}

impl Wave {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn deploy(
        &mut self,
        screen_width: u32,
        screen_height: u32,
        enemy_sprite: &Sprite,
    ) -> Vec<Enemy> {
        self.count += 1;

        // Pattern logic: 8 columns, rows grow with wave count
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
                    y: (screen_height / 6) + (r * (enemy_sprite.height + gap_y)),
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
