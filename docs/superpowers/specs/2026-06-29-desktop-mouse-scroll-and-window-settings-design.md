# Desktop: mouse-wheel scroll + window settings overlay

Date: 2026-06-29
Branch: `feat/standalone-desktop-and-scroll`
Crate: `smobb-desktop` (with one small `smobb-tui-core` addition)

## Problem

The native desktop window (`winit` + `wgpu` via `ratatui-wgpu`) currently only
accepts keyboard input. Two gaps:

1. **No mouse-wheel scrolling.** Readers must use arrow / PageUp / PageDown keys
   to move the story and sidebar. We want comfortable wheel scrolling with the
   mouse, scrolling whichever pane the cursor is over.
2. **No way to manage the window.** Font size is hardcoded at 16px and there is
   no fullscreen control. We want an in-app settings overlay to adjust font size
   and toggle fullscreen, plus keyboard shortcuts for power users.

These are window-management concerns specific to the desktop frontend — the web
and terminal frontends cannot change a pixel font size or go fullscreen — so the
implementation stays in `smobb-desktop`, with only a tiny frontend-agnostic
scroll helper added to shared `smobb-tui-core`.

## Background: current layout & scroll model

- `smobb-tui-core` owns the app state and `ui::render`. The layout is: an outer
  bordered block, a body row + a 1-line input row, and the body splits into a
  story column (70%) and an optional sidebar column (30%) when
  `app.sidebar_visible()`.
- Scroll state lives in `App`: `scroll` / `user_scrolled` (story) and
  `sidebar_scroll` / `sidebar_user_scrolled` (sidebar), plus `scroll_focus:
  Focus` (`Story` | `Sidebar`). The max-scroll clamp is computed at render time
  from `Paragraph::line_count`, not stored — so callers only nudge a counter and
  render clamps it.
- `ratatui-wgpu` 0.4.1 supports runtime font resize: build a `Fonts` set
  (`Fonts::set_size_px`) and call `backend.update_fonts(fonts)`. The DejaVu mono
  faces are `'static` byte slices, so a freshly built `Fonts<'static>` has no
  lifetime trouble.

## Design

### Scope split

- `smobb-tui-core`: one new method + two named constants. Frontend-agnostic.
- `smobb-desktop`: all window-management logic (wheel mapping, overlay, font
  rebuild, fullscreen, persistence).

### 1. `smobb-tui-core` changes (minimal)

**`app.rs` — new method:**

```rust
/// Scroll a specific pane by `lines` (negative = toward the top). Focuses the
/// pane so the border highlight follows the wheel. Render clamps the result.
pub fn scroll_pane(&mut self, pane: Focus, lines: i32) {
    self.scroll_focus = pane;
    match pane {
        Focus::Story => {
            self.user_scrolled = true;
            self.scroll = apply_delta(self.scroll, lines);
        }
        Focus::Sidebar => {
            self.sidebar_user_scrolled = true;
            self.sidebar_scroll = apply_delta(self.sidebar_scroll, lines);
        }
    }
}
```

`apply_delta(cur: u16, d: i32) -> u16` is a small saturating helper: clamps to
`>= 0` on the low end (`u16::MAX` ceiling is fine; render clamps to content).

**`ui.rs` — promote split to consts** so the desktop hit-test references them
instead of duplicating `70`/`30`:

```rust
pub const STORY_PCT: u16 = 70;
pub const SIDEBAR_PCT: u16 = 30;
```

Use these in the existing `Layout::horizontal([Percentage(STORY_PCT),
Percentage(SIDEBAR_PCT)])`.

### 2. Mouse-wheel scroll (`smobb-desktop`)

**`input.rs` — pane hit-test (pure):**

```rust
/// Which pane sits under cursor x (physical px) given window width.
/// Sidebar occupies the right `SIDEBAR_PCT`%. Always Story when hidden.
pub fn pane_at(window_w: f64, cursor_x: f64, sidebar_visible: bool) -> Focus {
    if !sidebar_visible { return Focus::Story; }
    let boundary = window_w * (STORY_PCT as f64) / 100.0;
    if cursor_x >= boundary { Focus::Sidebar } else { Focus::Story }
}
```

The ~1-cell error from ignoring borders is irrelevant for choosing a pane.

**`main.rs` — handler state + events:**

- Add fields: `cursor_x: f64` (last cursor physical x, default 0.0),
  `wheel_accum: f32` (trackpad pixel accumulator), `font_px: u32`.
- `WindowEvent::CursorMoved { position, .. }` → store `position.x`.
- `WindowEvent::MouseWheel { delta, .. }`:
  - Determine `pane = input::pane_at(window_w, cursor_x, app.sidebar_visible())`
    where `window_w = window.inner_size().width as f64`.
  - Compute line delta:
    - `MouseScrollDelta::LineDelta(_, y)` → `lines = -(y.round() as i32) * 3`
      (3 lines per notch; negate so wheel-up moves toward earlier text).
    - `MouseScrollDelta::PixelDelta(p)` → accumulate `wheel_accum += p.y as f32`;
      `lines = -(wheel_accum / font_px as f32).trunc() as i32`; subtract the
      consumed whole lines from `wheel_accum` to keep the remainder (smooth
      trackpad).
  - If `lines != 0`: `app.scroll_pane(pane, lines)` and `request_redraw()`.

### 3. Settings overlay (`smobb-desktop`, new `settings.rs`)

State:

```rust
pub struct Settings {
    pub open: bool,
    pub sel: Row,          // FontSize | Fullscreen
    pub font_px: u32,
    pub fullscreen: bool,
}
pub enum Row { FontSize, Fullscreen }

pub enum Action { None, FontChanged(u32), ToggleFullscreen, Close }
```

- `const FONT_MIN: u32 = 8; const FONT_MAX: u32 = 40; const FONT_STEP: u32 = 2;`
- `handle_key(&mut self, key: Key) -> Action` — only called while `open`:
  - `Up`/`Down` → move `sel`.
  - `Left` or `Char('-')` → if `sel == FontSize`, `font_px =
    (font_px - FONT_STEP).max(FONT_MIN)`, return `FontChanged(font_px)`.
  - `Right` / `Char('+')` / `Char('=')` → if `sel == FontSize`, `font_px =
    (font_px + FONT_STEP).min(FONT_MAX)`, return `FontChanged`.
  - `Enter` / `Char(' ')` → if `sel == Fullscreen`, flip `fullscreen`, return
    `ToggleFullscreen`.
  - `Esc` → set `open = false`, return `Close`.
  - else `Action::None`.
- `render(f: &mut Frame, s: &Settings)` — center a small `Rect` (~30×8),
  `Clear` it, draw a bordered block titled ` Settings `, two rows:
  - `Font size   {px}  [-] [+]` (highlighted when `sel == FontSize`)
  - `Fullscreen  [x]` / `[ ]` (highlighted when `sel == Fullscreen`)
  - footer line: `↑↓ select   ←→ adjust   Esc close`.
  Uses `ratatui_wgpu::ratatui` widgets — no `tui-core` change.

### 4. Key routing & apply (`main.rs`)

In `WindowEvent::KeyboardInput` (pressed only), **before** forwarding to
`app.key`:

1. If `settings.open`: map the winit key via `input::to_key`, call
   `settings.handle_key(key)`, act on the returned `Action`, **consume** the
   event (do not forward to `app`). This is why `Esc` closes the panel instead
   of quitting the app.
2. Else, handle global shortcuts (these also consume the event):
   - `F4` → `settings.open = true` (sync `settings.font_px`/`fullscreen` from
     current state first).
   - `Cmd/Ctrl` + `=`/`+` → font up; `Cmd/Ctrl` + `-` → font down; `Cmd/Ctrl` +
     `0` → reset to default 16. (Detected from `self.mods` + the character.)
   - `F11` → toggle fullscreen.
3. Else forward to `app.key` as today.

**Apply helpers:**

```rust
fn build_fonts(px: u32) -> Fonts<'static>   // factored out of resumed()
fn apply_font(&mut self, px: u32)           // update_fonts + resize to window + redraw + persist
fn apply_fullscreen(&mut self, on: bool)    // window.set_fullscreen(...) + persist
```

`build_fonts` builds the regular + bold + italic + bold-italic DejaVu mono set
(same faces as today) and `set_size_px(px)`. `resumed()` calls it instead of the
current inline builder chain.

### 5. Persistence (`smobb-desktop`, new `prefs.rs`)

```rust
#[derive(Serialize, Deserialize)]
pub struct WindowPrefs { pub font_px: u32, pub fullscreen: bool }
```

- Requires adding `serde = { version = "1", features = ["derive"] }` and
  `serde_json = "1"` to `crates/smobb-desktop/Cargo.toml` (the crate does not
  currently depend on serde directly). Keeps the prefs file JSON, consistent with
  the existing save.
- File `smobb-window.json` in the same data dir as the save (reuse the dir logic;
  factor `data_dir()` if convenient, else duplicate the small helper).
- `load() -> WindowPrefs` (defaults `font_px = 16`, `fullscreen = false` on
  missing/parse error). `save(&WindowPrefs)`.
- `resumed()` loads prefs → initial `font_px` for `build_fonts`, applies
  fullscreen if set. Font/fullscreen changes write prefs immediately; also write
  on `CloseRequested` / quit alongside the existing `save::write_save`.

### Data flow

```
winit MouseWheel ─► pane_at(cursor_x) ─► App::scroll_pane ─► (render clamps)
winit Key (F4)   ─► Settings.open = true
winit Key (open) ─► Settings.handle_key ─► Action ─► apply_font / apply_fullscreen ─► prefs.save
RedrawRequested  ─► ui::render(f, app); if open { settings::render(f, &settings) }
```

## Error handling

- Font clamp `[8, 40]` prevents a 0px / unusably huge grid.
- `prefs::load` is total: any IO/parse failure → defaults (never panics, never
  blocks startup).
- `scroll_pane` is saturating; render's existing clamp keeps the offset within
  content regardless of wheel spam.
- `update_fonts` + `resize` always followed by `request_redraw` so a font change
  is never left half-applied on screen.

## Testing

`smobb-tui-core`:
- `scroll_pane` sets `scroll_focus`, flips the right `user_scrolled` flag, moves
  the right counter, and saturates at 0 on upward over-scroll.

`smobb-desktop`:
- `input::pane_at`: hidden sidebar → always `Story`; cursor in left 70% → `Story`;
  cursor in right 30% → `Sidebar`; boundary case.
- `settings::handle_key`: `Up`/`Down` move selection; `+`/`-` change font and
  clamp at `FONT_MIN`/`FONT_MAX`; `Enter` on Fullscreen returns `ToggleFullscreen`
  and flips state; `Esc` closes.
- `prefs`: `WindowPrefs` JSON round-trip; file name/path ends with
  `smobb/smobb-window.json`.

`wgpu`/`winit` glue (`build_fonts`, `apply_font`, fullscreen, wheel-delta math)
is verified manually by running the app — not unit-tested, consistent with the
existing desktop crate.

## Files touched

| File | Change |
| --- | --- |
| `crates/smobb-tui-core/src/app.rs` | `scroll_pane` + `apply_delta` helper + test |
| `crates/smobb-tui-core/src/ui.rs` | `STORY_PCT`/`SIDEBAR_PCT` consts |
| `crates/smobb-desktop/src/input.rs` | `pane_at` + test |
| `crates/smobb-desktop/src/settings.rs` | **new** — overlay state, `handle_key`, `render` + tests |
| `crates/smobb-desktop/src/prefs.rs` | **new** — `WindowPrefs` load/save + test |
| `crates/smobb-desktop/src/main.rs` | wheel + cursor events, key routing, `build_fonts`/`apply_font`/`apply_fullscreen`, prefs wiring |
| `crates/smobb-desktop/Cargo.toml` | add `serde` (derive) + `serde_json` for `prefs` |

## Out of scope (YAGNI)

- Configurable scroll speed / natural-scroll toggle.
- Native macOS menu bar.
- Per-pane independent font sizes; theme/color settings.
- Window opacity, position/size persistence beyond fullscreen.
```

