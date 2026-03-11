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

use crate::entities::Sprite;

// --- Private Raw Data ---
static SHIP_PNG: &[u8] = include_bytes!("../png/ship.png");
static BEAM_PNG: &[u8] = include_bytes!("../png/beam.png");
static ENEMY1_PNG: &[u8] = include_bytes!("../png/enemy1.png");
static BOMB_PNG: &[u8] = include_bytes!("../png/bomb.png");

pub static SFX_SHOT: &[u8] = include_bytes!("../sfx/laser1.wav");
pub static SFX_EXPLOSION: &[u8] = include_bytes!("../sfx/explosion.wav");
pub static SFX_OVERHEAT: &[u8] = include_bytes!("../sfx/Metal_Click.wav");

pub struct Assets {
    pub sprites: Vec<Sprite>,
    pub sfx_shot: &'static [u8],
    pub sfx_explosion: &'static [u8],
    pub sfx_overheat: &'static [u8],
}

impl Assets {
    pub fn load() -> Self {
        let mut sprites = Vec::new();
        let image_data = [SHIP_PNG, BEAM_PNG, ENEMY1_PNG, BOMB_PNG];

        for (i, data) in image_data.iter().enumerate() {
            let img = image::load_from_memory(data).expect("Asset load error");
            let (w, h, pix) = if i == 3 {
                // Bomb resize logic
                let res = img.resize(img.width() / 20, img.height() / 20, image::imageops::FilterType::Nearest);
                (res.width(), res.height(), res.to_rgba8().into_raw())
            } else {
                (img.width(), img.height(), img.to_rgba8().into_raw())
            };
            sprites.push(Sprite { width: w, height: h, pixels: pix });
        }

        Self { sprites, sfx_shot: SFX_SHOT, sfx_explosion: SFX_EXPLOSION, sfx_overheat: SFX_OVERHEAT }
    }

    pub fn ship(&self) -> &Sprite {
        &self.sprites[0]
    }
    pub fn beam(&self) -> &Sprite {
        &self.sprites[1]
    }
    pub fn enemy(&self) -> &Sprite {
        &self.sprites[2]
    }
    pub fn bomb(&self) -> &Sprite {
        &self.sprites[3]
    }
}
