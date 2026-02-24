pub mod classic;
pub mod swoop;

use crate::entities::enemy::Enemy;
use crate::entities::Sprite;

pub enum WaveType {
    Classic(classic::ClassicWave),
    Swoop(swoop::SwoopWave),
}

impl WaveType {
    pub fn update(&mut self, enemies: &mut Vec<Enemy>, dt: f32, width: u32, sprite: &Sprite) {
        match self {
            WaveType::Classic(w) => w.update(enemies, dt, width, sprite),
            WaveType::Swoop(s) => s.update(enemies, dt, width, sprite),
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        match self {
            WaveType::Classic(w) => w.deploy(width),
            WaveType::Swoop(s) => s.deploy(width),
        }
    }

    pub fn is_extinct(&self, enemies: &[Enemy]) -> bool {
        enemies.iter().all(|e| !e.active)
    }
}