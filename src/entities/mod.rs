pub mod beam;
pub mod enemy;
pub mod particle;
pub mod ship;

pub trait Collidable {
    fn pos(&self) -> (u32, i32);
    fn set_active(&mut self, active: bool);
    fn is_active(&self) -> bool;
}

pub struct Sprite {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
