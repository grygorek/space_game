use crate::drawing::set_pixel;
use winit::dpi::PhysicalSize;

pub const MAX_STARS: usize = 20;

#[derive(Clone)]
pub struct Star {
    pub x: f32, // Changed to f32 for smooth movement
    pub y: f32,
    pub diameter: u32,
    pub color: [u8; 4],
}

impl Star {
    /// Creates a new star with randomized properties
    pub fn new_random(rng: &mut SimpleRng, width: u32, height: u32, start_anywhere: bool) -> Self {
        let diameter = (rng.next_u32() % 4) + 1; // 1 to 4 pixels
        let x = (rng.next_u32() % width) as f32;

        // If start_anywhere is true, spawn anywhere (initial load).
        // If false, spawn at the top (respawning after falling off).
        let y = if start_anywhere {
            (rng.next_u32() % height) as f32
        } else {
            -(diameter as f32)
        };

        let choice = rng.next_u32() % 4;
        let alpha = 100 + (rng.next_u32() % 156) as u8;
        let color = match choice {
            0 => [150, 180, 255, alpha],
            1 => [200, 200, 220, alpha],
            2 => [255, 230, 160, alpha],
            3 => [255, 150, 150, alpha],
            _ => [255, 255, 255, alpha],
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

pub fn draw_star(frame: &mut [u8], frame_width: u32, star: &Star) {
    let cx = star.x as i32;
    let cy = star.y as i32;
    let size = star.diameter as i32;

    // Boundary check for the center point
    if cx < 0 || cx >= frame_width as i32 || cy < 0 || cy >= 1080 {
        return;
    }

    // High speed path: 1x1 stars
    if size <= 1 {
        set_pixel(frame, frame_width, cx as u32, cy as u32, star.color);
        return;
    }

    // Performance path: Draw small squares instead of circles
    // This avoids the expensive math in the inner loop
    for dy in 0..size {
        for dx in 0..size {
            let sx = cx + dx;
            let sy = cy + dy;

            // Basic bounds check
            if sx >= 0 && sx < frame_width as i32 && sy >= 0 && sy < 1080 {
                set_pixel(frame, frame_width, sx as u32, sy as u32, star.color);
            }
        }
    }
}

pub fn update_stars(stars: &mut [Star], rng: &mut SimpleRng, size: PhysicalSize<u32>, dt: f32) {
    for star in stars.iter_mut() {
        // PARALLAX: speed is based on diameter.
        // Small stars = far away = slow. Large stars = close = fast.
        let speed = (star.diameter as f32 * 50.0) + 20.0;
        star.y += speed * dt;

        if star.y > size.height as f32 {
            *star = Star::new_random(rng, size.width, size.height, false);
        }
    }
}

// Simple xorshift RNG to avoid adding dependencies
pub struct SimpleRng(u64);
impl SimpleRng {
    pub fn seed_from_instant() -> Self {
        // Use system time since UNIX_EPOCH to create a seed
        let seed = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_nanos() as u64,
            Err(_) => 0u64,
        };
        SimpleRng(seed.wrapping_add(0x9E3779B97F4A7C15))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
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
