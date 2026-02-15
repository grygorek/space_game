pub struct Beam {
    pub x: u32,
    pub y: i32,
    pub remain_y: f32,
}

pub struct Ship {
    pub x: u32,
    pub y: u32,
    pub speed: f32,
    pub remain_x: f32,
    pub remain_y: f32,
}

pub struct Enemy {
    pub x: u32,
    pub y: u32,
    pub active: bool,
}
