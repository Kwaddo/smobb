# S.M.O.B.B. TUI (Phase 3) — Design

**Date:** 2026-06-27
**Status:** Approved (design)
**Builds on:** Phases 1–2 (engine + content) and Phase 2.5 (QoL layer). Consumes `smobb-core` as-is.

## Goal

A cross-platform native terminal app (`smobb-tui`) that makes S.M.O.B.B. convenient and ideal to play: a story pane with inline hex translations, a persistent tabbed sidebar (OPTIONS / JOURNAL), and the original's dramatic timed text — kept responsive.

## Decisions locked
- **Layout:** story pane (left) + right sidebar that toggles OPTIONS↔JOURNAL with Tab; sidebar hideable for full-width story.
- **Pacing:** timed line-by-line reveal (the `interval`), but responsive — scroll anytime, Space fast-forwards, `skipmode` (F2) makes text instant.
- **Style:** bordered panes with a `S.M.O.B.B.` title bar and labeled borders.
- **Scope:** desktop TUI only (Win/macOS/Linux). No `smobb-core` changes expected (add a minimal accessor only if strictly needed). Web/WASM = Phase 4.

## Crate & dependencies
- New binary crate `crates/smobb-tui` (workspace member).
- Deps: `smobb-core` (path), `ratatui` (latest 0.x), `crossterm` (ratatui's backend). These two are new to the workspace; the offline cargo cache is warmed before building.
- Run: `cargo run -p smobb-tui`. Release: `cargo build --release -p smobb-tui` → single binary.

## Architecture (3 focused files)

### `src/app.rs` — state + logic (terminal-free, unit-testable)
```rust
pub enum Tab { Options, Journal }
pub struct App {
    engine: Engine,
    transcript: Vec<Span>,      // revealed styled lines (text + Color)
    queue: VecDeque<Output>,    // outputs not yet revealed
    reveal_at: Option<Instant>, // when the next queued item may reveal
    tab: Tab,
    sidebar_visible: bool,
    scroll: u16,
    input: String,
    awaiting: bool,             // true = accept command input; false = revealing
    quit: bool,
}
```
Key methods (all terminal-free):
- `App::new(engine) -> App` — runs `engine.start()`, loads outputs into the queue, begins revealing.
- `submit(&mut self)` — on Enter while `awaiting`: `engine.advance(&input)`, clear input, enqueue outputs, begin revealing.
- `tick(&mut self, now: Instant)` — if revealing and `reveal_at` elapsed, pop+apply the next queued `Output` (see reveal model), set the next `reveal_at`.
- `skip_reveal(&mut self)` — drain the queue instantly (ignore `Sleep`) up to the next `AwaitInput`/`End`.
- `key(&mut self, key)` — dispatch (see keybindings).
- A `Span { text: String, color: Color, dim: bool }` (or reuse ratatui spans) carries style into the transcript.

### Reveal model (in `tick`)
Apply one queued `Output` per ready tick:
- `Line{text,color}` → push a `Span` to transcript; set `reveal_at = now` (next item immediately unless a Sleep follows).
- `Translation(t)` → push a dim, indented span `  ⤷ "{t}"`.
- `Sleep(ms)` → `reveal_at = now + ms` (skipmode already makes the engine emit no Sleeps).
- `Recap(_)` → push a subtle dim `  ✎ noted` line (the sidebar JOURNAL/OPTIONS reflect the real data).
- `AwaitInput` → stop revealing, `awaiting = true`.
- `End` → `quit = true`.
Auto-scroll to bottom as lines append, unless the user has scrolled up.

### `src/ui.rs` — rendering (ratatui)
- Outer block titled `S.M.O.B.B.`.
- Horizontal split: story pane (≈70%) | sidebar (≈30%, hidden if `!sidebar_visible` → story full width).
- Story pane: the `transcript` as styled `Line`s; `Color` → ratatui `Color::Rgb` parsed from `Color::hex()`; dim spans use `Modifier::DIM` + gray.
- Sidebar: a bordered block titled `OPTIONS` or `JOURNAL`; content = the `Output::Line` text from `engine.options_view()` / `engine.journal_view()` (re-read each frame).
- Bottom input line: `> {input}` with a cursor; shows a hint (`[Tab] switch  [F1] sidebar  [F2] skip  [Esc] quit`).

### `src/main.rs` — terminal lifecycle + event loop
- Enter raw mode + alternate screen (crossterm); restore on exit (incl. on panic via a guard).
- Loop: `terminal.draw(ui)`; `event::poll(tick_timeout ≈ 30ms)`; on event → `app.key(...)`; always `app.tick(Instant::now())`; break when `app.quit`.

## Keybindings
- Printable char → append to `input` (only while `awaiting`). **Enter** → `submit`. **Backspace** → edit.
- **Tab** → toggle sidebar tab. **F1** → toggle sidebar visibility. **F2** → toggle skipmode (`engine.set_skip`).
- **Space** (while revealing) → `skip_reveal`. (While `awaiting`, Space types a space.)
- **↑/↓**, **PgUp/PgDn** → scroll transcript.
- **Esc** or **Ctrl-C** → quit.

## Color mapping
`smobb-core::speaker::Color` already exposes `hex()`. `ui.rs` parses the `#rrggbb` into `ratatui::style::Color::Rgb`. Single mapping helper; no core change.

## Testing (`cargo test -p smobb-tui`, no TTY)
- **App logic (unit):** construct `App` over a `content::from_str` fixture; drive `submit`/`tick`/`key`/`skip_reveal` with explicit `Instant`s; assert transcript contents, queue draining, Sleep honoring, tab toggle, sidebar toggle, scroll, quit on `End`. Reveal timing tested by passing controlled `now` values.
- **Render (TestBackend):** render `App` to a ratatui `TestBackend` buffer; assert the title, a story line, and the sidebar header/text appear; assert sidebar hidden when toggled off.

## Error handling
- Terminal restore guaranteed via a Drop guard (raw mode + alt screen) so a panic never leaves the user's terminal broken.
- Engine panics (missing scene/location) are already impossible for shipped content (Phase 1–2 verified all targets resolve); if one occurred, the Drop guard still restores the terminal before the process exits.

## Out of scope (YAGNI)
- No mouse support, no resize-specific layouts beyond ratatui's automatic reflow, no config file, no save/load to disk (recap/journal are in-memory this phase; persistence can be a later addition).
- No web/WASM (Phase 4). No packaging/installers (Phase 5).
