//! Window and OpenGL context creation helpers (glutin / winit boilerplate).

use glutin::config::{Config, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, GlProfile, NotCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowAttributes};

pub const WINDOW_TITLE: &str = "OpenGL HF";

pub enum GlDisplayCreationState {
    /// The display was not built yet.
    Builder(Box<DisplayBuilder>),
    /// The display was already created for the application.
    Init,
}

pub fn window_attributes() -> WindowAttributes {
    Window::default_attributes()
        .with_transparent(false)
        .with_maximized(false)
        // .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .with_fullscreen(None)
        .with_title(WINDOW_TITLE)
}

/// Picks the config with the most samples (smoother edges), preferring transparency support.
pub fn gl_config_picker(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    configs
        .reduce(|accum, config| {
            let transparency_check = config.supports_transparency().unwrap_or(false) & !accum.supports_transparency().unwrap_or(false);

            if transparency_check || config.num_samples() > accum.num_samples() { config } else { accum }
        })
        .expect("No OpenGL config available")
}

/// Creates an OpenGL 4.6 core context, falling back to GLES then to OpenGL 2.1.
pub fn create_gl_context(window: &Window, gl_config: &Config) -> NotCurrentContext {
    let raw_window_handle = window.window_handle().ok().map(|wh| wh.as_raw());

    let context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(4, 6))))
        .with_profile(GlProfile::Core)
        .with_debug(true)
        .build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new().with_context_api(ContextApi::Gles(None)).build(raw_window_handle);
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(raw_window_handle);

    let gl_display = gl_config.display();
    unsafe {
        gl_display.create_context(gl_config, &context_attributes).unwrap_or_else(|_| {
            gl_display
                .create_context(gl_config, &fallback_context_attributes)
                .unwrap_or_else(|_| gl_display.create_context(gl_config, &legacy_context_attributes).expect("failed to create context"))
        })
    }
}
