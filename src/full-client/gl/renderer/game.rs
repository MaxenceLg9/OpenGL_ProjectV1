use crate::client::game::world::BACKGROUND_COLOR;
use crate::client::game::world::ClientWorld;
use crate::client::gl::gui::Cursor;
use crate::client::gl::renderer::Renderer;
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glutin::display::GlDisplay;
use shared::common::error::err_type::GameError;
use shared::print_base;
use shared::print_debug;
use std::os::raw::c_void;
use std::time::Duration;
use winit::window::Window;

pub struct GameRender {
    world: ClientWorld,
    cursor: Cursor,
}

impl GameRender {
    pub fn get_world(&mut self) -> &mut ClientWorld {
        &mut self.world
    }

    pub fn new<D: GlDisplay>(gl_display: &D) -> Result<Self, GameError> {
        gl::load_with(|symbol| match std::ffi::CString::new(symbol) {
            Ok(symbol_cstr) => gl_display.get_proc_address(&symbol_cstr) as *const _,
            Err(e) => {
                print_base!("Got error while trying to load gl environment {}", e);
                std::ptr::null()
            }
        });
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

            gl::DebugMessageCallback(Some(message_callback), std::ptr::null());
            let mut renderer = Self { world: ClientWorld::new()?, cursor: Cursor::new()? };
            renderer.world.load_characters();
            Ok(renderer)
        }
    }
}

extern "system" fn message_callback(source: GLenum, gltype: GLenum, id: GLuint, severity: GLenum, length: GLsizei, message: *const GLchar, user_param: *mut c_void) {
    let msg = unsafe { std::ffi::CStr::from_ptr(message).to_string_lossy() };
    print_base!("GL Debug [{}]: {}", severity, msg);
}

impl Renderer for GameRender {
    fn draw(&mut self, window: &Window, redraw_time: Duration) {
        unsafe {
            gl::ClearColor(BACKGROUND_COLOR.x, BACKGROUND_COLOR.y, BACKGROUND_COLOR.z, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW); // Counter-clockwise is front
            gl::CullFace(gl::BACK); // Cull back faces
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            self.world.render(window, redraw_time);
            print_debug!("Collecting meshes");
            self.cursor.draw_cursor(window);
        }
    }

    fn tick(&mut self) {
        self.world.tick();
    }

    fn keyboard_input(&self, device_id: winit::event::DeviceId, event: winit::event::KeyEvent, is_synthetic: bool) {}

    fn mouse_input(&self, device_id: winit::event::DeviceId, state: winit::event::ElementState, button: winit::event::MouseButton) {
        todo!()
    }

    fn mouse_wheel(&self, device_id: winit::event::DeviceId, delta: winit::event::MouseScrollDelta, phase: winit::event::TouchPhase) {
        todo!()
    }

    fn mouse_motion(&self, delta: (f64, f64)) {
        todo!()
    }

    fn resize(&self, width: i32, height: i32) {
        unsafe {
            gl::Viewport(0, 0, width, height);
        }
    }
}

// Find the config with the maximum number of samples, so our triangle will be
// smooth.
