use crate::drawing::set_pixel;
use winit::dpi::PhysicalSize;

pub const MAX_STARS: usize = 120; // Adjusted for a good balance of density and performance

#[derive(Clone)]
pub struct Star {
    pub x: f32,
    pub y: f32,
    pub diameter: u32,
    pub color: [u8; 4],
}

impl Star {
    /// Creates a new star.
    /// If start_anywhere is true, it populates the initial screen.
    /// If false, it spawns just above the top for a continuous flow.
    pub fn new_random(rng: &mut SimpleRng, width: u32, height: u32, start_anywhere: bool) -> Self {
        // Diameter between 1 and 4 pixels
        let diameter = (rng.next_u32() % 4) + 1;

        let x = (rng.next_u32() % width) as f32;
        let y = if start_anywhere {
            (rng.next_u32() % height) as f32
        } else {
            -(diameter as f32)
        };

        let alpha = 100 + (rng.next_u32() % 156) as u8;

        // Color variation: Bigger stars are slightly warmer/brighter
        let color = match diameter {
            1 => [150, 150, 220, alpha], // Small: Dim Blue
            2 => [200, 200, 255, alpha], // Medium-Small: Soft Blue
            3 => [255, 255, 255, alpha], // Medium-Large: Pure White
            _ => [255, 240, 150, alpha], // Large: Yellowish
        };

        Star {
            x,
            y,
            diameter,
            color,
        }
    }
}

/// Populates the initial vector of stars
pub fn generate_stars(rng: &mut SimpleRng, size: PhysicalSize<u32>) -> Vec<Star> {
    let mut stars = Vec::with_capacity(MAX_STARS);
    for _ in 0..MAX_STARS {
        stars.push(Star::new_random(rng, size.width, size.height, true));
    }
    stars
}

/// Move stars down based on their size (Parallax effect)
pub fn update_stars(stars: &mut [Star], rng: &mut SimpleRng, size: PhysicalSize<u32>, dt: f32) {
    for star in stars.iter_mut() {
        // Speed scaling: Larger stars move faster to create depth
        let speed = (star.diameter as f32) * 60.0;
        star.y += speed * dt;

        // If the star moves off the bottom, respawn it at the top
        if star.y > size.height as f32 {
            *star = Star::new_random(rng, size.width, size.height, false);
        }
    }
}

/// Renders the star to the pixel buffer
pub fn draw_star(frame: &mut [u8], frame_width: u32, star: &Star) {
    let cx = star.x as u32;
    let cy = star.y as u32;
    let d = star.diameter;

    // Boundary check for the starting corner
    if cx >= frame_width || cy >= 1080 {
        return;
    }

    // Draw a square based on diameter
    for dy in 0..d {
        for dx in 0..d {
            let px = cx + dx;
            let py = cy + dy;

            // Per-pixel bounds check to prevent memory errors
            if px < frame_width && py < 1080 {
                set_pixel(frame, frame_width, px, py, star.color);
            }
        }
    }
}

// --- Utility: Xorshift Random Number Generator ---

pub struct SimpleRng(u64);

impl SimpleRng {
    pub fn seed_from_instant() -> Self {
        let seed = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(dur) => dur.as_nanos() as u64,
            Err(_) => 0u64,
        };
        // Add a large constant to ensure seed isn't 0
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
