use crate::client::gl::gui::text::characters::load_characters;
use gl::types::{GLchar, GLenum, GLsizei, GLuint};
use glutin::display::GlDisplay;
use shared::common::error::err_type::GameError;
use shared::print_base;
use std::ffi::{CStr, CString, c_void};
use std::time::Duration;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::window::Window;

/// What the `App` must do after a renderer handled an event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Nothing to do
    None,
    /// Leave the menu and start the game
    StartGame,
    /// Close the application
    Quit,
}

/// A screen of the application (menu, game...). The `App` owns one at a time,
/// forwards it the window events and switches to another one following the returned `Action`.
pub trait Renderer {
    /// Draws a frame. The buffers are swapped by the `App`.
    fn draw(&mut self, window: &Window, redraw_time: Duration);

    /// The window was resized (physical pixels).
    fn resize(&mut self, width: u32, height: u32) {
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    /// Called at every event loop iteration, to process background work (network, meshes...).
    fn tick(&mut self) {}

    /// Called once per frame, before `draw`, with the time since the previous frame in seconds.
    fn update(&mut self, _delta: f32) {}

    /// Whether the cursor must be hidden and locked in the window.
    fn grabs_cursor(&self) -> bool;

    fn keyboard_input(&mut self, event: KeyEvent) -> Action;

    fn mouse_input(&mut self, _state: ElementState, _button: MouseButton) -> Action {
        Action::None
    }

    /// The cursor moved in the window, in physical pixels from the top-left corner.
    fn cursor_moved(&mut self, _x: f64, _y: f64) {}

    fn mouse_wheel(&mut self, _delta: MouseScrollDelta) {}

    /// Raw mouse movement, also received when the cursor is locked.
    fn mouse_motion(&mut self, _delta: (f64, f64)) {}

    /// The window lost the focus (alt-tab, click outside...).
    fn focus_lost(&mut self) {}
}

extern "system" fn message_callback(_source: GLenum, _gltype: GLenum, _id: GLuint, severity: GLenum, _length: GLsizei, message: *const GLchar, _user_param: *mut c_void) {
    let msg = unsafe { CStr::from_ptr(message).to_string_lossy() };
    print_base!("GL Debug [{}]: {}", severity, msg);
}

/// Loads the OpenGL functions and the shared GL resources (font glyphs).
/// Must be called once, with the context current, before creating any renderer.
pub fn init_gl<D: GlDisplay>(gl_display: &D) -> Result<(), GameError> {
    gl::load_with(|symbol| match CString::new(symbol) {
        Ok(symbol) => gl_display.get_proc_address(&symbol) as *const _,
        Err(e) => {
            print_base!("Got error while trying to load gl environment {}", e);
            std::ptr::null()
        }
    });
    unsafe {
        let renderer = CStr::from_ptr(gl::GetString(gl::RENDERER) as *const i8);
        print_base!("Initiating OpenGL on : {}", renderer.to_string_lossy());
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(message_callback), std::ptr::null());
        load_characters()
    }
}
