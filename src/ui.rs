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

use crate::drawing::*;

pub use crate::drawing::{
    COLOR_GRAY_DARK, COLOR_GRAY_LIGHT, COLOR_HEALTH_GREEN, COLOR_HEAT_ORANGE, COLOR_OVERHEAT_RED, COLOR_RED,
    COLOR_SCORE_GOLD, COLOR_WHITE,
};

/// Draws the Start Screen overlay
pub fn draw_start_menu(frame: &mut [u8], w: u32, h: u32) {
    draw_text_centered(frame, w, h, "SPACE GAME", 8, COLOR_SCORE_GOLD);
    draw_text(frame, w, h, (w / 2) - 150, (h / 2) + 100, "PRESS SPACE TO START", 2, COLOR_WHITE);
}

/// Draws the Name Entry screen for high scores
pub fn draw_name_entry_overlay(frame: &mut [u8], w: u32, h: u32, name: &str) {
    draw_text_centered(frame, w, h, "NEW RECORD!", 6, COLOR_SCORE_GOLD);
    let display = format!("{}_", name);
    draw_text(frame, w, h, (w / 2) - 60, (h / 3 * 2) as u32, &display, 8, COLOR_WHITE);
    draw_text(frame, w, h, (w / 2) - 180, (h / 3 * 2) as u32 + 100, "TYPE 3 LETTERS & PRESS ENTER", 2, COLOR_WHITE);
}

/// Draws the Game Over screen and the Top 5 Leaderboard
pub fn draw_game_over_overlay(frame: &mut [u8], w: u32, h: u32, scores: &[(String, u32)]) {
    draw_text_centered(frame, w, h, "GAMEOVER", 10, COLOR_RED);

    let list_start_y = (h as i32 / 3 * 2) - 40;
    draw_text(frame, w, h, (w / 2) - 100, list_start_y as u32, "--- TOP 5 ---", 2, COLOR_SCORE_GOLD);

    for (i, (name, score)) in scores.iter().enumerate() {
        let y_pos = (list_start_y + 40) + (i as i32 * 35);
        let color = if i == 0 { COLOR_SCORE_GOLD } else { COLOR_WHITE };
        let entry = format!("#{} {} .... {:>6}", i + 1, name, score);
        draw_text(frame, w, h, (w / 2) - 130, y_pos as u32, &entry, 2, color);
    }
    draw_text(frame, w, h, (w / 2) - 130, h - 80, "PRESS R TO RESTART", 2, COLOR_WHITE);
}

/// Draws the HUD (Score and Heat Bar)
pub fn draw_hud(frame: &mut [u8], width: u32, height: u32, heat: f32, is_overheated: bool, score: u32) {
    let bar_w = 300;
    let bar_h = 30;
    let x = (width as i32 - bar_w as i32) / 2;
    let y = 20;

    // 1. Draw Outline
    let outline_color = if is_overheated { COLOR_OVERHEAT_RED } else { COLOR_GRAY_LIGHT };
    draw_rect_outline(frame, width, height, x - 4, y - 4, bar_w + 8, bar_h + 8, 4, outline_color);

    // 2. Draw Background
    draw_rect(frame, width, height, x, y, bar_w, bar_h, COLOR_GRAY_DARK);

    // 3. Draw Heat Fill
    let fill_w = (heat * bar_w as f32) as u32;
    let fill_color = if is_overheated {
        COLOR_OVERHEAT_RED
    } else if heat > 0.5 {
        COLOR_HEAT_ORANGE
    } else {
        COLOR_HEALTH_GREEN
    };
    draw_rect(frame, width, height, x, y, fill_w, bar_h, fill_color);

    // 4. Labels
    draw_text(frame, width, height, (width / 2) - 35, (y + bar_h as i32 + 10) as u32, "HEAT", 2, COLOR_WHITE);
    draw_text(frame, width, height, 20, 20, &format!("SCORE: {}", score), 3, COLOR_WHITE);
}
