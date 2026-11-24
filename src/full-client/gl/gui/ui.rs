//! Small immediate-mode UI toolkit: GUI scale, bevelled buttons, shadowed text, button lists.
//! Everything is drawn with flat colored rectangles and the bitmap font, no textures.

use crate::client::gl::gui::rect::{Rect, RectRenderer};
use crate::client::gl::gui::text::characters::{FONT_PX_SIZE, Text};
use crate::client::gl::mesh::shader::shader::Shader;
use shared::common::error::err_type::GameError;
use shared::common::file::path::AssetsReader;
use winit::event::{ElementState, KeyEvent, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// Virtual resolution the GUI is designed for: the GUI scale is the biggest integer
/// such that this size still fits in the window.
const GUI_SIZE: glam::Vec2 = glam::vec2(320.0, 240.0);
/// Height of the default text in GUI units.
pub const TEXT_SIZE: f32 = 10.0;
/// Height of a button in GUI units.
pub const BUTTON_HEIGHT: f32 = 20.0;
/// Width of a full button in GUI units.
pub const BUTTON_WIDTH: f32 = 200.0;
/// Gap between two buttons in GUI units.
pub const BUTTON_GAP: f32 = 4.0;

pub const WHITE: glam::Vec3 = glam::vec3(1.0, 1.0, 1.0);
pub const YELLOW: glam::Vec3 = glam::vec3(1.0, 1.0, 0.0);
const HOVER_TEXT: glam::Vec3 = glam::vec3(1.0, 1.0, 0.63);
const DISABLED_TEXT: glam::Vec3 = glam::vec3(0.63, 0.63, 0.63);

/// Colors of a button: fill, light bevel (top/left), dark bevel (bottom/right), outline.
struct ButtonStyle {
    fill: glam::Vec4,
    light: glam::Vec4,
    dark: glam::Vec4,
    outline: glam::Vec4,
    text: glam::Vec3,
}

const NORMAL_STYLE: ButtonStyle = ButtonStyle {
    fill: glam::vec4(0.44, 0.44, 0.44, 1.0),
    light: glam::vec4(0.66, 0.66, 0.66, 1.0),
    dark: glam::vec4(0.30, 0.30, 0.30, 1.0),
    outline: glam::vec4(0.0, 0.0, 0.0, 1.0),
    text: WHITE,
};
const HOVER_STYLE: ButtonStyle = ButtonStyle {
    fill: glam::vec4(0.48, 0.53, 0.70, 1.0),
    light: glam::vec4(0.66, 0.71, 0.88, 1.0),
    dark: glam::vec4(0.33, 0.37, 0.52, 1.0),
    outline: glam::vec4(1.0, 1.0, 1.0, 1.0),
    text: HOVER_TEXT,
};
const DISABLED_STYLE: ButtonStyle = ButtonStyle {
    fill: glam::vec4(0.17, 0.17, 0.17, 1.0),
    light: glam::vec4(0.22, 0.22, 0.22, 1.0),
    dark: glam::vec4(0.12, 0.12, 0.12, 1.0),
    outline: glam::vec4(0.0, 0.0, 0.0, 1.0),
    text: DISABLED_TEXT,
};

/// A clickable button producing `A` when pressed.
pub struct Button<A> {
    pub label: &'static str,
    pub action: A,
    pub enabled: bool,
    /// position on screen, set by the owner with `ButtonList::set_rects`
    rect: Rect,
}

impl<A> Button<A> {
    pub fn new(label: &'static str, action: A) -> Self {
        Self {
            label,
            action,
            enabled: true,
            rect: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn disabled(label: &'static str, action: A) -> Self {
        Self { enabled: false, ..Self::new(label, action) }
    }
}

/// Buttons navigable with the mouse (hover + left click) and the keyboard
/// (Tab / arrows / W S to move, Enter / Space to press). Disabled buttons are skipped.
pub struct ButtonList<A> {
    buttons: Vec<Button<A>>,
    selected: Option<usize>,
}

impl<A: Clone> ButtonList<A> {
    pub fn new(buttons: Vec<Button<A>>) -> Self {
        Self { buttons, selected: None }
    }

    /// Places the buttons, in the same order as they were given.
    pub fn set_rects(&mut self, rects: impl IntoIterator<Item = Rect>) {
        for (button, rect) in self.buttons.iter_mut().zip(rects) {
            button.rect = rect;
        }
    }

    pub fn buttons(&self) -> &[Button<A>] {
        &self.buttons
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    /// Selects the enabled button under the cursor (and deselects when leaving it).
    pub fn hover(&mut self, mouse: glam::Vec2) {
        self.selected = self.buttons.iter().position(|b| b.enabled && b.rect.contains(mouse));
    }

    pub fn mouse_input(&mut self, mouse: Option<glam::Vec2>, state: ElementState, button: MouseButton) -> Option<A> {
        if state != ElementState::Pressed || button != MouseButton::Left {
            return None;
        }
        let mouse = mouse?;
        self.buttons.iter().find(|b| b.enabled && b.rect.contains(mouse)).map(|b| b.action.clone())
    }

    pub fn keyboard_input(&mut self, event: &KeyEvent) -> Option<A> {
        if event.state != ElementState::Pressed {
            return None;
        }
        match event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowUp | KeyCode::KeyW) => self.move_selection(-1),
            PhysicalKey::Code(KeyCode::ArrowDown | KeyCode::KeyS | KeyCode::Tab) => self.move_selection(1),
            PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space) => {
                return self.selected.and_then(|i| self.buttons.get(i)).filter(|b| b.enabled).map(|b| b.action.clone());
            }
            _ => {}
        }
        None
    }

    /// Moves the selection to the next enabled button in `direction` (wrapping around).
    fn move_selection(&mut self, direction: isize) {
        let len = self.buttons.len() as isize;
        if len == 0 {
            return;
        }
        let mut index = self.selected.map_or(if direction > 0 { -1 } else { len }, |i| i as isize);
        for _ in 0..len {
            index = (index + direction).rem_euclid(len);
            if self.buttons[index as usize].enabled {
                self.selected = Some(index as usize);
                return;
            }
        }
    }
}

/// Draws the UI elements in window pixels, origin at the bottom-left.
pub struct UiRenderer {
    text_shader: Shader,
    text: Text,
    rects: RectRenderer,
    screen: glam::Vec2,
}

impl UiRenderer {
    /// Needs `init_gl` to have been called.
    pub fn new(width: u32, height: u32) -> Result<Self, GameError> {
        unsafe {
            Ok(Self {
                text_shader: Shader::new(AssetsReader::new("shaders/text/vertex.vert"), AssetsReader::new("shaders/text/fragment.frag"))?,
                text: Text::new(),
                rects: RectRenderer::new()?,
                screen: glam::vec2(width as f32, height as f32),
            })
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen = glam::vec2(width as f32, height as f32);
    }

    pub fn screen(&self) -> glam::Vec2 {
        self.screen
    }

    /// Pixels per GUI unit: the biggest integer such that `GUI_SIZE` fits in the window.
    pub fn unit(&self) -> f32 {
        (self.screen / GUI_SIZE).min_element().floor().max(1.0)
    }

    /// Converts a cursor position from winit (y down) to UI coordinates (y up).
    pub fn to_ui(&self, x: f64, y: f64) -> glam::Vec2 {
        glam::vec2(x as f32, self.screen.y - y as f32)
    }

    /// Sets the GL state for 2D drawing over whatever was drawn before.
    pub unsafe fn begin(&self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }
    }

    pub unsafe fn fill(&self, rect: Rect, color: glam::Vec4) {
        unsafe { self.rects.draw(rect, color, self.screen) }
    }

    /// Font scale for a text `size` GUI units high.
    fn font_scale(&self, size: f32) -> f32 {
        size * self.unit() / FONT_PX_SIZE
    }

    pub fn text_width(&self, text: &str, size: f32) -> f32 {
        Text::text_width(text, self.font_scale(size))
    }

    /// Draws text with its baseline at `y`, with a darker drop shadow one GUI unit away.
    pub unsafe fn text(&self, text: &str, x: f32, y: f32, size: f32, color: glam::Vec3) {
        let scale = self.font_scale(size);
        let offset = self.unit() * size / TEXT_SIZE;
        unsafe {
            self.text
                .render_text_on(&self.text_shader, text, x + offset, y - offset, scale, color * 0.25, self.screen);
            self.text.render_text_on(&self.text_shader, text, x, y, scale, color, self.screen);
        }
    }

    /// Draws text centered horizontally on `center_x` and vertically on `center_y`.
    pub unsafe fn text_centered(&self, text: &str, center_x: f32, center_y: f32, size: f32, color: glam::Vec3) {
        let x = center_x - self.text_width(text, size) / 2.0;
        // the visual middle of capitals is about a third of the font size above the baseline
        let y = center_y - size * self.unit() * 0.33;
        unsafe { self.text(text, x, y, size, color) }
    }

    /// Draws a bevelled button: black outline (white when hovered), light top-left edge, dark bottom-right edge.
    unsafe fn button(&self, rect: Rect, label: &str, hovered: bool, enabled: bool) {
        let style = match (enabled, hovered) {
            (false, _) => &DISABLED_STYLE,
            (true, true) => &HOVER_STYLE,
            (true, false) => &NORMAL_STYLE,
        };
        let u = self.unit();
        let inner = Rect::new(rect.x + u, rect.y + u, rect.width - 2.0 * u, rect.height - 2.0 * u);
        unsafe {
            self.fill(rect, style.outline);
            self.fill(inner, style.dark);
            // light edge on the top and left, the dark one stays visible on the bottom and right
            self.fill(Rect::new(inner.x, inner.y + u, inner.width - u, inner.height - u), style.light);
            self.fill(Rect::new(inner.x + u, inner.y + u, inner.width - 2.0 * u, inner.height - 2.0 * u), style.fill);
            let center = rect.center();
            self.text_centered(label, center.x, center.y, TEXT_SIZE, style.text);
        }
    }

    pub unsafe fn buttons<A: Clone>(&self, list: &ButtonList<A>) {
        for (i, button) in list.buttons().iter().enumerate() {
            unsafe { self.button(button.rect, button.label, list.selected() == Some(i), button.enabled) }
        }
    }

    /// Rectangle of a button `width` GUI units wide, with its top edge `top` GUI units under the top of the screen.
    pub fn button_rect(&self, center_x: f32, top: f32, width: f32) -> Rect {
        let u = self.unit();
        Rect::new(center_x - width * u / 2.0, self.screen.y - (top + BUTTON_HEIGHT) * u, width * u, BUTTON_HEIGHT * u)
    }

    /// Two half buttons side by side filling the width of a full button.
    pub fn half_button_rects(&self, center_x: f32, top: f32) -> [Rect; 2] {
        let half = (BUTTON_WIDTH - BUTTON_GAP) / 2.0;
        let offset = (half + BUTTON_GAP) / 2.0 * self.unit();
        [self.button_rect(center_x - offset, top, half), self.button_rect(center_x + offset, top, half)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Choice {
        A,
        B,
        C,
    }

    fn list() -> ButtonList<Choice> {
        let mut list = ButtonList::new(vec![Button::new("a", Choice::A), Button::disabled("b", Choice::B), Button::new("c", Choice::C)]);
        list.set_rects([Rect::new(0.0, 20.0, 100.0, 10.0), Rect::new(0.0, 10.0, 100.0, 10.0), Rect::new(0.0, 0.0, 100.0, 9.0)]);
        list
    }

    #[test]
    fn keyboard_navigation_skips_disabled_buttons_and_wraps() {
        let mut list = list();
        list.move_selection(1);
        assert_eq!(list.selected(), Some(0));
        list.move_selection(1);
        assert_eq!(list.selected(), Some(2), "b is disabled");
        list.move_selection(1);
        assert_eq!(list.selected(), Some(0), "wraps around");
        list.move_selection(-1);
        assert_eq!(list.selected(), Some(2));
    }

    #[test]
    fn first_move_up_selects_the_last_button() {
        let mut list = list();
        list.move_selection(-1);
        assert_eq!(list.selected(), Some(2));
    }

    #[test]
    fn hover_selects_only_enabled_buttons() {
        let mut list = list();
        list.hover(glam::vec2(50.0, 25.0));
        assert_eq!(list.selected(), Some(0));
        list.hover(glam::vec2(50.0, 15.0));
        assert_eq!(list.selected(), None, "b is disabled");
        list.hover(glam::vec2(500.0, 500.0));
        assert_eq!(list.selected(), None);
    }

    #[test]
    fn clicks_trigger_only_enabled_buttons_under_the_cursor() {
        let mut list = list();
        let click = |list: &mut ButtonList<Choice>, y| list.mouse_input(Some(glam::vec2(50.0, y)), ElementState::Pressed, MouseButton::Left);
        assert_eq!(click(&mut list, 25.0), Some(Choice::A));
        assert_eq!(click(&mut list, 15.0), None);
        assert_eq!(click(&mut list, 5.0), Some(Choice::C));
        assert_eq!(list.mouse_input(Some(glam::vec2(50.0, 5.0)), ElementState::Released, MouseButton::Left), None);
        assert_eq!(list.mouse_input(Some(glam::vec2(50.0, 5.0)), ElementState::Pressed, MouseButton::Right), None);
        assert_eq!(list.mouse_input(None, ElementState::Pressed, MouseButton::Left), None);
    }
}
