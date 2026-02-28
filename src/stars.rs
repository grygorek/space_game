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

use crate::drawing::set_pixel;
use winit::dpi::PhysicalSize;
use crate::rng::SimpleRng;

pub const MAX_STARS: usize = 120;

#[derive(Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub diameter: u32,
    pub color: [u8; 4],
}

impl Star {
    pub fn new_random(rng: &mut SimpleRng, width: u32, height: u32, start_anywhere: bool) -> Self {
        let diameter = (rng.next_u32() % 4) + 1;
        let x = (rng.next_u32() % width) as f32;
        let y = if start_anywhere { (rng.next_u32() % height) as f32 } else { -(diameter as f32) };

        let alpha = 100 + (rng.next_u32() % 156) as u8;

        let color = match diameter {
            1 => [150, 150, 220, alpha],
            2 => [200, 200, 255, alpha],
            3 => [255, 255, 255, alpha],
            _ => [255, 240, 150, alpha],
        };

        Star { x, y, diameter, color }
    }
}

pub fn generate_stars(rng: &mut SimpleRng, size: PhysicalSize<u32>) -> Vec<Star> {
    let mut stars = Vec::with_capacity(MAX_STARS);
    for _ in 0..MAX_STARS {
        stars.push(Star::new_random(rng, size.width, size.height, true));
    }
    stars
}

pub fn update_stars(stars: &mut [Star], rng: &mut SimpleRng, size: PhysicalSize<u32>, dt: f32) {
    for star in stars.iter_mut() {
        let speed = (star.diameter as f32) * 60.0;
        star.y += speed * dt;

        if star.y > size.height as f32 {
            *star = Star::new_random(rng, size.width, size.height, false);
        }
    }
}

pub fn draw_star(frame: &mut [u8], frame_width: u32, frame_height: u32, star: &Star) {
    let cx = star.x as u32;
    let cy = star.y as u32;
    let d = star.diameter;

    if cx >= frame_width || cy >= frame_height {
        return;
    }

    for dy in 0..d {
        for dx in 0..d {
            let px = cx + dx;
            let py = cy + dy;

            if px < frame_width && py < frame_height {
                set_pixel(frame, frame_width, px, py, star.color);
            }
        }
    }
}
