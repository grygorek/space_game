use crate::entities::Collidable;
use crate::input::InputState;
use winit::dpi::PhysicalSize;

pub struct Ship {
    pub x: u32,
    pub y: u32,
    pub speed: f32,
    pub remain_x: f32,
    pub remain_y: f32,
    pub sprite_idx: usize,
    pub active: bool,
}

impl Ship {
    pub fn update(&mut self, input: &InputState, size: PhysicalSize<u32>, sprite_w: u32, _sprite_h: u32, dt: f32) {
        if !self.active {
            return;
        }

        let mut dx = 0.0;
        if input.is_key_down(winit::event::VirtualKeyCode::Left) {
            dx -= self.speed * dt;
        }
        if input.is_key_down(winit::event::VirtualKeyCode::Right) {
            dx += self.speed * dt;
        }

        self.remain_x += dx;
        let move_x = self.remain_x as i32;

        let new_x = self.x as i32 + move_x;
        if new_x >= 0 && new_x <= (size.width - sprite_w) as i32 {
            self.x = new_x as u32;
            self.remain_x -= move_x as f32;
        } else {
            self.remain_x = 0.0;
        }
    }
}

impl Collidable for Ship {
    fn pos(&self) -> (u32, i32) {
        (self.x, self.y as i32)
    }
    fn is_active(&self) -> bool {
        self.active
    }
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}
