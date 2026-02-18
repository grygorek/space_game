use crate::input::InputState;
use winit::dpi::PhysicalSize;
use winit::event::VirtualKeyCode;

pub struct Ship {
    pub x: u32,
    pub y: u32,
    pub speed: f32,
    pub remain_x: f32,
    pub remain_y: f32,
    pub sprite_idx: usize,
}

impl Ship {
    pub fn update(
        &mut self,
        input: &InputState,
        size: PhysicalSize<u32>,
        sw: u32,
        sh: u32,
        dt: f32,
    ) {
        let mut mx = 0.0;
        let mut my = 0.0;
        if input.is_key_down(VirtualKeyCode::Left) {
            mx -= self.speed * dt;
        }
        if input.is_key_down(VirtualKeyCode::Right) {
            mx += self.speed * dt;
        }
        if input.is_key_down(VirtualKeyCode::Up) {
            my -= self.speed * dt;
        }
        if input.is_key_down(VirtualKeyCode::Down) {
            my += self.speed * dt;
        }

        self.remain_x += mx;
        self.remain_y += my;
        let dx = self.remain_x as i32;
        let dy = self.remain_y as i32;

        self.x = (self.x as i32 + dx).clamp(0, (size.width - sw) as i32) as u32;
        self.y = (self.y as i32 + dy).clamp(0, (size.height - sh) as i32) as u32;

        self.remain_x -= dx as f32;
        self.remain_y -= dy as f32;
    }
}
