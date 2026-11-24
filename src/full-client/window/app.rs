use crate::client::gl::renderer::game::GameRenderer;
use crate::client::gl::renderer::menu::MenuRenderer;
use crate::client::gl::renderer::renderer::{Action, Renderer, init_gl};
use crate::client::window::gl_setup::{GlDisplayCreationState, create_gl_context, gl_config_picker, window_attributes};
use glutin::config::{ConfigTemplateBuilder, GetGlConfig};
use glutin::context::PossiblyCurrentContext;
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use shared::print_base;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{CursorGrabMode, Window, WindowId};

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

pub struct App {
    template: ConfigTemplateBuilder,
    /// current screen (menu or game), created once the GL context exists
    renderer: Option<Box<dyn Renderer>>,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_display: GlDisplayCreationState,
    exit_state: Result<(), Box<dyn std::error::Error>>,
    last_frame: Instant,
    redraw_time: Duration,
    /// cursor lock last applied to the window, to only touch it when the screen wants a change
    cursor_grabbed: Option<bool>,
}

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}

impl App {
    pub fn new(template: ConfigTemplateBuilder) -> Self {
        Self {
            template,
            gl_display: GlDisplayCreationState::Builder(Box::new(DisplayBuilder::new().with_window_attributes(Some(window_attributes())))),
            exit_state: Ok(()),
            gl_context: None,
            state: None,
            redraw_time: Duration::ZERO,
            renderer: None,
            last_frame: Instant::now(),
            cursor_grabbed: None,
        }
    }

    pub fn exit_state(&self) -> Result<(), String> {
        self.exit_state.as_ref().map(|_| ()).map_err(|e| e.to_string())
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, error: impl Into<Box<dyn std::error::Error>>) {
        let error = error.into();
        print_base!("Fatal error: {}", error);
        self.exit_state = Err(error);
        event_loop.exit();
    }

    fn window_size(&self) -> (u32, u32) {
        self.state.as_ref().map(|s| s.window.inner_size()).map(|s| (s.width, s.height)).unwrap_or((1, 1))
    }

    /// Replaces the current screen and adapts the cursor and viewport to it.
    fn set_renderer(&mut self, mut renderer: Box<dyn Renderer>) {
        let (width, height) = self.window_size();
        renderer.resize(width, height);
        self.renderer = Some(renderer);
        self.update_cursor_grab();
    }

    /// Locks and hides the cursor, or frees it, depending on what the current screen wants.
    fn update_cursor_grab(&mut self) {
        let (Some(AppState { window, .. }), Some(renderer)) = (&self.state, &self.renderer) else {
            return;
        };
        let grab = renderer.grabs_cursor();
        self.cursor_grabbed = Some(grab);
        if grab {
            let _ = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }

    /// Applies the cursor lock if the screen changed its mind (e.g. the game opened its pause menu).
    fn sync_cursor_grab(&mut self) {
        if self.renderer.as_ref().map(|r| r.grabs_cursor()) != self.cursor_grabbed {
            self.update_cursor_grab();
        }
    }

    /// Executes what the current screen asked for.
    fn handle_action(&mut self, event_loop: &ActiveEventLoop, action: Action) {
        match action {
            Action::None => {}
            Action::Quit => event_loop.exit(),
            Action::StartGame => match GameRenderer::new() {
                Ok(game) => self.set_renderer(Box::new(game)),
                Err(e) => {
                    print_base!("Cannot start the game: {}", e);
                    let (width, height) = self.window_size();
                    match MenuRenderer::new(width, height) {
                        Ok(menu) => self.set_renderer(Box::new(menu.with_status(format!("Cannot start the game: {}", e)))),
                        Err(e) => self.fail(event_loop, e),
                    }
                }
            },
        }
    }

    /// Creates the window and the GL context on the first call, only the window afterwards.
    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn std::error::Error>> {
        let (window, gl_config) = match &self.gl_display {
            GlDisplayCreationState::Builder(display_builder) => {
                let (window, gl_config) = display_builder.clone().build(event_loop, self.template.clone(), gl_config_picker)?;
                let window = window.ok_or("The display builder did not create a window")?;
                print_base!("Picked a config with {} samples", gl_config.num_samples());
                // the display is valid until we explicitly destroy it, don't recreate it on resume
                self.gl_display = GlDisplayCreationState::Init;
                self.gl_context = Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());
                (window, gl_config)
            }
            GlDisplayCreationState::Init => {
                print_base!("Recreating window in `resumed`");
                let gl_config = self.gl_context.as_ref().ok_or("No GL context")?.config();
                (glutin_winit::finalize_window(event_loop, window_attributes(), &gl_config)?, gl_config)
            }
        };

        let attrs = window.build_surface_attributes(Default::default())?;
        let gl_surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs)? };
        let gl_context = self.gl_context.as_ref().ok_or("No GL context")?;
        // the context must be current for the renderers to create their shaders and buffers
        gl_context.make_current(&gl_surface)?;
        gl_surface.set_swap_interval(gl_context, SwapInterval::DontWait)?;

        if self.renderer.is_none() {
            init_gl(&gl_config.display())?;
        }
        assert!(self.state.replace(AppState { gl_surface, window }).is_none());
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        let (Some(width_nz), Some(height_nz)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return; // minimized
        };
        // Some platforms like EGL require resizing the GL surface (Wayland, macOS)
        if let (Some(AppState { gl_surface, .. }), Some(gl_context)) = (&self.state, &self.gl_context) {
            gl_surface.resize(gl_context, width_nz, height_nz);
        }
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(width, height);
        }
    }

    fn redraw(&mut self) {
        if let (Some(renderer), Some(state), Some(context)) = (&mut self.renderer, &self.state, &self.gl_context) {
            let instant = Instant::now();
            renderer.draw(&state.window, self.redraw_time);
            if let Err(e) = state.gl_surface.swap_buffers(context) {
                print_base!("Failed to swap buffers: {}", e);
            }
            self.redraw_time = instant.elapsed();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.create_window(event_loop) {
            return self.fail(event_loop, e);
        }
        if self.renderer.is_some() {
            // resumed after a suspension: keep the current screen
            self.update_cursor_grab();
            return;
        }
        let (width, height) = self.window_size();
        match MenuRenderer::new(width, height) {
            Ok(menu) => self.set_renderer(Box::new(menu)),
            Err(e) => self.fail(event_loop, e),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(renderer) = self.renderer.as_mut() else {
            if let WindowEvent::CloseRequested = event {
                event_loop.exit();
            }
            return;
        };
        let action = match event {
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
                Action::None
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
                Action::None
            }
            WindowEvent::CloseRequested => Action::Quit,
            WindowEvent::Focused(true) => {
                // the window manager may have released the cursor while unfocused
                self.update_cursor_grab();
                Action::None
            }
            WindowEvent::Focused(false) => {
                renderer.focus_lost();
                Action::None
            }
            WindowEvent::KeyboardInput { event, .. } => renderer.keyboard_input(event),
            WindowEvent::MouseInput { state, button, .. } => renderer.mouse_input(state, button),
            WindowEvent::CursorMoved { position, .. } => {
                renderer.cursor_moved(position.x, position.y);
                Action::None
            }
            WindowEvent::MouseWheel { delta, .. } => {
                renderer.mouse_wheel(delta);
                Action::None
            }
            _ => Action::None,
        };
        self.handle_action(event_loop, action);
        self.sync_cursor_grab();
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        if let (DeviceEvent::MouseMotion { delta }, Some(renderer)) = (event, self.renderer.as_mut()) {
            renderer.mouse_motion(delta);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        renderer.tick();

        let now = Instant::now();
        if now - self.last_frame >= FRAME_DURATION {
            renderer.update((now - self.last_frame).as_secs_f32());
            self.last_frame = now;
            if let Some(AppState { window, .. }) = self.state.as_ref() {
                window.request_redraw();
            }
        }
        // sleep until the next frame instead of spinning, events still wake the loop up
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.last_frame + FRAME_DURATION));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        print_base!("Android window removed");

        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        self.state = None;
        if let Some(context) = self.gl_context.take() {
            self.gl_context = Some(context.make_not_current().expect("Cannot release the GL context").treat_as_possibly_current());
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // GL resources must be freed while the context still exists
        self.renderer = None;
        // NOTE: The handling below is only needed due to nvidia on Wayland to not crash
        // on exit due to nvidia driver touching the Wayland display from on
        // `exit` hook.
        let _gl_display = self.gl_context.take().map(|context| context.display());

        // Clear the window.
        self.state = None;
        #[allow(irrefutable_let_patterns)]
        if let Some(glutin::display::Display::Egl(display)) = _gl_display {
            unsafe {
                display.terminate();
            }
        }
    }
}
