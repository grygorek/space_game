use std::collections::HashSet;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

pub struct InputState {
    pressed_keys: HashSet<VirtualKeyCode>,
    just_pressed: HashSet<VirtualKeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            just_pressed: HashSet::new(),
        }
    }

    pub fn update(&mut self, event: &WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            if let Some(key) = input.virtual_keycode {
                match input.state {
                    ElementState::Pressed => {
                        if !self.pressed_keys.contains(&key) {
                            self.just_pressed.insert(key);
                        }
                        self.pressed_keys.insert(key);
                    }
                    ElementState::Released => {
                        self.pressed_keys.remove(&key);
                    }
                }
            }
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn was_key_pressed(&mut self, key: VirtualKeyCode) -> bool {
        self.just_pressed.remove(&key)
    }

    pub fn clear_just_pressed(&mut self) {
        self.just_pressed.clear();
    }
}
