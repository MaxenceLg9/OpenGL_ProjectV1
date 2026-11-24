use crate::client::game::world::{BACKGROUND_COLOR, ClientWorld};
use crate::client::gl::gui::Cursor;
use crate::client::gl::gui::rect::Rect;
use crate::client::gl::gui::ui::{BUTTON_GAP, BUTTON_HEIGHT, BUTTON_WIDTH, Button, ButtonList, TEXT_SIZE, UiRenderer, WHITE};
use crate::client::gl::renderer::renderer::{Action, Renderer};
use shared::common::error::err_type::GameError;
use shared::print_base;
use std::time::Duration;
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::Window;

/// Darkens the world behind the pause menu.
const PAUSE_OVERLAY: glam::Vec4 = glam::vec4(0.0, 0.0, 0.0, 0.55);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PauseAction {
    Resume,
    ToggleDebug,
    Quit,
}

/// Menu opened with Escape (or when the window loses the focus). The world keeps running behind it.
struct PauseMenu {
    buttons: ButtonList<PauseAction>,
    /// cursor position in UI coordinates
    mouse: Option<glam::Vec2>,
}

impl PauseMenu {
    fn new(ui: &UiRenderer) -> Self {
        let mut menu = Self {
            buttons: ButtonList::new(vec![
                Button::new("Back to Game", PauseAction::Resume),
                Button::disabled("Options...", PauseAction::Resume),
                Button::new("Debug View", PauseAction::ToggleDebug),
                Button::new("Quit Game", PauseAction::Quit),
            ]),
            mouse: None,
        };
        menu.layout(ui);
        menu
    }

    /// Top of the first button, in GUI units from the top of the screen.
    fn buttons_top(ui: &UiRenderer) -> f32 {
        ui.screen().y / ui.unit() / 4.0 + 8.0
    }

    /// "Back to Game", then "Options..." and "Debug View" side by side, then "Quit Game" a bit lower.
    fn layout(&mut self, ui: &UiRenderer) {
        let center_x = ui.screen().x / 2.0;
        let top = Self::buttons_top(ui);
        let row = BUTTON_HEIGHT + BUTTON_GAP;
        let back = ui.button_rect(center_x, top, BUTTON_WIDTH);
        let [options, debug] = ui.half_button_rects(center_x, top + row);
        let quit = ui.button_rect(center_x, top + 2.0 * row + 16.0, BUTTON_WIDTH);
        self.buttons.set_rects([back, options, debug, quit]);
    }

    unsafe fn draw(&self, ui: &UiRenderer) {
        let screen = ui.screen();
        unsafe {
            ui.begin();
            ui.fill(Rect::new(0.0, 0.0, screen.x, screen.y), PAUSE_OVERLAY);
            let title_y = screen.y - (Self::buttons_top(ui) - 16.0) * ui.unit();
            ui.text_centered("Game Menu", screen.x / 2.0, title_y, TEXT_SIZE, WHITE);
            ui.buttons(&self.buttons);
        }
    }
}

/// The in-game screen: renders the world, forwards the inputs to the player, and shows the pause menu.
pub struct GameRenderer {
    world: ClientWorld,
    cursor: Cursor,
    ui: UiRenderer,
    pause: Option<PauseMenu>,
}

impl GameRenderer {
    /// Creates the world, which connects to the server. Needs `init_gl` to have been called.
    pub fn new() -> Result<Self, GameError> {
        unsafe {
            Ok(Self {
                world: ClientWorld::new()?,
                cursor: Cursor::new()?,
                // the real size is given by `resize` right after the creation
                ui: UiRenderer::new(1, 1)?,
                pause: None,
            })
        }
    }

    pub fn get_world(&mut self) -> &mut ClientWorld {
        &mut self.world
    }

    fn pause(&mut self) {
        if self.pause.is_none() {
            // keys held when pausing would stay pressed: the player would keep moving after resuming
            self.world.get_player().write().unwrap().get_keyboard().release_all();
            self.pause = Some(PauseMenu::new(&self.ui));
        }
    }

    fn resume(&mut self) {
        self.pause = None;
    }

    fn handle_pause_action(&mut self, action: Option<PauseAction>) -> Action {
        match action {
            Some(PauseAction::Resume) => self.resume(),
            Some(PauseAction::ToggleDebug) => self.world.toggle_debug(),
            // the network thread can't be restarted yet, so quitting the game closes the client
            Some(PauseAction::Quit) => return Action::Quit,
            None => {}
        }
        Action::None
    }
}

fn is_escape_press(event: &KeyEvent) -> bool {
    event.physical_key == PhysicalKey::Code(KeyCode::Escape) && event.state == ElementState::Pressed && !event.repeat
}

impl Renderer for GameRenderer {
    fn draw(&mut self, window: &Window, redraw_time: Duration) {
        unsafe {
            gl::ClearColor(BACKGROUND_COLOR.x, BACKGROUND_COLOR.y, BACKGROUND_COLOR.z, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);
            gl::Enable(gl::CULL_FACE);
            gl::FrontFace(gl::CW);
            gl::CullFace(gl::BACK);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            self.world.render(window, redraw_time);
            match &self.pause {
                Some(pause) => pause.draw(&self.ui),
                None => self.cursor.draw_cursor(window),
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.ui.resize(width, height);
        if let Some(pause) = self.pause.as_mut() {
            pause.layout(&self.ui);
        }
        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn tick(&mut self) {
        // the world keeps running while paused: network, chunks, meshes
        if let Err(e) = self.world.tick() {
            print_base!("Error while ticking the world: {}", e);
        }
    }

    fn update(&mut self, delta: f32) {
        if self.pause.is_none() {
            self.world.poll_keys(delta);
        }
    }

    fn grabs_cursor(&self) -> bool {
        self.pause.is_none()
    }

    fn keyboard_input(&mut self, event: KeyEvent) -> Action {
        if let Some(pause) = self.pause.as_mut() {
            if is_escape_press(&event) {
                self.resume();
                return Action::None;
            }
            let action = pause.buttons.keyboard_input(&event);
            return self.handle_pause_action(action);
        }
        if is_escape_press(&event) {
            self.pause();
            return Action::None;
        }
        self.world.get_player().write().unwrap().get_keyboard().keyboard_callback(event);
        Action::None
    }

    fn mouse_input(&mut self, state: ElementState, button: MouseButton) -> Action {
        if let Some(pause) = self.pause.as_mut() {
            let action = pause.buttons.mouse_input(pause.mouse, state, button);
            return self.handle_pause_action(action);
        }
        self.world.get_player().write().unwrap().get_keyboard().button_callback(button, state);
        Action::None
    }

    fn cursor_moved(&mut self, x: f64, y: f64) {
        if let Some(pause) = self.pause.as_mut() {
            let mouse = self.ui.to_ui(x, y);
            pause.mouse = Some(mouse);
            pause.buttons.hover(mouse);
        }
    }

    fn mouse_wheel(&mut self, delta: MouseScrollDelta) {
        if self.pause.is_none() {
            self.world.get_player().write().unwrap().get_camera_mut().add_fov(delta);
        }
    }

    fn mouse_motion(&mut self, delta: (f64, f64)) {
        if self.pause.is_none() {
            self.world.get_player().write().unwrap().get_camera_mut().mouse_callback(delta.0, delta.1);
        }
    }

    fn focus_lost(&mut self) {
        self.pause();
    }
}
