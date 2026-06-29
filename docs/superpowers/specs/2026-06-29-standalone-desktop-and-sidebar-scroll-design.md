# Standalone desktop app + scrollable sidebar — design

Date: 2026-06-29
Status: approved

## Problem

Two issues with the S.M.O.B.B. TUI:

1. **It runs inside the terminal.** The user wants it to launch as its own
   application window, not take over a terminal session.
2. **The journal/notes sidebar is not scrollable.** When journal/options content
   grows taller than the sidebar viewport, the overflow is unreachable — content
   silently clips at the bottom.

## Context

The codebase already separates rendering from the host frontend:

- `smobb-tui-core` — backend-agnostic ratatui draw logic (`ui::render`) and the
  `App` state machine. Depends on `ratatui = "0.29"` (`default-features = false`,
  feature `unstable-rendered-line-info` for `Paragraph::line_count`).
- `smobb-tui` — terminal binary (crossterm backend) over `smobb-tui-core`.
- `smobb-web` — browser binary (`ratzilla = "0.2"`, also ratatui 0.29) over the
  same core.

Because `ui::render(f: &mut ratatui::Frame, app: &App)` is host-independent, a
third frontend can reuse it unchanged, provided it uses the same ratatui 0.29.

## Issue 1 — standalone native window

### Approach (decided)

A pure-Rust native window that renders the **same** ratatui UI into its own GPU
window. No terminal, no webview, no Node. Cross-platform (macOS / Windows /
Linux). Chosen over wrapping the web build (would add a Tauri/webview toolchain)
and over a `.app` that launches a terminal (the user explicitly does not want a
terminal).

### Backend choice

`ratatui-wgpu = "0.4"` (0.4.1). Verified facts that make this additive:

- Pins `ratatui ^0.29` — **same major as the repo; no 0.29→0.30 migration.**
  (The newer 0.5.0 targets ratatui 0.30's split crates `ratatui-core` /
  `ratatui-widgets` and would force bumping `smobb-tui-core`, `smobb-tui`, and
  `ratzilla` 0.2 → 0.3.1. Rejected.)
- `winit ^0.30` is a **dev**-dependency of the crate, so the application owns the
  winit event loop and feeds events — exactly the control we want for input
  mapping.
- `wgpu ^26`. MSRV 1.75 (repo is 1.74; local toolchain is 1.96 — non-issue).
- Re-exports `ratatui` and `wgpu`, so the ratatui version unifies with
  `smobb-tui-core` on 0.29 and the `Frame` types are identical. The
  `unstable-rendered-line-info` feature is additive and stays enabled.

### New crate: `smobb-desktop`

Added to the workspace `members`. Sibling of `smobb-tui` / `smobb-web`. The
terminal and web builds are not modified.

Dependencies: `ratatui-wgpu = "0.4"`, `winit = "0.30"`, `smobb-core`,
`smobb-tui-core`, `dirs` (save location), `web-time`, plus a vendored monospace
font asset.

### Components

- **`main.rs`** — builds the `Engine` (loading any existing save), constructs
  `App`, runs the winit event loop via an `ApplicationHandler`. Carries
  `#![windows_subsystem = "windows"]` so the Windows build spawns no console
  window.
- **`AppHandler` (`ApplicationHandler`)** — owns the window and the
  `Terminal<WgpuBackend>`:
  - `resumed`: create the window; build the backend
    `Builder::from_font(Font::new(FONT_BYTES))
       .with_fonts([... fallback ...])
       .with_width_and_height(Dimensions { width, height })
       .build_with_target(window.clone())` (block_on the async build); wrap in
    `ratatui::Terminal::new(backend)`.
  - `window_event`:
    - `Resized(size)` → `terminal.backend_mut().resize(size.width, size.height)`.
    - `KeyboardInput { event, .. }` → `to_key(event, modifiers)` → `app.key(..)`,
      reusing the press/repeat/Shift/Ctrl-C semantics already in `smobb-tui`.
    - `CloseRequested` → persist save, `event_loop.exit()`.
    - `ModifiersChanged` → track `ModifiersState` (for Ctrl+C and Shift).
  - `about_to_wait`: the animation/pacing pump — `app.tick(Instant::now())`;
    persist if `app.dirty()`; if `app.quit()` then `event_loop.exit()`;
    `set_control_flow(ControlFlow::WaitUntil(now + 30ms))`;
    `window.request_redraw()`.
  - redraw (handled in `window_event`'s `RedrawRequested`) →
    `terminal.draw(|f| ui::render(f, &app))`.
- **`input.rs`** — `fn to_key(event: &winit::event::KeyEvent, mods:
  ModifiersState) -> Option<Key>`. Pure function, unit-tested without a GPU.
  Mapping:
  - `NamedKey::Enter→Enter`, `Backspace→Backspace`, `Tab→Tab`, `Escape→Esc`,
    `ArrowUp→Up`, `ArrowDown→Down`, `ArrowLeft→Left`, `ArrowRight→Right`,
    `PageUp→PageUp`, `PageDown→PageDown`, `F1/F2/F3→F1/F2/F3`,
    `Shift→Shift`, `Space→Char(' ')`.
  - `Key::Character(s)` → first char → `Char(c)`; with Ctrl held and `c=='c'`
    → `CtrlC`.
  - Honor `event.repeat`: fast keys (Shift / Space) act on press+repeat; other
    keys act on initial press only — mirrors the terminal's filtering.
- **`save.rs`** — `fn save_path() -> PathBuf` via `dirs::data_dir()` →
  `<data-dir>/smobb/smobb-save.json`, creating the directory. `load_save` /
  `write_save` mirror the terminal build but use this path (a bundled `.app` has
  no meaningful cwd). The terminal build's cwd-relative save is unchanged
  (out of scope).
- **Font asset** — one vendored permissively-licensed monospace TTF in
  `crates/smobb-desktop/assets/`, covering box-drawing glyphs plus the content's
  dingbats (`↷` U+21B7, `✎` U+270E, `⚙` U+2699, `░▒▓` U+2591–2593). A fallback
  font is registered via the builder for any symbol the primary font lacks. The
  backend renders no hardware cursor; the input line draws its own caret
  character.

### Bundling

- All three OSes produce a native binary directly (`cargo run -p smobb-desktop`,
  `cargo build --release -p smobb-desktop`).
- macOS `.app`: `cargo-bundle` with `[package.metadata.bundle]` (identifier,
  name, icon). Icon derived from `crates/smobb-web/icon.svg` (rasterized to the
  required `.icns`/PNG sizes).
- Windows installer / Linux AppImage are follow-up packaging, not part of this
  change.

## Issue 2 — scrollable sidebar (focus-follows-sidebar)

### Root cause

In `ui::render`, the story Paragraph is scrolled with a `line_count`-based clamp
(`scroll((y,0))`), but the sidebar Paragraph is rendered with no `.scroll()` and
the `App` has no sidebar scroll offset. Journal/options content taller than the
viewport clips unreachably.

### Model

A spatial scroll-focus concept that needs no new modifier keys (modified keys are
unreliable across terminals, which the codebase already notes):

- New `Key::Left` / `Key::Right` variants. Currently unmapped, so free. Mapped in
  both `smobb-tui/src/main.rs` (crossterm `Left`/`Right`) and `smobb-desktop`
  (winit `ArrowLeft`/`ArrowRight`).
- `App` gains:
  - `scroll_focus: Focus` where `Focus { Story, Sidebar }`, default `Story`.
  - `sidebar_scroll: u16` and `sidebar_user_scrolled: bool` — the same pattern
    the story already uses.
- Key handling:
  - `Left` → `scroll_focus = Story`.
  - `Right` → if `sidebar_visible`, `scroll_focus = Sidebar`.
  - `Tab` (already toggles Options/Journal) → also `scroll_focus = Sidebar` and
    reset `sidebar_scroll = 0`, `sidebar_user_scrolled = false` (so the two tabs
    do not share a stale offset).
  - `Up` / `Down` / `PageUp` / `PageDown` → apply to the focused pane's scroll.
    Story keeps its existing behavior (default bottom; only upward moves set
    `user_scrolled`). The sidebar defaults to **top**, so **any** manual move —
    up or down — sets `sidebar_user_scrolled = true`; otherwise a downward scroll
    would immediately snap back to the top default.
  - `F1` hiding the sidebar → `scroll_focus = Story`.
- `ui::render`:
  - Sidebar Paragraph gets `.scroll((y, 0))` computed with the **same**
    `line_count(inner_w)` → `max_scroll = total.saturating_sub(inner_h)` clamp as
    the story. This clamp alone fixes the unreachable-overflow bug.
  - Default offset: the sidebar defaults to **top** (read top-down) when the user
    has not scrolled, whereas the story auto-follows the **bottom** (latest line).
    When `sidebar_user_scrolled`, use `sidebar_scroll.min(max_scroll)`.
  - The focused pane's border is highlighted (distinct border style) so it is
    clear which pane the arrow keys drive.
  - The same sidebar-scroll fix is applied in the banner branch of `render`
    (which also draws the sidebar).
- The input hint line gains a `←/→ focus` cue.

## Testing

- **Issue 2 (logic + render):**
  - `app.rs` unit tests: `Right`/`Tab` set sidebar focus only when visible;
    `Left`/`F1` return focus to story; Up/Down route to the focused pane's
    offset; switching tab resets the sidebar offset.
  - `ui.rs` `TestBackend` tests (modeled on `auto_scrolls_to_latest_line`): a
    journal taller than the viewport reveals its bottom entries after scrolling
    down; the offset clamps and never exceeds `max_scroll`.
- **Issue 1:**
  - `input.rs` unit tests: `to_key` maps representative winit events (named keys,
    a character, Ctrl+C, Shift, repeat filtering) to the correct `Key`, with no
    GPU/window.
  - The window/render path is verified by building and running
    `smobb-desktop` (the draw logic itself is already covered by
    `smobb-tui-core` tests).

## Risks and mitigations

- **wgpu 26 MSRV** could exceed 1.75. Local toolchain is 1.96, so building is
  fine; the existing crates keep their own MSRV. If a pinned wgpu/winit point
  release misbehaves, pin to a known-good combo.
- **Font glyph coverage** for the dingbats — mitigated by a registered fallback
  font.
- **No hardware cursor** in `ratatui-wgpu` — the app already draws its own
  `> input` line; add a caret character so the input position is visible.

## Out of scope

- Sharing one save file between the terminal and desktop builds.
- Windows installer / Linux AppImage packaging.
- Any change to `smobb-tui` (terminal) or `smobb-web` behavior beyond adding the
  `Key::Left`/`Key::Right` mapping in the terminal binary.
