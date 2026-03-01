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

pub mod beam;
pub mod enemy;
pub mod particle;
pub mod ship;
pub mod projectile;

pub trait Collidable {
    fn pos(&self) -> (u32, i32);
    fn set_active(&mut self, active: bool);
    fn is_active(&self) -> bool;

    // Default implementation for AABB collision
    fn collides_with<T: Collidable>(&self, other: &T, self_s: &Sprite, other_s: &Sprite) -> bool {
        let (ax, ay) = self.pos();
        let (bx, by) = other.pos();

        ax < bx + other_s.width
            && ax + self_s.width > bx
            && ay < by + other_s.height as i32
            && ay + self_s.height as i32 > by
    }
}

pub struct Sprite {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
