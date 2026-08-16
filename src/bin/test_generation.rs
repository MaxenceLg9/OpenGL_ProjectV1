use std::io;
use std::io::Cursor;
use std::ops::{Add, Deref, Div, Mul};
use bitvec::macros::internal::funty::Fundamental;
use glutin::config::ConfigTemplateBuilder;
use image::{DynamicImage, GenericImage, ImageFormat, ImageReader, Rgba, RgbaImage};
use noise::{NoiseFn, Perlin};
use winit::event_loop::{ControlFlow, EventLoop};
use shared::worldgen::Generator;
use shared::print_base;
use crate::test::display::app::App;

#[path="../client/mod.rs"] pub mod client;
#[path="../test/mod.rs"] pub mod test;
#[path="../server/mod.rs"] pub mod server;


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

pub fn main() -> io::Result<()> {
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
    let template = ConfigTemplateBuilder::new().with_alpha_size(8).with_transparency(false);
    let mut app = App::new(template);
    event_loop.run_app(&mut app).expect("Failed to run the event loop");

    app.exit_state();
    Ok(())
}