use crate::drawing::set_pixel;
use winit::dpi::PhysicalSize;

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
        let y = if start_anywhere {
            (rng.next_u32() % height) as f32
        } else {
            -(diameter as f32)
        };

        let alpha = 100 + (rng.next_u32() % 156) as u8;

        let color = match diameter {
            1 => [150, 150, 220, alpha],
            2 => [200, 200, 255, alpha],
            3 => [255, 255, 255, alpha],
            _ => [255, 240, 150, alpha],
        };

        Star {
            x,
            y,
            diameter,
            color,
        }
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

// --- Utility: Xorshift Random Number Generator ---

#[derive(Clone, Copy)]
pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn seed_from_instant() -> Self {
        let seed = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_nanos() as u64,
            Err(_) => 0u64,
        };
        SimpleRng(seed.wrapping_add(0x9E3779B97F4A7C15))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        // Ensure x is never 0 for Xorshift
        if x == 0 {
            x = 0x9E3779B97F4A7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() & 0xFFFF_FFFF) as u32
    }
}
