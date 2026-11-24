use std::collections::HashMap;
use std::collections::hash_map::Entry;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::PhysicalKey;

pub struct Inputs {
    keyboard: HashMap<PhysicalKey, KeyState<ElementState>>,
    mouse: HashMap<MouseButton, KeyState<ElementState>>,
}

pub struct KeyState<T> {
    pub(crate) last_state: T,
    pub(crate) current_state: T,
}

impl Inputs {
    pub fn new() -> Self {
        Self { keyboard: HashMap::new(), mouse: HashMap::new() }
    }

    /// Update the mouse buttons buffer with window events
    pub fn button_callback(&mut self, mouse_button: MouseButton, element_state: ElementState) {
        match self.mouse.entry(mouse_button) {
            Entry::Occupied(mut e) => {
                e.get_mut().last_state = e.get_mut().current_state;
                e.get_mut().current_state = element_state;
            }
            Entry::Vacant(e) => {
                e.insert(KeyState {
                    last_state: element_state,
                    current_state: element_state,
                });
            }
        }
    }

    /// Marks every key and button as released, e.g. when the game stops receiving the inputs.
    pub fn release_all(&mut self) {
        for state in self.keyboard.values_mut().chain(self.mouse.values_mut()) {
            state.last_state = ElementState::Released;
            state.current_state = ElementState::Released;
        }
    }

    pub fn get_keyboard(&mut self) -> &mut HashMap<PhysicalKey, KeyState<ElementState>> {
        &mut self.keyboard
    }

    pub fn get_mouse(&mut self) -> &mut HashMap<MouseButton, KeyState<ElementState>> {
        &mut self.mouse
    }

    /// Update the keyboard buffer with window events
    pub fn keyboard_callback(&mut self, input: KeyEvent) {
        match self.keyboard.entry(input.physical_key) {
            Entry::Occupied(mut e) => {
                e.get_mut().last_state = e.get_mut().current_state;
                e.get_mut().current_state = input.state;
            }
            Entry::Vacant(e) => {
                e.insert(KeyState {
                    last_state: ElementState::Released,
                    current_state: input.state,
                });
            }
        }
    }
}

impl Default for Inputs {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for KeyState<ElementState> {
    fn clone(&self) -> KeyState<ElementState> {
        Self {
            current_state: self.current_state,
            last_state: self.last_state,
        }
    }
}
