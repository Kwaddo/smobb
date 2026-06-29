# Standalone Desktop App + Scrollable Sidebar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the journal/options sidebar scrollable (focus-follows-sidebar) and add a `smobb-desktop` crate that renders the existing ratatui UI in its own native GPU window — no terminal.

**Architecture:** The backend-agnostic `smobb-tui-core` (`App` + `ui::render`) is reused unchanged by a new winit + `ratatui-wgpu` frontend. Sidebar scrolling adds focus state to `App` and a `line_count`-based clamp to `ui::render`, shared by all three frontends.

**Tech Stack:** Rust, ratatui 0.29, `ratatui-wgpu = "0.4"` (wgpu backend, pins ratatui 0.29), `winit = "0.30"`, `pollster`, `dejavu` (embedded font), `dirs`.

## Global Constraints

- ratatui stays at `0.29` across the whole workspace. Do NOT bump to 0.30 / the split `ratatui-core` crates. (`ratatui-wgpu 0.4.1` pins `ratatui ^0.29`; the 0.5 line requires 0.30 and is out of scope.)
- The new crate is **purely additive**. Do not change `smobb-web` behavior. The only allowed change to `smobb-tui` is mapping the new `Key::Left` / `Key::Right`.
- `Key` is matched exhaustively only in `App::key` (`smobb-tui-core/src/app.rs`). `to_key` in `smobb-tui` and `smobb-web` use a non-exhaustive `_ => return None`, so adding `Key` variants is safe there.
- Sidebar default scroll position is **top**; story default stays **bottom** (auto-follow latest).
- Workspace MSRV floor for the new crate: `rust-version = "1.75"` (ratatui-wgpu 0.4.1). Existing crates keep `1.74`.
- Commit after every task. Co-author trailer on commits: `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`.

---

### Task 1: Sidebar scroll-focus state + `Key::Left`/`Key::Right` routing

**Files:**
- Modify: `crates/smobb-tui-core/src/app.rs`
- Test: `crates/smobb-tui-core/src/app.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Produces:
  - `pub enum Focus { Story, Sidebar }` (derives `Debug, Clone, Copy, PartialEq, Eq`)
  - `Key::Left`, `Key::Right` variants
  - `App::scroll_focus(&self) -> Focus`
  - `App::sidebar_scroll(&self) -> u16`
  - `App::sidebar_user_scrolled(&self) -> bool`

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)] mod tests` block in `crates/smobb-tui-core/src/app.rs`:

```rust
#[test]
fn right_focuses_sidebar_left_focuses_story() {
    let mut app = app_fixture();
    app.tick(web_time::Instant::now());
    assert_eq!(app.scroll_focus(), Focus::Story); // default
    app.key(Key::Right); // sidebar visible by default
    assert_eq!(app.scroll_focus(), Focus::Sidebar);
    app.key(Key::Left);
    assert_eq!(app.scroll_focus(), Focus::Story);
}

#[test]
fn right_does_nothing_when_sidebar_hidden() {
    let mut app = app_fixture();
    app.tick(web_time::Instant::now());
    app.key(Key::F1); // hide sidebar
    assert!(!app.sidebar_visible());
    app.key(Key::Right);
    assert_eq!(app.scroll_focus(), Focus::Story);
}

#[test]
fn hiding_sidebar_returns_focus_to_story() {
    let mut app = app_fixture();
    app.tick(web_time::Instant::now());
    app.key(Key::Right);
    assert_eq!(app.scroll_focus(), Focus::Sidebar);
    app.key(Key::F1); // hide
    assert_eq!(app.scroll_focus(), Focus::Story);
}

#[test]
fn tab_focuses_sidebar_and_resets_its_scroll() {
    let mut app = app_fixture();
    app.tick(web_time::Instant::now());
    app.key(Key::Right);
    app.key(Key::Down); // scroll sidebar down
    assert!(app.sidebar_scroll() > 0);
    app.key(Key::Tab); // switch tab -> reset offset, keep sidebar focus
    assert_eq!(app.sidebar_scroll(), 0);
    assert!(!app.sidebar_user_scrolled());
    assert_eq!(app.scroll_focus(), Focus::Sidebar);
}

#[test]
fn arrows_route_to_focused_pane() {
    let mut app = app_fixture();
    app.tick(web_time::Instant::now());
    // story focus: Down moves story, not sidebar
    let s0 = app.scroll();
    app.key(Key::Down);
    assert_eq!(app.scroll(), s0 + 1);
    assert_eq!(app.sidebar_scroll(), 0);
    // sidebar focus: Down moves sidebar, marks user_scrolled, leaves story
    app.key(Key::Right);
    let story_before = app.scroll();
    app.key(Key::Down);
    assert_eq!(app.sidebar_scroll(), 1);
    assert!(app.sidebar_user_scrolled());
    assert_eq!(app.scroll(), story_before);
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-tui-core --lib`
Expected: FAIL — `Focus` not found, `Key::Left`/`Key::Right` not found, missing getters.

- [ ] **Step 3: Add the `Focus` enum**

In `crates/smobb-tui-core/src/app.rs`, next to the `Tab` enum (near line 7):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Story,
    Sidebar,
}
```

- [ ] **Step 4: Add `Left`/`Right` to the `Key` enum**

In the `pub enum Key { ... }` block, add the two variants after `Down`:

```rust
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
```

- [ ] **Step 5: Add the new fields to `App`**

In `struct App { ... }`, after `user_scrolled: bool,`:

```rust
    scroll_focus: Focus,
    sidebar_scroll: u16,
    sidebar_user_scrolled: bool,
```

In `App::new`, in the struct literal after `user_scrolled: false,`:

```rust
            scroll_focus: Focus::Story,
            sidebar_scroll: 0,
            sidebar_user_scrolled: false,
```

- [ ] **Step 6: Update key handling**

In `App::key`, replace the `Key::Tab`, `Key::F1`, and the four scroll arms with:

```rust
            Key::Tab => {
                self.tab = match self.tab {
                    Tab::Options => Tab::Journal,
                    Tab::Journal => Tab::Options,
                };
                // tabbing to a pane focuses it and starts at the top
                self.scroll_focus = Focus::Sidebar;
                self.sidebar_scroll = 0;
                self.sidebar_user_scrolled = false;
            }
            Key::F1 => {
                self.sidebar_visible = !self.sidebar_visible;
                if !self.sidebar_visible {
                    self.scroll_focus = Focus::Story;
                }
            }
            Key::Left => self.scroll_focus = Focus::Story,
            Key::Right => {
                if self.sidebar_visible {
                    self.scroll_focus = Focus::Sidebar;
                }
            }
            Key::Up => match self.scroll_focus {
                Focus::Story => {
                    self.user_scrolled = true;
                    self.scroll = self.scroll.saturating_sub(1);
                }
                Focus::Sidebar => {
                    self.sidebar_user_scrolled = true;
                    self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1);
                }
            },
            Key::Down => match self.scroll_focus {
                Focus::Story => self.scroll = self.scroll.saturating_add(1),
                Focus::Sidebar => {
                    self.sidebar_user_scrolled = true;
                    self.sidebar_scroll = self.sidebar_scroll.saturating_add(1);
                }
            },
            Key::PageUp => match self.scroll_focus {
                Focus::Story => {
                    self.user_scrolled = true;
                    self.scroll = self.scroll.saturating_sub(5);
                }
                Focus::Sidebar => {
                    self.sidebar_user_scrolled = true;
                    self.sidebar_scroll = self.sidebar_scroll.saturating_sub(5);
                }
            },
            Key::PageDown => match self.scroll_focus {
                Focus::Story => self.scroll = self.scroll.saturating_add(5),
                Focus::Sidebar => {
                    self.sidebar_user_scrolled = true;
                    self.sidebar_scroll = self.sidebar_scroll.saturating_add(5);
                }
            },
```

(Note the sidebar's default position is top, so **any** sidebar move — up or down — sets `sidebar_user_scrolled`; otherwise a down-scroll would snap back to the top default.)

- [ ] **Step 7: Add the getters**

Next to `pub fn user_scrolled(&self) -> bool`:

```rust
    pub fn scroll_focus(&self) -> Focus {
        self.scroll_focus
    }
    pub fn sidebar_scroll(&self) -> u16 {
        self.sidebar_scroll
    }
    pub fn sidebar_user_scrolled(&self) -> bool {
        self.sidebar_user_scrolled
    }
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test -p smobb-tui-core --lib`
Expected: PASS (all new tests + existing ones, e.g. `keys_toggle_tab_sidebar_and_quit`).

- [ ] **Step 9: Commit**

```bash
git add crates/smobb-tui-core/src/app.rs
git commit -m "feat(tui-core): sidebar scroll-focus state + Left/Right keys

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 2: Render the sidebar scroll + focus highlight (`ui.rs`)

**Files:**
- Modify: `crates/smobb-tui-core/src/ui.rs`
- Test: `crates/smobb-tui-core/src/ui.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Consumes: `App::scroll_focus`, `App::sidebar_scroll`, `App::sidebar_user_scrolled`, `Focus` (Task 1).

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)] mod tests` block in `crates/smobb-tui-core/src/ui.rs`:

```rust
#[test]
fn sidebar_scrolls_past_top_when_focused() {
    use crate::app::{Focus, Key};
    // 20 distinct option labels (via Enter steps) -> tall Options sidebar.
    let branches: String = (1..=20)
        .map(|i| format!(r#"Branch(keys:["k{0:02}"], target: Steps([ Enter("p{0:02}") ]))"#, i))
        .collect::<Vec<_>>()
        .join(", ");
    let locs: String = (1..=20)
        .map(|i| format!(r#""p{0:02}": ( commands: [] )"#, i))
        .collect::<Vec<_>>()
        .join(", ");
    let content_str = format!(
        r#"ContentPart(
            locations: {{ "hub": ( prompt:[Line(Smobb,"PROMPT")], commands: [ {} ] ), {} }},
            scenes: {{}},
        )"#,
        branches, locs
    );
    let c = content::from_str(&[content_str.as_str()]);
    let mut app = App::new(Engine::new(c));
    app.tick(web_time::Instant::now()); // reveal prompt + await; default tab = Options

    // Small terminal so the sidebar (30% width) overflows its viewport.
    let backend = TestBackend::new(70, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    // Not scrolled: the very first options line "You can:" is visible.
    terminal.draw(|f| render(f, &app)).unwrap();
    let dump: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
    assert!(dump.contains("You can:"), "top of sidebar visible before scrolling");

    // Focus the sidebar and scroll well past the top.
    app.key(Key::Right);
    assert_eq!(app.scroll_focus(), Focus::Sidebar);
    for _ in 0..6 {
        app.key(Key::PageDown);
    }
    terminal.draw(|f| render(f, &app)).unwrap();
    let dump: String = terminal.backend().buffer().content().iter().map(|c| c.symbol()).collect();
    assert!(!dump.contains("You can:"), "scrolled past the top header");
    assert!(dump.contains("- "), "still showing clamped option items, not blank");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-tui-core --lib sidebar_scrolls_past_top_when_focused`
Expected: FAIL — "You can:" still visible after scrolling (sidebar isn't scrolled yet).

- [ ] **Step 3: Add a focus-aware border-style helper**

At the top of `crates/smobb-tui-core/src/ui.rs`, after the `use` block, add:

```rust
/// Border style for a pane, brightened when it currently has scroll focus.
fn pane_border_style(focused: bool) -> Style {
    if focused {
        Style::default()
            .fg(RColor::Rgb(180, 220, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(RColor::Rgb(90, 90, 90))
    }
}
```

- [ ] **Step 4: Add a sidebar render helper**

Add this function to `crates/smobb-tui-core/src/ui.rs` (above `pub fn render`):

```rust
/// Render the Options/Journal sidebar into `area`, applying the same
/// `line_count`-based scroll clamp the story uses. Sidebar defaults to the top
/// (read top-down) unless the user has scrolled.
fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    use crate::app::Focus;
    let (title, outs) = match app.tab() {
        crate::app::Tab::Options => (" OPTIONS ", app.options_lines()),
        crate::app::Tab::Journal => (" JOURNAL ", app.journal_lines()),
    };
    let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
    let inner_w = area.width.saturating_sub(2);
    let inner_h = area.height.saturating_sub(2);
    let base = Paragraph::new(lines).wrap(Wrap { trim: false });
    let total = base.line_count(inner_w) as u16;
    let max_scroll = total.saturating_sub(inner_h);
    let y = if app.sidebar_user_scrolled() {
        app.sidebar_scroll().min(max_scroll)
    } else {
        0
    };
    let focused = app.scroll_focus() == Focus::Sidebar;
    let block = Block::bordered()
        .title(title)
        .border_style(pane_border_style(focused));
    f.render_widget(base.block(block).scroll((y, 0)), area);
}
```

- [ ] **Step 5: Use the helper in both sidebar render sites**

In `render`, in the **banner** branch, replace the existing `if let Some(sa) = sidebar_area { ... }` block (the one building `lines`/`Paragraph` inside the banner path) with:

```rust
        if let Some(sa) = sidebar_area {
            render_sidebar(f, app, sa);
        }
```

In the main (non-banner) path, replace the `// sidebar` block (`if let Some(sa) = sidebar_area { ... }`) with:

```rust
    // sidebar
    if let Some(sa) = sidebar_area {
        render_sidebar(f, app, sa);
    }
```

- [ ] **Step 6: Highlight the story border by focus**

In the main path, change the story block construction from:

```rust
    let story = story_base.block(Block::bordered()).scroll((y, 0));
```

to:

```rust
    let story_focused = app.scroll_focus() == crate::app::Focus::Story;
    let story = story_base
        .block(Block::bordered().border_style(pane_border_style(story_focused)))
        .scroll((y, 0));
```

- [ ] **Step 7: Update the input hint line**

In the main path, change the `hint` format string from:

```rust
    let hint = format!(
        "[Tab] sidebar  [hold Space] fast  [F2] instant  {}  ['leave' exits]  [Esc] quit",
        fx_label
    );
```

to:

```rust
    let hint = format!(
        "[Tab] sidebar  [<-/-> focus]  [hold Space] fast  [F2] instant  {}  ['leave' exits]  [Esc] quit",
        fx_label
    );
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test -p smobb-tui-core --lib`
Expected: PASS (new sidebar test + existing render tests such as `renders_title_story_and_sidebar`, `auto_scrolls_to_latest_line`).

- [ ] **Step 9: Commit**

```bash
git add crates/smobb-tui-core/src/ui.rs
git commit -m "feat(tui-core): scrollable sidebar with focus highlight

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 3: Map `Left`/`Right` in the terminal binary

**Files:**
- Modify: `crates/smobb-tui/src/main.rs:16-38` (`to_key`)
- Test: `crates/smobb-tui/src/main.rs` (inline `#[cfg(test)]` module)

**Interfaces:**
- Consumes: `Key::Left`, `Key::Right` (Task 1).

- [ ] **Step 1: Write the failing test**

Add to the bottom of `crates/smobb-tui/src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn maps_left_and_right_arrows() {
        let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
        assert_eq!(to_key(&left), Some(Key::Left));
        assert_eq!(to_key(&right), Some(Key::Right));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-tui`
Expected: FAIL — `to_key` returns `None` for `Left`/`Right` (no arm), assertion fails.

- [ ] **Step 3: Add the mapping arms**

In `to_key`, after the `KeyCode::Down => Key::Down,` arm:

```rust
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p smobb-tui`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-tui/src/main.rs
git commit -m "feat(tui): map Left/Right arrows for sidebar focus

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 4: Scaffold the `smobb-desktop` crate (manifest + save module)

**Files:**
- Modify: `Cargo.toml` (workspace `members`)
- Create: `crates/smobb-desktop/Cargo.toml`
- Create: `crates/smobb-desktop/src/save.rs`
- Create: `crates/smobb-desktop/src/main.rs` (temporary stub, replaced in Task 6)
- Test: `crates/smobb-desktop/src/save.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Produces:
  - `save::save_path() -> std::path::PathBuf`
  - `save::load_save() -> Option<smobb_core::save::SaveState>`
  - `save::write_save(app: &smobb_tui_core::app::App)`

- [ ] **Step 1: Add the crate to the workspace**

In the root `Cargo.toml`, add `"crates/smobb-desktop"` to `members`:

```toml
[workspace]
members = ["crates/smobb-core", "crates/smobb-tui-core", "crates/smobb-tui", "crates/smobb-web", "crates/smobb-desktop"]
default-members = ["crates/smobb-core", "crates/smobb-tui-core", "crates/smobb-tui"]
resolver = "2"
```

- [ ] **Step 2: Create `crates/smobb-desktop/Cargo.toml`**

```toml
[package]
name = "smobb-desktop"
version = "2.0.1"
edition = "2021"
rust-version = "1.75"

[dependencies]
smobb-core = { path = "../smobb-core" }
smobb-tui-core = { path = "../smobb-tui-core" }
ratatui-wgpu = "0.4"
winit = "0.30"
pollster = "0.4"
dejavu = "2.37"
dirs = "5"
web-time = "1"

[package.metadata.bundle]
name = "S.M.O.B.B."
identifier = "com.smobb.desktop"
```

- [ ] **Step 3: Create the temporary `main.rs` stub**

`crates/smobb-desktop/src/main.rs`:

```rust
mod save;

fn main() {
    println!("smobb-desktop scaffold");
}
```

- [ ] **Step 4: Write the failing test + `save.rs`**

`crates/smobb-desktop/src/save.rs`:

```rust
use smobb_core::save::SaveState;
use smobb_tui_core::app::App;
use std::fs;
use std::path::PathBuf;

const SAVE_FILE: &str = "smobb-save.json";

/// Per-user data path for the desktop save, e.g.
/// `~/Library/Application Support/smobb/smobb-save.json` on macOS. A bundled
/// `.app` has no meaningful cwd, so we never write relative to it.
pub fn save_path() -> PathBuf {
    let dir = dirs::data_dir()
        .map(|d| d.join("smobb"))
        .unwrap_or_else(|| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join(SAVE_FILE)
}

pub fn load_save() -> Option<SaveState> {
    let txt = fs::read_to_string(save_path()).ok()?;
    SaveState::from_json(&txt)
}

pub fn write_save(app: &App) {
    let _ = fs::write(save_path(), app.snapshot().to_json());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_path_is_under_smobb_dir_and_named() {
        let p = save_path();
        assert!(p.ends_with("smobb/smobb-save.json"));
    }
}
```

- [ ] **Step 5: Build and test**

Run: `cargo test -p smobb-desktop`
Expected: PASS. First build downloads `ratatui-wgpu`, `winit`, `wgpu`, `dejavu`, etc. — it will take a while but must compile cleanly.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock crates/smobb-desktop
git commit -m "feat(desktop): scaffold smobb-desktop crate + save module

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 5: winit → `Key` input mapping (`input.rs`)

**Files:**
- Create: `crates/smobb-desktop/src/input.rs`
- Modify: `crates/smobb-desktop/src/main.rs` (add `mod input;`)
- Test: `crates/smobb-desktop/src/input.rs` (inline `#[cfg(test)]`)

**Interfaces:**
- Produces: `input::to_key(logical: &winit::keyboard::Key, mods: winit::keyboard::ModifiersState) -> Option<smobb_tui_core::app::Key>`

- [ ] **Step 1: Write `input.rs` with failing tests**

`crates/smobb-desktop/src/input.rs`:

```rust
use smobb_tui_core::app::Key;
use winit::keyboard::{Key as WKey, ModifiersState, NamedKey};

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
    fn maps_named_keys() {
        let m = ModifiersState::empty();
        assert_eq!(to_key(&WKey::Named(NamedKey::Enter), m), Some(Key::Enter));
        assert_eq!(to_key(&WKey::Named(NamedKey::ArrowLeft), m), Some(Key::Left));
        assert_eq!(to_key(&WKey::Named(NamedKey::ArrowRight), m), Some(Key::Right));
        assert_eq!(to_key(&WKey::Named(NamedKey::Space), m), Some(Key::Char(' ')));
        assert_eq!(to_key(&WKey::Named(NamedKey::F2), m), Some(Key::F2));
    }

    #[test]
    fn maps_chars_and_ctrl_c() {
        let none = ModifiersState::empty();
        assert_eq!(to_key(&WKey::Character("a".into()), none), Some(Key::Char('a')));
        assert_eq!(
            to_key(&WKey::Character("c".into()), ModifiersState::CONTROL),
            Some(Key::CtrlC)
        );
    }
}
```

- [ ] **Step 2: Register the module**

In `crates/smobb-desktop/src/main.rs`, add `mod input;` next to `mod save;`.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cargo test -p smobb-desktop`
Expected: PASS (no GPU/window needed for these pure tests).

- [ ] **Step 4: Commit**

```bash
git add crates/smobb-desktop/src/input.rs crates/smobb-desktop/src/main.rs
git commit -m "feat(desktop): winit->Key input mapping

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 6: Window + event loop (`main.rs`)

**Files:**
- Modify: `crates/smobb-desktop/src/main.rs` (replace stub with the full app)

**Interfaces:**
- Consumes: `input::to_key`, `save::{load_save, write_save}`, `smobb_tui_core::{app::App, app::Key, ui}`, `smobb_core::{content, engine::Engine}`, `ratatui_wgpu::{Builder, Dimensions, Font, WgpuBackend}`, `dejavu`.

- [ ] **Step 1: Replace `main.rs` with the full implementation**

`crates/smobb-desktop/src/main.rs`:

```rust
// Release builds run as a GUI app with no console window (Windows). Debug keeps
// the console so logs are visible.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod input;
mod save;

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::{Duration, Instant as StdInstant};

use ratatui_wgpu::ratatui::Terminal;
use ratatui_wgpu::{Builder, Dimensions, Font, WgpuBackend};
use smobb_core::content;
use smobb_core::engine::Engine;
use smobb_tui_core::app::{App, Key};
use smobb_tui_core::ui;
use web_time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::ModifiersState;
use winit::window::{Window, WindowAttributes, WindowId};

const TICK: Duration = Duration::from_millis(30);

type DesktopTerminal = Terminal<WgpuBackend<'static, 'static>>;

struct AppHandler {
    app: App,
    window: Option<Arc<Window>>,
    terminal: Option<DesktopTerminal>,
    mods: ModifiersState,
}

impl AppHandler {
    fn new(app: App) -> Self {
        Self {
            app,
            window: None,
            terminal: None,
            mods: ModifiersState::empty(),
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return; // already initialised (e.g. resume from suspend)
        }
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("S.M.O.B.B."))
                .expect("create window"),
        );
        let size = window.inner_size();
        let width = NonZeroU32::new(size.width.max(1)).unwrap();
        let height = NonZeroU32::new(size.height.max(1)).unwrap();

        let backend = pollster::block_on(
            Builder::from_font(Font::new(dejavu::sans_mono::regular()).expect("mono font"))
                // proportional DejaVu Sans as fallback for symbol glyphs the
                // mono face may lack (e.g. the pencil/gear dingbats).
                .with_fonts([Font::new(dejavu::sans::regular()).expect("fallback font")])
                .with_font_size_px(16)
                .with_width_and_height(Dimensions { width, height })
                .build_with_target(window.clone()),
        )
        .expect("build wgpu backend");

        self.terminal = Some(Terminal::new(backend).expect("terminal"));
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                save::write_save(&self.app);
                event_loop.exit();
            }
            WindowEvent::ModifiersChanged(m) => self.mods = m.state(),
            WindowEvent::Resized(size) => {
                if let Some(t) = self.terminal.as_mut() {
                    t.backend_mut().resize(size.width, size.height);
                }
                if let Some(w) = self.window.as_ref() {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }
                if let Some(key) = input::to_key(&event.logical_key, self.mods) {
                    // Fast-forward keys want auto-repeat; other keys act on the
                    // initial press only (mirrors the terminal frontend).
                    let is_fast = matches!(key, Key::Shift | Key::Char(' '));
                    if event.repeat && !is_fast {
                        return;
                    }
                    self.app.key(key);
                    if let Some(w) = self.window.as_ref() {
                        w.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // disjoint field borrows so the draw closure can read `app`
                let Self { terminal, app, .. } = self;
                if let Some(t) = terminal.as_mut() {
                    let _ = t.draw(|f| ui::render(f, app));
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.tick(Instant::now());
        if self.app.dirty() {
            save::write_save(&self.app);
            self.app.clear_dirty();
        }
        if self.app.quit() {
            save::write_save(&self.app);
            event_loop.exit();
            return;
        }
        if let Some(w) = self.window.as_ref() {
            w.request_redraw();
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(StdInstant::now() + TICK));
    }
}

fn main() {
    let mut engine = Engine::new(content::load());
    if let Some(state) = save::load_save() {
        engine.restore(state); // restore the notebook before the intro runs
    }
    let app = App::new(engine);

    let event_loop = EventLoop::new().expect("event loop");
    let mut handler = AppHandler::new(app);
    event_loop.run_app(&mut handler).expect("run app");
}
```

- [ ] **Step 2: Build it**

Run: `cargo build -p smobb-desktop`
Expected: compiles cleanly (warnings OK). If the borrow checker complains at `RedrawRequested`, confirm the `let Self { terminal, app, .. } = self;` destructure is present.

- [ ] **Step 3: Run it and verify the window (manual)**

Run: `cargo run -p smobb-desktop`
Expected: a native window titled "S.M.O.B.B." opens (no terminal UI takeover). The intro text reveals with typewriter pacing. Verify by interacting:
- Typing + Enter advances the story.
- `Tab` switches the sidebar tab and focuses it (its border brightens).
- `→` focuses the sidebar, `←` focuses the story; `↑/↓`, `PageUp/PageDown` scroll the focused pane. With a long journal/options list, the sidebar now scrolls past the bottom that previously clipped.
- `Esc` closes the window; reopening restores progress (save persisted under the per-user data dir).

- [ ] **Step 4: Run the whole workspace test suite**

Run: `cargo test`
Expected: PASS across all crates (core, tui-core, tui, desktop). Confirms no regression from the `Key` enum / `ui.rs` changes.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-desktop/src/main.rs
git commit -m "feat(desktop): native winit + wgpu window for the TUI

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

### Task 7 (optional): macOS `.app` bundle

**Files:**
- Already configured: `[package.metadata.bundle]` in `crates/smobb-desktop/Cargo.toml` (Task 4).
- Modify (optional): add an `icon = [...]` entry once an icon file exists.

This task produces a double-clickable macOS `.app`. It is packaging only — the app already runs as its own window from Task 6.

- [ ] **Step 1: Install the bundler**

Run: `cargo install cargo-bundle`

- [ ] **Step 2: (Optional) generate an icon**

The web build ships `crates/smobb-web/icon.svg`. Convert it to PNGs / `.icns` (e.g. with `rsvg-convert` + `iconutil` on macOS) and reference them under `[package.metadata.bundle]`:

```toml
[package.metadata.bundle]
name = "S.M.O.B.B."
identifier = "com.smobb.desktop"
icon = ["assets/icon.icns"]
```

If no icon is provided, the bundle uses a default icon — acceptable for a first build.

- [ ] **Step 3: Build the bundle**

Run: `cargo bundle --release -p smobb-desktop`
Expected: produces `target/release/bundle/osx/S.M.O.B.B..app`. Double-click it: the window opens with no terminal.

- [ ] **Step 4: Commit any bundle config / icon assets**

```bash
git add crates/smobb-desktop/Cargo.toml crates/smobb-desktop/assets
git commit -m "chore(desktop): macOS .app bundle config

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec coverage:**
- Issue 1 native window → Tasks 4–6. Cross-platform (winit/wgpu are cross-platform; `windows_subsystem` guard) → Task 6. Save in per-user dir → Task 4. Font + fallback → Task 6. Bundling → Task 7. ✓
- Issue 2 scrollable sidebar, focus-follows-sidebar → Tasks 1–2; terminal Left/Right → Task 3. Default top vs story bottom, clamp, focus highlight, hint → Task 2. ✓
- Global constraint "ratatui stays 0.29" honored by pinning `ratatui-wgpu = "0.4"`. ✓
- Testing per spec: app logic tests (Task 1), render TestBackend test (Task 2), input unit tests (Task 5), manual window run (Task 6). ✓

**Placeholder scan:** No TBD/TODO; every code step shows full code. Task 7 is explicitly optional (packaging), not a placeholder.

**Type consistency:** `Focus`, `scroll_focus()`, `sidebar_scroll()`, `sidebar_user_scrolled()`, `Key::Left`/`Key::Right` defined in Task 1 and consumed identically in Tasks 2, 3, 5, 6. `WgpuBackend<'static,'static>`, `Dimensions{width,height}`, `Font::new`, `build_with_target`, `backend_mut().resize` match the verified ratatui-wgpu 0.4.1 API. `to_key(&WKey, ModifiersState)` signature consistent between Task 5 definition and Task 6 call site.
