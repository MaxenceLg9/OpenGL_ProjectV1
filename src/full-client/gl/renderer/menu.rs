use crate::client::gl::gui::rect::Rect;
use crate::client::gl::gui::ui::{BUTTON_GAP, BUTTON_HEIGHT, BUTTON_WIDTH, Button, ButtonList, TEXT_SIZE, UiRenderer, WHITE, YELLOW};
use crate::client::gl::renderer::renderer::{Action, Renderer};
use crate::client::window::gl_setup::WINDOW_TITLE;
use shared::common::error::err_type::GameError;
use std::time::{Duration, Instant};
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::window::Window;

const TITLE_SIZE: f32 = 36.0;
const SPLASH: &str = "Handmade from scratch!";
const SPLASH_SIZE: f32 = 10.0;
const STATUS_COLOR: glam::Vec3 = glam::vec3(1.0, 0.33, 0.33);
/// Side of a background tile in GUI units.
const TILE_SIZE: f32 = 16.0;
/// Shades of the background tiles, picked pseudo-randomly per tile.
const TILE_SHADES: [glam::Vec4; 3] = [glam::vec4(0.16, 0.12, 0.09, 1.0), glam::vec4(0.19, 0.14, 0.10, 1.0), glam::vec4(0.14, 0.11, 0.08, 1.0)];

/// Title screen: tiled background, title with a pulsing splash text, and the main buttons.
pub struct MenuRenderer {
    ui: UiRenderer,
    buttons: ButtonList<Action>,
    /// cursor position in UI coordinates
    mouse: Option<glam::Vec2>,
    /// message displayed under the buttons, e.g. why the game couldn't start
    status: Option<String>,
    start: Instant,
}

impl MenuRenderer {
    /// Needs `init_gl` to have been called.
    pub fn new(width: u32, height: u32) -> Result<Self, GameError> {
        let buttons = ButtonList::new(vec![Button::new("Play", Action::StartGame), Button::disabled("Options...", Action::None), Button::new("Quit Game", Action::Quit)]);
        let mut menu = Self {
            ui: UiRenderer::new(width, height)?,
            buttons,
            mouse: None,
            status: None,
            start: Instant::now(),
        };
        menu.layout();
        Ok(menu)
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Top of the first button, in GUI units from the top of the screen.
    fn buttons_top(&self) -> f32 {
        self.ui.screen().y / self.ui.unit() / 4.0 + 48.0
    }

    /// "Play" on its own row, then "Options..." and "Quit Game" side by side.
    fn layout(&mut self) {
        let center_x = self.ui.screen().x / 2.0;
        let top = self.buttons_top();
        let play = self.ui.button_rect(center_x, top, BUTTON_WIDTH);
        let [options, quit] = self.ui.half_button_rects(center_x, top + BUTTON_HEIGHT + BUTTON_GAP + 12.0);
        self.buttons.set_rects([play, options, quit]);
    }

    /// Dark tiles of slightly different shades, like a blocky ground seen from above.
    unsafe fn draw_background(&self) {
        let size = TILE_SIZE * self.ui.unit();
        let screen = self.ui.screen();
        let (columns, rows) = ((screen.x / size).ceil() as u32, (screen.y / size).ceil() as u32);
        for column in 0..columns {
            for row in 0..rows {
                // cheap integer hash so the pattern is stable between frames
                let hash = column.wrapping_mul(73_856_093) ^ row.wrapping_mul(19_349_663);
                let shade = TILE_SHADES[(hash % TILE_SHADES.len() as u32) as usize];
                unsafe { self.ui.fill(Rect::new(column as f32 * size, row as f32 * size, size, size), shade) };
            }
        }
    }
}

impl Renderer for MenuRenderer {
    fn draw(&mut self, _window: &Window, _redraw_time: Duration) {
        let screen = self.ui.screen();
        let u = self.ui.unit();
        let center_x = screen.x / 2.0;
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            self.ui.begin();
            self.draw_background();

            // title, then the splash text under its right end, pulsing like a heartbeat
            let title_y = screen.y - 50.0 * u;
            self.ui.text_centered(WINDOW_TITLE, center_x, title_y, TITLE_SIZE, WHITE);
            let pulse = 1.0 + 0.08 * (self.start.elapsed().as_secs_f32() * std::f32::consts::TAU * 1.2).sin().abs();
            let splash_x = center_x + self.ui.text_width(WINDOW_TITLE, TITLE_SIZE) / 2.0 - 30.0 * u;
            self.ui.text_centered(SPLASH, splash_x, title_y - 24.0 * u, SPLASH_SIZE * pulse, YELLOW);

            self.ui.buttons(&self.buttons);

            if let Some(status) = &self.status {
                let bottom = screen.y - (self.buttons_top() + 2.0 * BUTTON_HEIGHT + BUTTON_GAP + 24.0) * u;
                self.ui.text_centered(status, center_x, bottom, TEXT_SIZE, STATUS_COLOR);
            }

            let margin = 2.0 * u;
            self.ui
                .text(&format!("{} {}", WINDOW_TITLE, env!("CARGO_PKG_VERSION")), margin, margin + 2.0 * u, TEXT_SIZE, WHITE);
            let credits = "Made with Rust and OpenGL";
            self.ui
                .text(credits, screen.x - self.ui.text_width(credits, TEXT_SIZE) - margin, margin + 2.0 * u, TEXT_SIZE, WHITE);
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ui.resize(width, height);
        self.layout();
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn grabs_cursor(&self) -> bool {
        false
    }

    fn keyboard_input(&mut self, event: KeyEvent) -> Action {
        self.buttons.keyboard_input(&event).unwrap_or(Action::None)
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> Action {
        self.buttons.mouse_input(self.mouse, state, button).unwrap_or(Action::None)
    }

    fn cursor_moved(&mut self, x: f64, y: f64) {
        let mouse = self.ui.to_ui(x, y);
        self.mouse = Some(mouse);
        self.buttons.hover(mouse);
    }
}
