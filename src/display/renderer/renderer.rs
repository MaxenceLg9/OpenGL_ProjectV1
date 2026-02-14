use std::ffi::CString;
use std::sync::Arc;
use glutin::config::Config;
use glutin::display::GlDisplay;
use glutin_winit::DisplayBuilder;
use glutin::prelude::*;
use winit::window::Window;
use crate::display::renderer::gui::Cursor;
use crate::game::world::world::World;

pub enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(Box<DisplayBuilder>),
    /// The display was already created for the application.
    Init,
}

pub struct Renderer {
    world: World,
    cursor: Cursor
}

impl Renderer {
    pub unsafe fn new<D: GlDisplay>(gl_display: &D) -> Self {
        gl::load_with(|symbol| {
            let symbol_cstr = std::ffi::CString::new(symbol).unwrap();
            gl_display.get_proc_address(&symbol_cstr) as *const _
        });
        let mut renderer = Self {
            world: World::new(),
            cursor: Cursor::new()
        };
        renderer
    }
    pub fn draw(&mut self, window: &Window) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Disable(gl::CULL_FACE); // Show both sides of triangles
            gl::Disable(gl::DEPTH_TEST); // Draw regardless of distance
            self.world.render(window);
            self.world.collect_meshes();
            self.world.build_chunk_mesh();
            self.cursor.drawCursor(window);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}



// Find the config with the maximum number of samples, so our triangle will be
// smooth.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false)
                & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() {
                config
            } else {
                accum
            }
        })
        .unwrap()
}