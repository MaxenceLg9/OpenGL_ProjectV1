use std::error::Error;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};
use raw_window_handle::HasWindowHandle;
use winit::application::ApplicationHandler;
use winit::event::{ButtonId, DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{KeyCode};
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowAttributes};

use glutin::config::{Config, ConfigTemplateBuilder, GetGlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, PossiblyCurrentContext, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, SwapInterval::*, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::monitor::{MonitorHandle, VideoModeHandle};
use shared::{print_base, print_debug};
use crate::client::display::renderer::renderer::{gl_config_picker, GlDisplayCreationState, Renderer};

const FPS: u64 = 240;
const FRAME_DURATION: Duration = Duration::from_micros(1_000_000 / FPS);

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, gl_config) = match &self.gl_display {
            // We just created the event loop, so initialise the display, pick the config, and
            // create the context.
            GlDisplayCreationState::Builder(display_builder) => {
                let (window, gl_config) = match display_builder.clone().build(
                    event_loop,
                    self.template.clone(),
                    gl_config_picker,
                ) {
                    Ok((window, gl_config)) => (window.unwrap(), gl_config),
                    Err(err) => {
                        self.exit_state = Err(err);
                        event_loop.exit();
                        return;
                    },
                };
                // let fullscreen_mode = window_attributes().fullscreen; // Get the mode we defined
                // window.set_fullscreen(fullscreen_mode);
                print_base!("Picked a config with {} samples", gl_config.num_samples());

                // Mark the display as initialised to not recreate it on resume, since the
                // display is valid until we explicitly destroy it.
                self.gl_display = GlDisplayCreationState::Init;

                // Create gl context.
                self.gl_context = Some(create_gl_context(&window, &gl_config).treat_as_possibly_current());
                (window, gl_config)
            },
            GlDisplayCreationState::Init => {
                print_base!("Recreating window in `resumed`");
                // Pick the config which we already use for the context.
                let gl_config = self.gl_context.as_ref().unwrap().config();
                match glutin_winit::finalize_window(event_loop, window_attributes(Some(&self.state.as_ref().unwrap().window)), &gl_config) {
                    Ok(window) => (window, gl_config),
                    Err(err) => {
                        self.exit_state = Err(err.into());
                        event_loop.exit();
                        return;
                    },
                }
            },
        };
        let attrs = window.build_surface_attributes(Default::default()).expect("Failed to build surface attributes");
        let gl_surface = unsafe { gl_config.display().create_window_surface(&gl_config, &attrs).unwrap() };
        // The context needs to be current for the Renderer to set up shaders and
        // buffers. It also performs function loading, which needs a current context on
        // WGL.
        self.gl_context.as_ref().unwrap().make_current(&gl_surface).expect("Cannot make current from context");
        gl_surface.set_swap_interval(self.gl_context.as_ref().unwrap(), SwapInterval::DontWait).expect("Cannot define swap interval");

        unsafe { self.renderer.get_or_insert_with(|| Renderer::new(&gl_config.display())); }

        assert!(self.state.replace(AppState { gl_surface, window }).is_none());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) if size.width != 0 && size.height != 0 => {
                // Some platforms like EGL require resizing GL surface to update the size
                // Notable platforms here are Wayland and macOS, other don't require it
                // and the function is no-op, but it's wise to resize it for portability
                // reasons.
                if let Some(AppState { gl_surface, window: _ }) = self.state.as_ref() {
                    let gl_context = self.gl_context.as_ref().unwrap();
                    gl_surface.resize(
                        gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );

                    let renderer = self.renderer.as_ref().unwrap();
                    renderer.resize(size.width as i32, size.height as i32);
                }
            },
            WindowEvent::RedrawRequested => {
                // Combine all checks into one clean pattern
                if let (Some(renderer), Some(state), Some(context)) = (&mut self.renderer, &self.state, &self.gl_context) {
                    let instant = Instant::now();
                    // Draw logic
                    renderer.draw(&state.window, self.redraw_time); // Encapsulate the clear/draw inside the renderer!

                    // Swap buffers
                    state.gl_surface.swap_buffers(context).expect("Failed to swap buffers");

                    // Tell winit to loop immediately (for 60+ FPS)
                    // state.window.set_fullscreen(window_attributes().fullscreen);
                    self.redraw_time = Instant::now() - instant;
                }
            },
            WindowEvent::KeyboardInput { device_id, event, is_synthetic} => {
                if event.physical_key == KeyCode::Escape {
                    event_loop.exit();
                }
                self.renderer.as_mut().unwrap().get_world().get_keyboard().keyboard_callback(event);
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Focused(true) => {
                if let Some(AppState { window, .. }) = self.state.as_ref() {
                    print_debug!("Window focused, grabbing cursor");
                    let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                        .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
                    window.set_cursor_visible(false);
                }
            },
            WindowEvent::MouseInput {device_id, state, button} => {
                self.renderer.as_mut().unwrap().get_world().get_keyboard().button_callback(button,state);
            },
            WindowEvent::MouseWheel {device_id, delta, phase} => {
                self.renderer.as_mut().unwrap().get_world().get_player().write().unwrap().add_fov(delta);
            }
            _ => (),
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                // print_base!("Mouse motion delta: {:?}", delta);
                self.renderer.as_mut().unwrap().get_world().get_player().write().unwrap().mouse_callback(delta.0,delta.1);
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let current_frame = std::time::Instant::now();
        if current_frame - self.last_frame >= FRAME_DURATION {
            let delta = current_frame.duration_since(self.last_frame).as_secs_f32();
            self.renderer.as_mut().unwrap().get_world().poll_keys(delta);
            self.last_frame = current_frame;
            if let Some(AppState { gl_surface, window }) = self.state.as_ref() {
                window.request_redraw();
            }
        }
        _event_loop.set_control_flow(ControlFlow::Poll);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        // This event is only raised on Android, where the backing NativeWindow for a GL
        // Surface can appear and disappear at any moment.
        print_base!("Android window removed");

        // Destroy the GL Surface and un-current the GL Context before ndk-glue releases
        // the window back to the system.
        self.state = None;

        // Make context not current.
        self.gl_context = Some(
            self.gl_context.take().unwrap().make_not_current().unwrap().treat_as_possibly_current(),
        );
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // NOTE: The handling below is only needed due to nvidia on Wayland to not crash
        // on exit due to nvidia driver touching the Wayland display from on
        // `exit` hook.
        let _gl_display = self.gl_context.take().unwrap().display();

        // Clear the window.
        self.state = None;
        #[cfg(egl_backend)]
        #[allow(irrefutable_let_patterns)]
        if let glutin::display::Display::Egl(display) = _gl_display {
            unsafe {
                display.terminate();
            }
        }
    }
}

fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
        .with_profile(GlProfile::Core)
        .with_debug(true)
        .build(raw_window_handle);

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    unsafe {
        gl_display.create_context(gl_config, &context_attributes).unwrap_or_else(|_| {
            gl_display.create_context(gl_config, &fallback_context_attributes).unwrap_or_else(
                |_| {
                    gl_display
                        .create_context(gl_config, &legacy_context_attributes)
                        .expect("failed to create context")
                },
            )
        })
    }
}

pub fn window_attributes(option_window: Option<&Window>) -> WindowAttributes {
    // Wayland fix: Get the first available monitor since "Primary" is usually None
    if let Some(window) = option_window {
        for monitor in window.available_monitors() {
            for modes in monitor.video_modes() {
                print_base!("Initialised window with Exclusive mode");
                return Window::default_attributes()
                    .with_transparent(false)
                    .with_maximized(true)
                    .with_fullscreen(Some(Fullscreen::Exclusive(modes)))
                    .with_title("OpenGL HF");
            }
        }
    }
    Window::default_attributes()
        .with_transparent(false)
        .with_maximized(true)
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_title("OpenGL HF")
}

pub struct App {
    template: ConfigTemplateBuilder,
    renderer: Option<Renderer>,
    // NOTE: `AppState` carries the `Window`, thus it should be dropped after everything else.
    state: Option<AppState>,
    gl_context: Option<PossiblyCurrentContext>,
    gl_display: GlDisplayCreationState,
    exit_state: Result<(), Box<dyn Error>>,
    last_frame: Instant,
    redraw_time: Duration
}

impl App {
    pub fn new(template: ConfigTemplateBuilder) -> Self {
        Self {
            template,
            gl_display: GlDisplayCreationState::Builder(Box::new(DisplayBuilder::new().with_window_attributes(Some(window_attributes(None))))),
            exit_state: Ok(()),
            gl_context: None,
            state: None,
            redraw_time: Duration::from_secs(0),
            renderer: None,
            last_frame: Instant::now()
        }
    }

    pub fn exit_state(&self) -> Result<(), String> {
        (&self.exit_state).as_ref().map(|_| ()).map_err(|e| e.to_string())
    }
}

struct AppState {
    gl_surface: Surface<WindowSurface>,
    // NOTE: Window should be dropped after all resources created using its
    // raw-window-handle.
    window: Window,
}