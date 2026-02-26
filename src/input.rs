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

use std::collections::HashSet;
use winit::event::{ElementState, VirtualKeyCode, WindowEvent};

pub struct InputState {
    pressed_keys: HashSet<VirtualKeyCode>,
    just_pressed: HashSet<VirtualKeyCode>,
}

impl InputState {
    pub fn new() -> Self {
        Self { pressed_keys: HashSet::new(), just_pressed: HashSet::new() }
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
