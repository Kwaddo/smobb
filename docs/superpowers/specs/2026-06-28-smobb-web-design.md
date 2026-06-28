# S.M.O.B.B. Web/Mobile (Phase 4) — Design

**Date:** 2026-06-28
**Status:** Approved (design)
**Builds on:** Phase 3 (`smobb-tui` ratatui app) + the engine/QoL. Reuses the TUI in the browser.

## Goal

Ship S.M.O.B.B. to the browser and as an installable mobile app (PWA), with the same terminal UI as the desktop TUI — completing "all platforms with the look of a terminal."

## Decisions locked
- **Rendering:** reuse the ratatui `App` + `ui::render` in the browser via **ratzilla 0.2** (ratatui's web/DOM backend, re-exports ratatui 0.29). One UI everywhere.
- **Shared core crate:** extract the reusable logic into a new **`smobb-tui-core`** lib (App + ui) that does NOT depend on crossterm, so it compiles for both native and `wasm32`. The native `smobb-tui` and the new `smobb-web` both depend on it. (Discovered via spike: crossterm does not build for `wasm32`, and ratatui's default features pull it in — so the wasm path must be crossterm-free.)
- **Neutral key enum:** the App takes a crate-defined `Key` enum, not crossterm's (or ratzilla's) `KeyCode`. Native translates crossterm → `Key`; web translates `ratzilla::event::KeyEvent` → `Key`. Decouples the App from any backend's input types.
- **Time:** the core uses the **`web-time`** crate's `Instant` (drop-in; `std` on native, `performance.now()` on wasm). Required because `std::time::Instant` panics on `wasm32`.
- **Mobile input:** an on-screen **control bar** of touch buttons that dispatch synthetic keyboard events (ratzilla already handles keys); tapping focuses a hidden `<input>` to raise the soft keyboard for typing. Desktop keyboard unchanged.
- **PWA:** web app manifest + service worker (installable + offline).
- **Build:** **trunk** → static `dist/`. Hosting/installers are Phase 5.

## Architecture

### `crates/smobb-tui-core` — shared portable lib (new)
Move `app.rs` + `ui.rs` here (out of `smobb-tui`). Changes:
- Define `pub enum Key { Char(char), Enter, Backspace, Tab, F1, F2, Up, Down, PageUp, PageDown, Esc, CtrlC }`. `App::key` takes `Key` (was crossterm `KeyCode`/`KeyModifiers`).
- `app.rs` uses only `smobb-core` + `web_time::Instant` + `Key` (no ratatui, no crossterm).
- `ui.rs` uses `ratatui` with **`default-features = false`** + `features = ["unstable-rendered-line-info"]` (no crossterm backend) — only layout/widgets/text/Frame, which compile on wasm. `to_rgb`/render unchanged.
- Deps: `smobb-core` (path), `ratatui` (0.29, default-features=false, unstable-rendered-line-info), `web-time` (1).
- The existing App/ui unit tests + TestBackend test move here and must stay green (`cargo test -p smobb-tui-core`). Tests that built `KeyCode` now build `Key`.

### `crates/smobb-tui` — native binary (slimmed)
- Depends on `smobb-tui-core` + `crossterm` (+ ratatui crossterm backend, for `ratatui::init`/`DefaultTerminal`).
- `main.rs`: the crossterm event loop, translating `crossterm::event::KeyEvent` → `smobb_tui_core::Key`, then `app.key(key)`; `ui::render` + `app.tick(web_time::Instant::now())`. Behavior identical to today's TUI.

### `crates/smobb-web` — wasm frontend (new)
- Deps: `smobb-tui-core` (path), `ratzilla = "0.2"`, `web-time = "1"`. (ratzilla re-exports the SAME ratatui 0.29 → `ui::render`'s `Frame` type unifies; no crossterm pulled — verified by spike.)
- Pattern (ratzilla 0.2 API, confirmed):
  ```rust
  let app = Rc::new(RefCell::new(App::new(Engine::new(content::load()))));
  let backend = DomBackend::new()?;
  let terminal = Terminal::new(backend)?;
  terminal.on_key_event({ let app = app.clone(); move |ke| {
      if let Some(k) = to_key(&ke) { app.borrow_mut().key(k); }
  }});
  terminal.draw_web(move |f| {
      app.borrow_mut().tick(web_time::Instant::now());
      ui::render(f, &app.borrow());
  });
  ```
  `to_key` maps `ratzilla::event::KeyEvent { code, ctrl, .. }` → `Key` (Char(' ')→Char(' '); Char('c')+ctrl→CtrlC; F(1)/F(2)→F1/F2; Enter/Backspace/Tab/Up/Down/PageUp/PageDown/Esc → matching).

### `web/` — page shell, controls, PWA
- `index.html` (trunk entry): mounts the ratzilla render target; includes the terminal-framed CSS, the control bar, and registers the service worker + manifest.
- **Control bar** (static HTML + a small vanilla JS, no wasm bridge): buttons `≡ Sidebar` (F1), `Tab`, `Journal` (sets sidebar→Journal then shows; implemented as Tab logic), `Hold: Skip` (Space, repeated while pressed via pointerdown/up), `↑`/`↓` (scroll), `Enter`. Each button dispatches a synthetic `KeyboardEvent` to the target ratzilla listens on. Tapping the terminal focuses a hidden `<input autocapitalize=off autocomplete=off>`; its `input`/`beforeinput` events forward typed characters as synthetic `keydown`s and Enter submits. This reuses ratzilla's existing key handling — no extra wasm↔JS calls.
- Responsive: a CSS breakpoint for narrow widths; on phones the sidebar starts hidden (the App already supports F1 hide — the web entry sends one F1 at startup when the viewport is narrow) and the ≡ button peeks it.
- **PWA:** `manifest.json` (`display: standalone`, `name`, `theme_color`, `icons`), an app icon (simple generated mark), and `sw.js` caching the html/js/wasm/css/icon for offline. Content is embedded in the wasm, so once cached the game runs fully offline.

## Data flow
`content::load()` (embedded) → `Engine` → `App` (reveal queue/timer via `web_time::Instant`) → `ui::render` → ratzilla DOM. Input: physical keyboard → ratzilla; touch buttons/soft-keyboard → synthetic `KeyboardEvent` → ratzilla → `app.key`. Identical to the desktop pipeline except the backend + the synthetic-event source.

## Testing
- **Reused logic:** `cargo test -p smobb-tui` (App/ui) still runs natively and must stay green after the `web-time` migration.
- **wasm compile gate:** `smobb-web` must build for `wasm32-unknown-unknown` (via `trunk build` or `cargo build --target wasm32-unknown-unknown`).
- **Browser smoke (Playwright):** build, serve `dist/`, load in a headless browser, assert the title + a story line render, and that a control-bar button changes state (e.g. Tab switches the sidebar header to JOURNAL). This is the integration check; deep interaction isn't unit-testable.

## Risks & mitigations
- **ratzilla API/maturity:** third-party, evolving API. Mitigation: a small spike compiles a trivial ratzilla app first to pin the exact API + version and warm the offline cache; the plan's ratzilla code is written against that pinned version and kept adaptable. If ratzilla proves unworkable, STOP and escalate (fallback = custom-WASM UI driving the engine `Output` stream directly).
- **wasm `Instant`:** addressed by `web-time` (the only reason the migration exists).
- **trunk/wasm toolchain:** needs the `wasm32-unknown-unknown` target + trunk; the spike installs/warms these (network) before the offline subagent build loop.

## Out of scope (YAGNI)
- Hosting/deploy + installers (Phase 5). No server, no analytics, no accounts.
- No separate mobile-native build (the PWA is the mobile target).
- No new game content or engine changes (pure frontend reuse).
