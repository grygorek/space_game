pub mod beam;
pub mod enemy;
pub mod particle;
pub mod ship;

pub trait Collidable {
    fn pos(&self) -> (u32, i32);
    fn set_active(&mut self, active: bool);
    fn is_active(&self) -> bool;

    // Default implementation for AABB collision
    fn collides_with<T: Collidable>(&self, other: &T, self_s: &Sprite, other_s: &Sprite) -> bool {
        let (ax, ay) = self.pos();
        let (bx, by) = other.pos();

        ax < bx + other_s.width
            && ax + self_s.width > bx
            && ay < by + other_s.height as i32
            && ay + self_s.height as i32 > by
    }
}

pub struct Sprite {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
