use crate::entities::enemy::Enemy;
use crate::entities::Sprite;

pub struct SwoopWave {
    pub timer: f32,
    pub center_y: f32,
}

impl SwoopWave {
    pub fn new() -> Self {
        Self { timer: 0.0, center_y: 550.0 }
    }

    pub fn update(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, _sprite: &Sprite) {
        self.timer += dt;

        for (i, enemy) in enemies.iter_mut().filter(|e| e.active).enumerate() {
            // Give each enemy a slightly different start time (offset)
            let offset = i as f32 * 0.4;
            let t = self.timer + offset;

            // Figure-eight / Infinity pattern math
            let x_pos = (width as f32 / 2.0) + t.cos() * 400.0;
            let y_pos = self.center_y + (t * 2.0).sin() * 300.0;

            enemy.x = x_pos as u32;
            enemy.y = y_pos as u32;
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        for i in 0..8 {
            enemies.push(Enemy { x: (width / 2) - 160 + (i * 40), y: 100, active: true, remain_x: 0.0, sprite_idx: 2 });
        }
        enemies
    }
}
