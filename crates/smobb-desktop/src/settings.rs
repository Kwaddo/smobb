//! Desktop-only window settings overlay: font size + fullscreen. Drawn on top
//! of the shared `ui::render` in the same draw pass, and driven by keys the
//! desktop intercepts while the panel is open (so `Esc` closes the panel rather
//! than quitting the app).

use ratatui_wgpu::ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};
use smobb_tui_core::app::Key;

pub const FONT_MIN: u32 = 8;
pub const FONT_MAX: u32 = 40;
pub const FONT_STEP: u32 = 2;
pub const FONT_DEFAULT: u32 = 16;

/// Which control the selection cursor is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    FontSize,
    Fullscreen,
}

/// What the caller must do in response to a key. The desktop applies these to
/// the real window/backend; the overlay only owns its own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    FontChanged(u32),
    ToggleFullscreen,
    Close,
}

pub struct Settings {
    pub open: bool,
    pub sel: Row,
    pub font_px: u32,
    pub fullscreen: bool,
}

impl Settings {
    pub fn new(font_px: u32, fullscreen: bool) -> Self {
        Self {
            open: false,
            sel: Row::FontSize,
            font_px,
            fullscreen,
        }
    }

    /// Open the panel, syncing its shown values from the live window state.
    pub fn open(&mut self, font_px: u32, fullscreen: bool) {
        self.open = true;
        self.font_px = font_px;
        self.fullscreen = fullscreen;
    }

    /// Handle a key while the panel is open. Returns the action the desktop must
    /// apply. Only call this when `self.open`.
    pub fn handle_key(&mut self, key: Key) -> Action {
        match key {
            Key::Up => {
                self.sel = Row::FontSize;
                Action::None
            }
            Key::Down => {
                self.sel = Row::Fullscreen;
                Action::None
            }
            Key::Left | Key::Char('-') | Key::Char('_') => self.nudge_font(false),
            Key::Right | Key::Char('+') | Key::Char('=') => self.nudge_font(true),
            Key::Enter | Key::Char(' ') => {
                if self.sel == Row::Fullscreen {
                    self.fullscreen = !self.fullscreen;
                    Action::ToggleFullscreen
                } else {
                    Action::None
                }
            }
            Key::Esc => {
                self.open = false;
                Action::Close
            }
            _ => Action::None,
        }
    }

    /// Step the font size when the FontSize row is selected. Clamped to
    /// `[FONT_MIN, FONT_MAX]`; returns `FontChanged` only on an actual change.
    fn nudge_font(&mut self, up: bool) -> Action {
        if self.sel != Row::FontSize {
            return Action::None;
        }
        let next = if up {
            (self.font_px + FONT_STEP).min(FONT_MAX)
        } else {
            self.font_px.saturating_sub(FONT_STEP).max(FONT_MIN)
        };
        if next != self.font_px {
            self.font_px = next;
            Action::FontChanged(next)
        } else {
            Action::None
        }
    }

    /// Draw the centered panel over the current frame.
    pub fn render(&self, f: &mut Frame) {
        let area = f.area();
        let w = 34u16.min(area.width);
        let h = 9u16.min(area.height);
        let rect = Rect {
            x: area.x + area.width.saturating_sub(w) / 2,
            y: area.y + area.height.saturating_sub(h) / 2,
            width: w,
            height: h,
        };
        f.render_widget(Clear, rect);

        let block = Block::bordered()
            .title(" Settings ")
            .border_style(Style::default().fg(Color::Rgb(180, 220, 255)));
        let inner = block.inner(rect);
        f.render_widget(block, rect);

        let row_style = |r: Row| {
            if self.sel == r {
                Style::default()
                    .fg(Color::Rgb(16, 16, 16))
                    .bg(Color::Rgb(180, 220, 255))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(220, 220, 220))
            }
        };

        let font = Line::from(Span::styled(
            format!(" Font size   {:>2}   [-] [+] ", self.font_px),
            row_style(Row::FontSize),
        ));
        let full = Line::from(Span::styled(
            format!(
                " Fullscreen  [{}]         ",
                if self.fullscreen { "x" } else { " " }
            ),
            row_style(Row::Fullscreen),
        ));
        let foot = Line::from(Span::styled(
            " up/down select  -/+ adjust  Esc close ",
            Style::default()
                .fg(Color::Rgb(110, 110, 110))
                .add_modifier(Modifier::DIM),
        ));

        let body = Paragraph::new(vec![
            Line::from(""),
            font,
            Line::from(""),
            full,
            Line::from(""),
            foot,
        ]);
        f.render_widget(body, inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_down_move_selection() {
        let mut s = Settings::new(FONT_DEFAULT, false);
        assert_eq!(s.sel, Row::FontSize);
        assert_eq!(s.handle_key(Key::Down), Action::None);
        assert_eq!(s.sel, Row::Fullscreen);
        assert_eq!(s.handle_key(Key::Up), Action::None);
        assert_eq!(s.sel, Row::FontSize);
    }

    #[test]
    fn font_adjusts_and_clamps() {
        let mut s = Settings::new(FONT_DEFAULT, false);
        assert_eq!(s.handle_key(Key::Char('+')), Action::FontChanged(18));
        assert_eq!(s.handle_key(Key::Char('-')), Action::FontChanged(16));

        // Walk down to the floor and confirm it stops there (no change at min).
        let mut last = s.font_px;
        for _ in 0..20 {
            if let Action::FontChanged(px) = s.handle_key(Key::Left) {
                last = px;
            }
        }
        assert_eq!(s.font_px, FONT_MIN);
        assert_eq!(last, FONT_MIN);
        assert_eq!(s.handle_key(Key::Left), Action::None, "no change at floor");

        // And up to the ceiling.
        for _ in 0..40 {
            let _ = s.handle_key(Key::Right);
        }
        assert_eq!(s.font_px, FONT_MAX);
        assert_eq!(
            s.handle_key(Key::Right),
            Action::None,
            "no change at ceiling"
        );
    }

    #[test]
    fn font_keys_ignored_off_font_row() {
        let mut s = Settings::new(FONT_DEFAULT, false);
        s.sel = Row::Fullscreen;
        assert_eq!(s.handle_key(Key::Right), Action::None);
        assert_eq!(s.font_px, FONT_DEFAULT);
    }

    #[test]
    fn enter_toggles_fullscreen_on_its_row() {
        let mut s = Settings::new(FONT_DEFAULT, false);
        // On the font row, Enter does nothing.
        assert_eq!(s.handle_key(Key::Enter), Action::None);
        assert!(!s.fullscreen);

        s.sel = Row::Fullscreen;
        assert_eq!(s.handle_key(Key::Enter), Action::ToggleFullscreen);
        assert!(s.fullscreen);
        assert_eq!(s.handle_key(Key::Enter), Action::ToggleFullscreen);
        assert!(!s.fullscreen);
    }

    #[test]
    fn esc_closes() {
        let mut s = Settings::new(FONT_DEFAULT, false);
        s.open = true;
        assert_eq!(s.handle_key(Key::Esc), Action::Close);
        assert!(!s.open);
    }
}
