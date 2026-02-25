pub const COLOR_RED: [u8; 4] = [255, 0, 0, 255];
pub const COLOR_WHITE: [u8; 4] = [255, 255, 255, 255];
pub const COLOR_GREEN: [u8; 4] = [0, 255, 0, 255];

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
    frame_height: u32,
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
            if tx < 0 || tx >= frame_width as i32 || ty < 0 || ty >= frame_height as i32 {
                continue;
            }

            let src_idx = ((row * sprite_w + col) * 4) as usize;
            let src_alpha = sprite_pixels[src_idx + 3];

            // 2. Optimization: Skip fully transparent pixels
            if src_alpha == 0 {
                continue;
            }

            let dst_idx = ((ty * frame_width as i32 + tx) * 4) as usize;

            // Defensive check to ensure we don't write out of bounds
            if dst_idx + 4 > frame.len() {
                continue;
            }

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

// Simple 8x8 bitmapped letters (1 = pixel, 0 = empty)
const FONT_S: [u8; 8] = [0x3C, 0x42, 0x40, 0x3C, 0x02, 0x42, 0x3C, 0x00];
const FONT_C: [u8; 8] = [0x3C, 0x42, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00];
const FONT_O: [u8; 8] = [0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00];
const FONT_R: [u8; 8] = [0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x00];
const FONT_E: [u8; 8] = [0x7E, 0x40, 0x40, 0x78, 0x40, 0x40, 0x7E, 0x00];
const FONT_G: [u8; 8] = [0x3C, 0x42, 0x40, 0x5E, 0x42, 0x42, 0x3C, 0x00];
const FONT_A: [u8; 8] = [0x18, 0x24, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00];
const FONT_M: [u8; 8] = [0x42, 0x66, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00];
const FONT_V: [u8; 8] = [0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x18, 0x00];

const FONT_0: [u8; 8] = [0x3C, 0x46, 0x4A, 0x52, 0x62, 0x42, 0x3C, 0x00];
const FONT_1: [u8; 8] = [0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00];
const FONT_2: [u8; 8] = [0x3C, 0x42, 0x02, 0x3C, 0x40, 0x40, 0x7E, 0x00];
const FONT_3: [u8; 8] = [0x3C, 0x42, 0x02, 0x1C, 0x02, 0x42, 0x3C, 0x00];
const FONT_4: [u8; 8] = [0x08, 0x18, 0x28, 0x48, 0x7E, 0x08, 0x08, 0x00];
const FONT_5: [u8; 8] = [0x7E, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00];
const FONT_6: [u8; 8] = [0x3C, 0x40, 0x40, 0x7C, 0x42, 0x42, 0x3C, 0x00];
const FONT_7: [u8; 8] = [0x7E, 0x02, 0x04, 0x08, 0x10, 0x10, 0x10, 0x00];
const FONT_8: [u8; 8] = [0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x3C, 0x00];
const FONT_9: [u8; 8] = [0x3C, 0x42, 0x42, 0x3E, 0x02, 0x02, 0x3C, 0x00];
const FONT_COLON: [u8; 8] = [0x00, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00];

pub fn draw_text_centered(frame: &mut [u8], width: u32, height: u32, text: &str, scale: u32, color: [u8; 4]) {
    let char_w = 8 * scale;
    let spacing = scale;
    let total_w = (text.len() as u32 * char_w) + ((text.len() as u32 - 1) * spacing);

    let start_x = (width.saturating_sub(total_w)) / 2;
    let start_y = (height.saturating_sub(8 * scale)) / 2;

    draw_text(frame, width, height, start_x, start_y, text, scale, color);
}

/// Generic text drawing function
pub fn draw_text(frame: &mut [u8], width: u32, height: u32, x: u32, y: u32, text: &str, scale: u32, color: [u8; 4]) {
    for (i, c) in text.chars().enumerate() {
        let glyph = get_glyph(c.to_ascii_uppercase());
        let x_offset = x + (i as u32 * (8 * scale + scale));

        for row in 0..8 {
            for col in 0..8 {
                if (glyph[row] & (0x80 >> col)) != 0 {
                    for py in 0..scale {
                        for px in 0..scale {
                            let tx = x_offset + (col as u32 * scale) + px;
                            let ty = y + (row as u32 * scale) + py;
                            if tx < width && ty < height {
                                set_pixel(frame, width, tx, ty, color);
                            }
                        }
                    }
                }
            }
        }
    }
}

// Helper to map characters to your FONT_ constants
fn get_glyph(c: char) -> &'static [u8; 8] {
    match c {
        'G' => &FONT_G,
        'A' => &FONT_A,
        'M' => &FONT_M,
        'E' => &FONT_E,
        'O' => &FONT_O,
        'V' => &FONT_V,
        'R' => &FONT_R,
        'S' => &FONT_S,
        'C' => &FONT_C,
        '0' => &FONT_0,
        '1' => &FONT_1,
        '2' => &FONT_2,
        '3' => &FONT_3,
        '4' => &FONT_4,
        '5' => &FONT_5,
        '6' => &FONT_6,
        '7' => &FONT_7,
        '8' => &FONT_8,
        '9' => &FONT_9,
        ':' => &FONT_COLON,
        _ => &[0x00; 8],
    }
}
