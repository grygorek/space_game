use super::Collidable;

pub struct Enemy {
    pub x: u32,
    pub y: u32,
    pub active: bool,
    pub remain_x: f32,
    pub sprite_idx: usize,
}

impl Collidable for Enemy {
    fn pos(&self) -> (u32, i32) {
        (self.x, self.y as i32)
    }
    fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    fn is_active(&self) -> bool {
        self.active
    }
}
