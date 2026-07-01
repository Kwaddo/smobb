use ratzilla::event::{KeyCode, KeyEvent};
use ratzilla::ratatui::Terminal;
use ratzilla::{DomBackend, WebRenderer};
use smobb_core::content;
use smobb_core::effect::Mood;
use smobb_core::engine::Engine;
use smobb_core::save::SaveState;
use smobb_tui_core::{app::App, app::Key, ui};
use std::cell::RefCell;
use std::rc::Rc;

/// localStorage key the browser build persists progress under.
const SAVE_KEY: &str = "smobb_save";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load_save() -> Option<SaveState> {
    let json = local_storage()?.get_item(SAVE_KEY).ok()??;
    SaveState::from_json(&json)
}

fn write_save(state: &SaveState) {
    if let Some(storage) = local_storage() {
        let _ = storage.set_item(SAVE_KEY, &state.to_json());
    }
}

fn to_key(k: &KeyEvent) -> Option<Key> {
    // A bare Shift press arrives as Unidentified with the shift flag set.
    if k.shift && matches!(k.code, KeyCode::Unidentified) {
        return Some(Key::Shift);
    }
    Some(match k.code {
        KeyCode::Char('c') if k.ctrl => Key::CtrlC,
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Tab => Key::Tab,
        KeyCode::F(1) => Key::F1,
        KeyCode::F(2) => Key::F2,
        KeyCode::F(3) => Key::F3,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Esc => Key::Esc,
        _ => return None,
    })
}

/// Mirror the App's effect state onto <body> classes so CSS can react.
fn sync_effects(app: &App) {
    let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    else {
        return;
    };
    let cl = body.class_list();
    let set = |name: &str, on: bool| {
        if on {
            let _ = cl.add_1(name);
        } else {
            let _ = cl.remove_1(name);
        }
    };
    let flash = app.flash();
    set("fx-flash", flash.is_some());
    set("fx-glitch", app.glitch());
    set("reduced", app.reduced_effects());
    let mood = app.mood();
    set("mood-redalert", matches!(mood, Mood::RedAlert));
    set("mood-coldcyan", matches!(mood, Mood::ColdCyan));
    set("mood-corruption", matches!(mood, Mood::Corruption));
}

fn main() -> std::io::Result<()> {
    let mut engine = Engine::new(content::load());
    if let Some(state) = load_save() {
        engine.restore(state); // restore the notebook from localStorage
    }
    let app = Rc::new(RefCell::new(App::new(engine)));
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let app = app.clone();
        move |ke| {
            if let Some(key) = to_key(&ke) {
                // process the key, then persist if it changed progress
                // (drop the mutable borrow before saving)
                let snapshot = {
                    let mut a = app.borrow_mut();
                    a.key(key);
                    if a.dirty() {
                        a.clear_dirty();
                        Some(a.snapshot())
                    } else {
                        None
                    }
                };
                if let Some(state) = snapshot {
                    write_save(&state);
                }
            }
        }
    });

    terminal.draw_web(move |f| {
        {
            let mut a = app.borrow_mut();
            a.tick(web_time::Instant::now());
        }
        let a = app.borrow();
        sync_effects(&a);
        ui::render(f, &a);
    });

    Ok(())
}
