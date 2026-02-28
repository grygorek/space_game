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

            enemy.x = x_pos as f32;
            enemy.y = y_pos as f32;
        }
    }

    pub fn deploy(&self, width: u32) -> Vec<Enemy> {
        let mut enemies = Vec::new();
        let width_f = width as f32;

        for i in 0..8 {
            let i_f = i as f32;
            let target_y = 100.0; // Swoop waves usually have a target formation height too

            enemies.push(Enemy {
                x: (width_f / 2.0) - 160.0 + (i_f * 40.0),
                y: -50.0, // Start slightly off-screen for a smooth swoop in
                target_y, // Make sure your Swoop Enemy also has a target_y!
                active: true,
                remain_x: 0.0,
                sprite_idx: 2,
            });
        }
        enemies
    }
}
