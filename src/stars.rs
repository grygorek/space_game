use winit::dpi::PhysicalSize;

pub const MAX_STARS: usize = 20;

#[derive(Clone)]
pub struct Star {
    pub x: u32,
    pub y: u32,
    pub diameter: u32,
    pub color: [u8; 4], // RGBA
}

pub fn generate_stars(rng: &mut SimpleRng, size: PhysicalSize<u32>) -> Vec<Star> {
    let count = (rng.next_u32() as usize % (MAX_STARS + 1)).max(5); // between 5 and MAX_STARS
    let mut stars = Vec::with_capacity(count);
    for _ in 0..count {
        let diameter = (rng.next_u32() % 5) + 1; // 1..5
        let radius = diameter / 2;
        let x = (rng.next_u32() % (size.width.saturating_sub(diameter) + 1)) + radius;
        let y = (rng.next_u32() % (size.height.saturating_sub(diameter) + 1)) + radius;

        // Color palettes: bluish, gray, yellowish, reddish
        let choice = rng.next_u32() % 4;
        let alpha = 100 + (rng.next_u32() % 156) as u8; // 100..255
        let color = match choice {
            0 => [150, 180, 255, alpha], // bluish
            1 => [200, 200, 220, alpha], // grayish
            2 => [255, 230, 160, alpha], // yellowish
            3 => [255, 150, 150, alpha], // reddish
            _ => [200, 200, 200, alpha],
        };

        stars.push(Star { x, y, diameter, color });
    }
    stars
}

pub fn draw_star(frame: &mut [u8], frame_width: u32, star: &Star) {
    let r = (star.diameter as i32) / 2;
    let cx = star.x as i32;
    let cy = star.y as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let sx = cx + dx;
            let sy = cy + dy;
            if sx < 0 || sy < 0 {
                continue;
            }
            // circle mask
            if dx * dx + dy * dy <= r * r {
                set_pixel(frame, frame_width, sx as u32, sy as u32, star.color);
            }
        }
    }
}

// Moves stars down and resets them when they pass the bottom.
pub fn update_stars(stars: &mut [Star], rng: &mut SimpleRng, size: PhysicalSize<u32>) {
    for star in stars.iter_mut() {
        // speed depends on diameter: larger stars fall slightly faster
        let speed = (star.diameter / 2).max(1);
        star.y = star.y.saturating_add(speed);
        if star.y > size.height {
            // reset to top with new random properties
            star.diameter = (rng.next_u32() % 5) + 1;
            let radius = star.diameter / 2;
            // avoid underflow if diameter > width
            if size.width > star.diameter {
                star.x = (rng.next_u32() % (size.width.saturating_sub(star.diameter) + 1)) + radius;
            } else {
                star.x = 0;
            }
            star.y = 0;
            let choice = rng.next_u32() % 4;
            let alpha = 100 + (rng.next_u32() % 156) as u8; // 100..255
            star.color = match choice {
                0 => [150, 180, 255, alpha], // bluish
                1 => [200, 200, 220, alpha], // grayish
                2 => [255, 230, 160, alpha], // yellowish
                3 => [255, 150, 150, alpha], // reddish
                _ => [200, 200, 200, alpha],
            };
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

pub fn set_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 < frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}
