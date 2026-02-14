use winit::event_loop::{ControlFlow, EventLoop};
use glutin::config::{ConfigTemplateBuilder};
use glutin_winit::{DisplayBuilder};

mod display;
mod game;
mod utils;
mod math;

use display::app::*;

pub fn main() -> Result<(), String> {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    // The template will match only the configurations supporting rendering
    // to windows.
    //
    // XXX We force transparency only on macOS, given that EGL on X11 doesn't
    // have it, but we still want to show window. The macOS situation is like
    // that, because we can query only one config at a time on it, but all
    // normal platforms will return multiple configs, so we can find the config
    // with transparency ourselves inside the `reduce`.
    let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(true);

    let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes()));

    let mut app = App::new(template, display_builder);
    event_loop.run_app(&mut app).expect("Failed to run the event loop");

    app.exit_state()
}