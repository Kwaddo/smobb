use smobb_tui_core::app::{Focus, Key};
use smobb_tui_core::ui::STORY_PCT;
use winit::keyboard::{Key as WKey, ModifiersState, NamedKey};

/// Which pane sits under cursor x (physical px) given the window width. The
/// sidebar occupies the right `SIDEBAR_PCT`% of the body; everything left is the
/// story. Always `Story` when the sidebar is hidden. The ~1-cell border error is
/// irrelevant for choosing a pane.
pub fn pane_at(window_w: f64, cursor_x: f64, sidebar_visible: bool) -> Focus {
    if !sidebar_visible {
        return Focus::Story;
    }
    let boundary = window_w * STORY_PCT as f64 / 100.0;
    if cursor_x >= boundary {
        Focus::Sidebar
    } else {
        Focus::Story
    }
}

/// Translate a winit logical key (+ current modifiers) into the app's `Key`.
/// Returns `None` for keys the app ignores.
pub fn to_key(logical: &WKey, mods: ModifiersState) -> Option<Key> {
    Some(match logical {
        WKey::Named(NamedKey::Enter) => Key::Enter,
        WKey::Named(NamedKey::Backspace) => Key::Backspace,
        WKey::Named(NamedKey::Tab) => Key::Tab,
        WKey::Named(NamedKey::Escape) => Key::Esc,
        WKey::Named(NamedKey::ArrowUp) => Key::Up,
        WKey::Named(NamedKey::ArrowDown) => Key::Down,
        WKey::Named(NamedKey::ArrowLeft) => Key::Left,
        WKey::Named(NamedKey::ArrowRight) => Key::Right,
        WKey::Named(NamedKey::PageUp) => Key::PageUp,
        WKey::Named(NamedKey::PageDown) => Key::PageDown,
        WKey::Named(NamedKey::F1) => Key::F1,
        WKey::Named(NamedKey::F2) => Key::F2,
        WKey::Named(NamedKey::F3) => Key::F3,
        WKey::Named(NamedKey::Shift) => Key::Shift,
        WKey::Named(NamedKey::Space) => Key::Char(' '),
        WKey::Character(s) => {
            let c = s.chars().next()?;
            if mods.control_key() && c.eq_ignore_ascii_case(&'c') {
                Key::CtrlC
            } else {
                Key::Char(c)
            }
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_at_picks_pane_under_cursor() {
        // Hidden sidebar: always the story, regardless of x.
        assert_eq!(pane_at(1000.0, 990.0, false), Focus::Story);
        // 70/30 split at x=700 for a 1000px window.
        assert_eq!(pane_at(1000.0, 0.0, true), Focus::Story);
        assert_eq!(pane_at(1000.0, 699.0, true), Focus::Story);
        assert_eq!(pane_at(1000.0, 700.0, true), Focus::Sidebar);
        assert_eq!(pane_at(1000.0, 999.0, true), Focus::Sidebar);
    }

    #[test]
    fn maps_named_keys() {
        let m = ModifiersState::empty();
        assert_eq!(to_key(&WKey::Named(NamedKey::Enter), m), Some(Key::Enter));
        assert_eq!(
            to_key(&WKey::Named(NamedKey::ArrowLeft), m),
            Some(Key::Left)
        );
        assert_eq!(
            to_key(&WKey::Named(NamedKey::ArrowRight), m),
            Some(Key::Right)
        );
        assert_eq!(
            to_key(&WKey::Named(NamedKey::Space), m),
            Some(Key::Char(' '))
        );
        assert_eq!(to_key(&WKey::Named(NamedKey::F2), m), Some(Key::F2));
    }

    #[test]
    fn maps_chars_and_ctrl_c() {
        let none = ModifiersState::empty();
        assert_eq!(
            to_key(&WKey::Character("a".into()), none),
            Some(Key::Char('a'))
        );
        assert_eq!(
            to_key(&WKey::Character("c".into()), ModifiersState::CONTROL),
            Some(Key::CtrlC)
        );
    }
}
