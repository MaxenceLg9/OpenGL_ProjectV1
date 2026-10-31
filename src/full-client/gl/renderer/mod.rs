use std::time::Duration;

use glutin::config::Config;
use glutin::prelude::*;
use glutin_winit::DisplayBuilder;
use winit::event::{DeviceId, KeyEvent};
use winit::window::Window;

pub mod game;
pub mod menu;

pub trait Renderer {
    fn draw(&mut self, window: &Window, redraw_time: Duration);

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    fn tick(&self);

    fn keyboard_input(&self, device_id: DeviceId, event: KeyEvent, is_synthetic: bool);

    fn mouse_input(&self, device_id: DeviceId, state: winit::event::ElementState, button: winit::event::MouseButton);

    fn mouse_wheel(&self, device_id: DeviceId, delta: winit::event::MouseScrollDelta, phase: winit::event::TouchPhase);

    fn mouse_motion(&self, delta: (f64, f64));
}

pub enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(Box<DisplayBuilder>),
    /// The display was already created for the application.
    Init,
}

pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false) & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() { config } else { accum }
        })
        .unwrap()
}
