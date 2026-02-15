pub fn set_pixel(frame: &mut [u8], width: u32, x: u32, y: u32, color: [u8; 4]) {
    let index = ((y * width + x) * 4) as usize;
    if index + 4 <= frame.len() {
        // Using copy_from_slice is often faster than 4 manual assignments
        frame[index..index + 4].copy_from_slice(&color);
    }
}

pub fn draw_sprite(
    frame: &mut [u8],
    frame_width: u32,
    dest_x: i32,
    dest_y: i32,
    sprite_pixels: &[u8],
    sprite_w: u32,
    sprite_h: u32,
) {
    for row in 0..sprite_h {
        for col in 0..sprite_w {
            let tx = dest_x + col as i32;
            let ty = dest_y + row as i32;

            // 1. Boundary Check (Early Exit)
            if tx < 0 || tx >= frame_width as i32 || ty < 0 || ty >= 1080 {
                continue;
            }

            let src_idx = ((row * sprite_w + col) * 4) as usize;
            let src_alpha = sprite_pixels[src_idx + 3];

            // 2. Optimization: Skip fully transparent pixels
            if src_alpha == 0 {
                continue;
            }

            let dst_idx = ((ty * frame_width as i32 + tx) * 4) as usize;

            // 3. Optimization: Skip blending for fully opaque pixels
            if src_alpha == 255 {
                frame[dst_idx..dst_idx + 4].copy_from_slice(&sprite_pixels[src_idx..src_idx + 4]);
            } else {
                // 4. Fast Alpha Blending (using bit-shifting instead of division)
                // (source * alpha + destination * (255 - alpha)) / 256
                for i in 0..3 {
                    let src = sprite_pixels[src_idx + i] as u32;
                    let dst = frame[dst_idx + i] as u32;
                    let alpha = src_alpha as u32;

                    frame[dst_idx + i] = ((src * alpha + dst * (255 - alpha)) >> 8) as u8;
                }
                frame[dst_idx + 3] = 255; // Keep the screen opaque
            }
        }
    }
}
