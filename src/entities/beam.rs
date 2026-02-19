use crate::entities::Collidable;

pub struct Beam {
    pub x: u32,
    pub y: i32,
    pub remain_y: f32,
    pub sprite_idx: usize,
}

impl Collidable for Beam {
    fn pos(&self) -> (u32, i32) {
        (self.x, self.y)
    }

    fn is_active(&self) -> bool {
        self.y > -100 // Active as long as it hasn't flown too far off screen
    }

    fn set_active(&mut self, active: bool) {
        if !active {
            self.y = -1000; // The "off-screen kill" trick
        }
    }
}
