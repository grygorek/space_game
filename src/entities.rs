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

pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32, // Remaining time to live in seconds
}
