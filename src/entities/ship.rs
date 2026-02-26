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

use crate::entities::Collidable;
use crate::input::InputState;
use winit::dpi::PhysicalSize;

pub struct Ship {
    pub x: u32,
    pub y: u32,
    pub speed: f32,
    pub remain_x: f32,
    pub remain_y: f32,
    pub sprite_idx: usize,
    pub active: bool,

    // Gun overheat system
    pub heat: f32,           // 0.0 to 1.0 (0% to 100%)
    pub is_overheated: bool, // True if we hit 1.0 and are waiting to hit 0.5
}

impl Ship {
    pub fn update(&mut self, input: &InputState, size: PhysicalSize<u32>, sprite_w: u32, _sprite_h: u32, dt: f32) {
        if !self.active {
            return;
        }

        let cooling_rate = 0.2;
        self.heat = (self.heat - cooling_rate * dt).max(0.0);

        if self.is_overheated && self.heat == 0.0 {
            self.is_overheated = false;
        }

        let mut dx = 0.0;
        if input.is_key_down(winit::event::VirtualKeyCode::Left) {
            dx -= self.speed * dt;
        }
        if input.is_key_down(winit::event::VirtualKeyCode::Right) {
            dx += self.speed * dt;
        }

        self.remain_x += dx;
        let move_x = self.remain_x as i32;

        let new_x = self.x as i32 + move_x;
        if new_x >= 0 && new_x <= (size.width - sprite_w) as i32 {
            self.x = new_x as u32;
            self.remain_x -= move_x as f32;
        } else {
            self.remain_x = 0.0;
        }
    }

    pub fn try_fire(&mut self) -> bool {
        if !self.is_overheated && self.active {
            self.heat += 0.15;
            if self.heat >= 1.0 {
                self.heat = 1.0;
                self.is_overheated = true;
            }
            return true; // "Yes, I fired a shot"
        }
        false // "No, I'm overheated or dead"
    }
}

impl Collidable for Ship {
    fn pos(&self) -> (u32, i32) {
        (self.x, self.y as i32)
    }
    fn is_active(&self) -> bool {
        self.active
    }
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}
