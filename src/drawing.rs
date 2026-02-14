// Utilities for drawing pixels and sprites (alpha blending)

/// Set pixel without blending (overwrite)
pub fn set_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, color: [u8; 4]) {
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 < frame.len() {
        frame[idx..idx + 4].copy_from_slice(&color);
    }
}

/// Blend src RGBA over destination pixel at (x,y) in frame.
pub fn blend_pixel(frame: &mut [u8], frame_width: u32, x: u32, y: u32, src: [u8; 4]) {
    let idx = ((y * frame_width + x) * 4) as usize;
    if idx + 3 >= frame.len() {
        return;
    }
    let dst_r = frame[idx] as f32;
    let dst_g = frame[idx + 1] as f32;
    let dst_b = frame[idx + 2] as f32;
    let dst_a = frame[idx + 3] as f32 / 255.0;

    let src_r = src[0] as f32;
    let src_g = src[1] as f32;
    let src_b = src[2] as f32;
    let src_a = src[3] as f32 / 255.0;

    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        frame[idx] = 0;
        frame[idx + 1] = 0;
        frame[idx + 2] = 0;
        frame[idx + 3] = 0;
        return;
    }

    let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a;
    let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a;
    let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a;

    frame[idx] = out_r as u8;
    frame[idx + 1] = out_g as u8;
    frame[idx + 2] = out_b as u8;
    frame[idx + 3] = (out_a * 255.0) as u8;
}

/// Draw a sprite (RGBA8) at top-left position (dst_x, dst_y).
/// Coordinates are allowed to be negative to support partial offscreen rendering.
pub fn draw_sprite(
    frame: &mut [u8],
    frame_width: u32,
    dst_x: i32,
    dst_y: i32,
    sprite_pixels: &[u8],
    sprite_w: u32,
    sprite_h: u32,
) {
    for yy in 0..(sprite_h as i32) {
        for xx in 0..(sprite_w as i32) {
            let px = dst_x + xx;
            let py = dst_y + yy;
            if px < 0 || py < 0 {
                continue;
            }
            let pxu = px as u32;
            let pyu = py as u32;
            // check bounds
            // Note: we can't access frame size here, so the caller must ensure bounds or pass frame size separately.
            // We'll assume caller checks bounds against frame size before calling, or the blend_pixel will check.
            let sidx = ((yy as u32 * sprite_w + xx as u32) * 4) as usize;
            if sidx + 3 >= sprite_pixels.len() {
                continue;
            }
            let src = [
                sprite_pixels[sidx],
                sprite_pixels[sidx + 1],
                sprite_pixels[sidx + 2],
                sprite_pixels[sidx + 3],
            ];
            blend_pixel(frame, frame_width, pxu, pyu, src);
        }
    }
}
