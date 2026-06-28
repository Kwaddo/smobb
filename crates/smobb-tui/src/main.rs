use ratatui::crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        ModifierKeyCode, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::supports_keyboard_enhancement,
};
use smobb_core::content;
use smobb_core::engine::Engine;
use smobb_tui_core::{app::App, app::Key, ui};
use std::io::{stdout, Write};
use std::time::Duration;
use web_time::Instant;

fn to_key(k: &KeyEvent) -> Option<Key> {
    let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
    Some(match k.code {
        KeyCode::Char('c') if ctrl => Key::CtrlC,
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Tab => Key::Tab,
        KeyCode::F(1) => Key::F1,
        KeyCode::F(2) => Key::F2,
        KeyCode::F(3) => Key::F3,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Esc => Key::Esc,
        // Bare Shift press/repeat — only delivered when keyboard enhancement is
        // active (modern terminals). Drives hold-Shift fast-forward.
        KeyCode::Modifier(ModifierKeyCode::LeftShift)
        | KeyCode::Modifier(ModifierKeyCode::RightShift) => Key::Shift,
        _ => return None,
    })
}

/// Ask the terminal to report modifier-key presses + event types (Press/Repeat),
/// so hold-Shift fast-forward works. Returns whether the flags were pushed
/// (so we can pop them on exit). No-ops gracefully on terminals that don't
/// support the Kitty keyboard protocol (e.g. Apple Terminal).
fn enable_key_enhancement() -> bool {
    if matches!(supports_keyboard_enhancement(), Ok(true)) {
        let flags = KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES;
        execute!(stdout(), PushKeyboardEnhancementFlags(flags)).is_ok()
    } else {
        false
    }
}

/// Progress save file, kept next to where the game is launched.
const SAVE_PATH: &str = "smobb-save.json";

fn load_save() -> Option<smobb_core::save::SaveState> {
    std::fs::read_to_string(SAVE_PATH)
        .ok()
        .and_then(|t| smobb_core::save::SaveState::from_json(&t))
}

fn write_save(app: &App) {
    let _ = std::fs::write(SAVE_PATH, app.snapshot().to_json());
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let enhanced = enable_key_enhancement();
    let mut engine = Engine::new(content::load());
    if let Some(state) = load_save() {
        engine.restore(state); // restore the notebook before the intro runs
    }
    let mut app = App::new(engine);
    let res = run(&mut terminal, &mut app);
    write_save(&app); // final save on exit
    if enhanced {
        let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
        let _ = stdout().flush();
    }
    ratatui::restore();
    res
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let tick = Duration::from_millis(30);
    // debounce window so a toggle key's auto-repeat doesn't flip it several times
    let debounce = Duration::from_millis(250);
    let mut last_toggle: Option<(Key, Instant)> = None;
    while !app.quit() {
        terminal.draw(|f| ui::render(f, app))?;
        if event::poll(tick)? {
            if let Event::Key(k) = event::read()? {
                let press = k.kind == KeyEventKind::Press;
                let repeat = k.kind == KeyEventKind::Repeat;
                if let (true, Some(key)) = (press || repeat, to_key(&k)) {
                    // the fast-forward keys (Space/Shift) want every repeat; all
                    // other keys act on the initial press only (avoids repeats
                    // re-toggling things like F1/F2)
                    let is_fast = matches!(key, Key::Shift | Key::Char(' '));
                    if repeat && !is_fast {
                        // ignore auto-repeat for non-fast keys
                    } else {
                        let now = Instant::now();
                        // extra guard: terminals without enhancement report
                        // auto-repeat as fresh Presses, so debounce the toggles
                        let toggling = matches!(key, Key::F1 | Key::F2 | Key::F3);
                        let recent = toggling
                            && matches!(last_toggle, Some((lk, lt))
                                if lk == key && now.duration_since(lt) < debounce);
                        if !recent {
                            if toggling {
                                last_toggle = Some((key, now));
                            }
                            app.key(key);
                        }
                    }
                }
            }
        }
        app.tick(Instant::now());
        if app.dirty() {
            write_save(app); // crash-safe: persist after every change
            app.clear_dirty();
        }
    }
    Ok(())
}
