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
