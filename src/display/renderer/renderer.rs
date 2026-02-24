use std::ffi::CString;
use std::os::raw::c_void;
use std::sync::Arc;
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glutin::config::Config;
use glutin::display::GlDisplay;
use glutin_winit::DisplayBuilder;
use glutin::prelude::*;
use winit::window::Window;
use crate::display::renderer::gui::Cursor;
use crate::game::world::player::player::PlayerUser;
use crate::game::world::world::World;

pub enum GlDisplayCreationState {
    /// The display was not build yet.
    Builder(Box<DisplayBuilder>),
    /// The display was already created for the application.
    Init,
}

pub struct Renderer {
    world: World,
    cursor: Cursor,
    player: PlayerUser
}

impl Renderer {
    pub(crate) fn get_world(&self) -> &World {
        &self.world
    }
}

extern "system" fn message_callback(source: GLenum, gltype: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, user_param: *mut c_void) {
    let msg = unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() };
    println!("GL Debug [{}]: {}", severity, msg);
}

impl Renderer {
    pub unsafe fn new<D: GlDisplay>(gl_display: &D) -> Self {
        gl::load_with(|symbol| {
            let symbol_cstr = std::ffi::CString::new(symbol).unwrap();
            gl_display.get_proc_address(&symbol_cstr) as *const _
        });
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(message_callback),std::ptr::null());
        let mut renderer = Self {
            world: World::new(),
            cursor: Cursor::new(),
            player: PlayerUser::new(-10_f32, 650_f32, -10_f32)
        };
        renderer
    }

    pub fn draw(&mut self, window: &Window) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK,gl::FILL);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW); // Counter-clockwise is front
            gl::CullFace(gl::BACK); // Cull back faces

            self.world.render(window,&self.player);
            self.world.collect_meshes();
            self.world.build_chunk_mesh();
            self.cursor.draw_cursor(window);
        }
    }

    pub fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }

    pub fn get_player(&mut self) -> &mut PlayerUser {
        &mut self.player
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