# S.M.O.B.B. — Visual Polish & Dramatic Event Effects

**Date:** 2026-06-28
**Status:** Design — awaiting review
**Branch:** smobb-rust-port

## 1. Goal

Make the game look substantially better, and give major story beats a
distinct, dramatic visual treatment. Two outcomes:

1. **Baseline polish** — the normal reading experience looks intentional
   instead of a flat wall of single-color text.
2. **Dramatic effects** — a small vocabulary of effects (flash, glitch,
   banner, mood) authored into the content and fired at the handful of real
   climaxes.

Decisions locked in brainstorming:

- **Both layers** — structural polish in the grid *and* CRT atmosphere in the
  web shell.
- **Engine-authored effects (approach #1)** — effects are first-class content
  (`Step::Effect`), flow through the engine as `Output::Effect`, and render
  per-platform. Not heuristic text-sniffing.
- **Restrained intensity** — subtle always-on atmosphere; big effects reserved
  for the few genuine climaxes so they land hard.

## 2. Non-goals

- No modern reskin / chat-bubble UI. We keep the terminal identity.
- No new game content, branching, or story changes — only effect annotations
  on existing beats.
- No audio.
- No change to save format semantics beyond one additive settings flag.

## 3. Architecture overview

Four crates, unchanged in shape:

- `smobb-core` — engine + RON content. **Gains** the effect model and pass-through.
- `smobb-tui-core` — shared ratatui render. **Gains** the effect runtime (state +
  timing) and all grid-level rendering (polish + grid effects).
- `smobb-tui` — native binary. No code change; inherits everything.
- `smobb-web` — ratzilla/WASM + CSS shell. **Gains** the CSS atmosphere layer
  and a thin bridge that mirrors effect state onto DOM classes.

Layering principle: **every effect has a cross-platform grid implementation in
`ui.rs`, and the web shell adds CSS enhancement on top.** The native TUI is
never left blank; the browser just looks richer.

```
content (.ron)
   │  Step::Effect(EffectKind)
   ▼
smobb-core Engine ──► Output::Effect(EffectKind)   (pass-through, skip-independent)
   ▼
smobb-tui-core App   (effect runtime: timing, fast-forward capping, clear-on-enqueue)
   ├─► ui.rs          grid render: banner overlay, invert-flash, char-scramble, mood recolor
   └─► (web) main.rs  reads App effect snapshot ──► toggles body CSS classes
                                                      ▼
                                          style.css: glow, scanlines, vignette,
                                                     flash, glitch, shake, mood tint
```

## 4. Effect model (smobb-core)

New module `crates/smobb-core/src/effect.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum EffectKind {
    /// One-shot screen flash (~120ms). Reuses the speaker palette for tint.
    Flash(crate::speaker::Color),
    /// Corruption burst for N ms (char scramble + web RGB-split/jitter).
    Glitch(u16),
    /// Full-screen takeover headline for a fixed duration. Cross-platform.
    Banner(String),
    /// Sustained palette/tint until `Mood(Normal)` or the next command.
    Mood(Mood),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Mood {
    Normal,      // clears any active mood
    RedAlert,    // self-destruct
    ColdCyan,    // SMOBB takes over
    Corruption,  // sickly green; breaches / Jaffar
}
```

`Step` (scene.rs) gains one variant:

```rust
Effect(crate::effect::EffectKind),
```

`Output` (engine.rs) gains one variant:

```rust
Effect(crate::effect::EffectKind),
```

**Engine behavior:** in `run()` (and `emit_intro`/`emit_step_lines` where
relevant), `Step::Effect(k)` emits `Output::Effect(k)` and nothing else — no
`Sleep`, no state mutation. Effects are emitted **regardless of skip mode**
(skip only suppresses `Sleep`; drama still plays). Timing/pacing is entirely the
renderer's job.

**Exhaustive-match fallout (must update):**
- `engine.rs` test `runs_a_command_scene_then_returns_to_menu` — the `kinds`
  match arm needs an `Output::Effect(_) => "X"` case.
- `app.rs` `tick` and `skip_reveal` — handle `Output::Effect`.
- `ui.rs` `output_lines_to_text` — already a `filter_map`; non-`Line` returns
  `None`, so sidebar/journal ignore effects safely.

## 5. Speaker labels — threading speaker identity

The structural polish includes per-turn speaker labels (e.g. `ALI  …`), but
`Output::Line { text, color }` currently discards the speaker (`color` is not a
unique identity — System and Smobb are both White).

**Change:** add an optional speaker to dialogue lines.

```rust
Output::Line { text: String, color: Color, speaker: Option<Speaker> },
```

- Dialogue emit sites (`run`, `emit_intro`, `emit_step_lines`) set
  `speaker: Some(sp)`.
- UI-construction sites (`options_view`, `journal_view`, `recap_view`,
  `menu_prompt` framing, `emit_pause` dots) set `speaker: None`.
- `App::Span` carries an optional display name derived from the speaker.

This is the most test-touching change (many `Output::Line { text, color }`
literals in engine.rs + tests). It is mechanical; tests that use
`Output::Line { text, .. }` are unaffected, only full-field literals need the
extra field.

**Label rendering rule (ui.rs):** show the speaker name only when it *changes*
from the previous transcript line (grouped turns), dimmed and fixed-width
(e.g. 6-col gutter). Consecutive lines from the same speaker are not re-labelled.
`None`-speaker lines (menus, dots, system UI) render with no gutter as today.

## 6. Effect runtime (smobb-tui-core `App`)

New state on `App`:

```rust
flash_until:  Option<Instant>,   // active flash + its tint
flash_tone:   Color,
glitch_until: Option<Instant>,
banner:       Option<String>,    // active takeover text
banner_until: Option<Instant>,
mood:         Mood,              // sustained; default Normal
last_tick:    Instant,          // clock for render-time getters
```

`reduced_effects` is **not** an `App` field — it lives in the engine alongside
`skip` (so it persists via the existing snapshot path) and `App` reads/writes it
through accessors, exactly mirroring the `skip` pattern.

Constants (tuned, restrained):

```
FLASH_MS  = 120
BANNER_MS = 1400
// Glitch carries its own ms from content.
```

**`tick` handling of `Output::Effect(k)`** (inside the existing reveal loop):

- `Flash(tone)`: set `flash_until = now + FLASH_MS`, `flash_tone = tone`; set
  `reveal_at = now + FLASH_MS` so the flash is perceptible before the next line.
- `Glitch(ms)`: set `glitch_until = now + ms`; **non-blocking** (text keeps
  revealing while corrupted) — do not set `reveal_at`.
- `Banner(text)`: set `banner = Some(text)`, `banner_until = now + BANNER_MS`;
  set `reveal_at = now + BANNER_MS` (takeover blocks reveal for its duration).
- `Mood(m)`: set `mood = m`; non-blocking, instantaneous.

**Fast-forward (hold-Space) capping:** the existing `fast` path caps pending
`reveal_at` to `FAST_STEP_MS`. The same cap applies to flash/banner blocks, so a
player holding Space blows through effects. Glitch/mood, being non-blocking, are
unaffected (they simply elapse). This reuses the existing mechanism — no new
fast-forward logic.

**`reduced_effects` (accessibility):** when true —
- `Flash` → no flash, no block (treated as a no-op beat).
- `Glitch` → ignored (no scramble).
- `Banner` → still shown (it is information, not motion) but with no block-skip
  difference; rendered static.
- `Mood` → still applied (it is a static tint, low motion).

**`enqueue()` resets all effect state** (`flash_until`/`glitch_until`/`banner*`
to `None`, `mood = Normal`). This scopes effects to a single command's reveal
stream and guarantees a half-finished dramatic scene can't leak mood into the
hub. (A sustained mood therefore lasts through the await prompt and clears when
the player submits the next command — desirable: red-alert lingers while you
decide, clears when you act.)

**New key + persistence:** `Key::F3` toggles `reduced_effects` and marks
`dirty`. `SaveState` gains `reduced_effects: bool` (default `false`) in
`save.rs`, wired through `Engine::snapshot`/`restore`. Snapshot lives in the
engine for consistency with `skip`; `App` reads/writes it via new
`engine.reduced_effects()` / `engine.set_reduced_effects()` accessors, mirroring
the existing `skip` pattern.

Getters for the renderer: `flash(now) -> Option<Color>`, `glitch(now) -> bool`,
`banner() -> Option<&str>`, `mood() -> Mood`. They compare against the `now`
passed to the most recent `tick` (stored as `last_tick: Instant`) so `render`
needs no clock.

## 7. Grid rendering (smobb-tui-core `ui.rs`) — cross-platform

### 7a. Structural polish (always on)

- **Header:** keep the bordered title but add a thin status strip — current
  location name + mood indicator + `reduced` flag when active.
- **Speaker grouping:** insert a blank spacer line when the speaker changes
  (using the new `speaker` field), and render the dimmed name gutter (§5). This
  is the single biggest readability win — it breaks the wall of same-color text.
- **Separators:** a faint horizontal rule between a returned-from scene and the
  next hub prompt (visual "scene break").
- **Sidebar:** style `OPTIONS`/`JOURNAL` with section spacing and a consistent
  bullet; right-align the tab indicator. Journal section headers (`[Clues]`
  etc.) get a subtle highlight style.
- **Input line:** unchanged behavior; restyle the hint row to be dimmer so the
  prompt stands out.

### 7b. Grid effect rendering

- **Banner:** while `banner()` is `Some`, render large block text centered over
  the story area (figlet-style ASCII from a tiny built-in 5-row font for A–Z,
  0–9, space; fall back to plain centered uppercase if a glyph is missing).
  Overlays the story viewport for its duration, then transcript resumes.
- **Flash:** while `flash()` is `Some(tone)`, fill the story area background with
  the tone color for that frame (invert/wash). Cheap, works in any terminal.
- **Glitch:** while `glitch()` is true, deterministically scramble a fraction of
  glyphs in newly revealed lines (seed from line index — no RNG; `Math.random`
  is unavailable in this environment anyway) and shift color toward the
  Corruption palette.
- **Mood:** recolor the story background / apply a tint bias to line colors per
  the active `Mood` (RedAlert → red bias, ColdCyan → cyan bias, Corruption →
  green bias). Subtle; readability preserved.

## 8. Web shell (smobb-web) — CSS atmosphere + bridge

### 8a. CRT baseline (always on, web-only) — `style.css`

- **Phosphor glow:** `#grid span { text-shadow: 0 0 4px currentColor; }` —
  `currentColor` is each cell's own color, so per-speaker colors keep their hue
  while glowing. No global green wash.
- **Scanlines:** a fixed full-viewport overlay (`repeating-linear-gradient`),
  very low opacity, `pointer-events: none`.
- **Vignette:** a fixed radial-gradient overlay darkening the edges.
- All three respect `@media (prefers-reduced-motion: reduce)` (no animated
  flicker; static scanlines may remain).

### 8b. Effect classes (toggled per frame) — `main.rs` bridge + `style.css`

In the `draw_web` closure, after `tick`, read the `App` effect snapshot and set
classes on `document.body` (via `web_sys`):

- `fx-flash` (+ a tone modifier) — keyframed full-screen flash overlay.
- `fx-glitch` — RGB-split + jitter on `#grid` (CSS `filter`/`transform`
  keyframes) layered over the grid's own scramble.
- `fx-shake` — optional subtle screen shake, bundled with flash on the biggest
  hits.
- `mood-redalert` / `mood-coldcyan` / `mood-corruption` — a full-screen tint
  overlay (semi-transparent color, low opacity) that lingers with the mood.
- `reduced` — set when `reduced_effects` is on; CSS uses it to disable all
  motion (flash/glitch/shake become no-ops; mood tint may stay as static color).

The bridge is **idempotent** (set/remove classes to match state each frame) and
adds no per-frame allocation beyond class toggles. Banner needs **no** web
special-casing — it is grid content and already renders.

### 8c. Manifest / theme

Update `manifest.json` theme color if needed to match the darker shell. Minor.

## 9. Content authoring (the major beats)

Annotate existing beats with effects (exact line placement during
implementation). Restrained — only these:

| Beat (file) | Effect treatment |
|---|---|
| SMOBB first contact, all-caps intro (hub/smobb) | `Mood(ColdCyan)` on entry, `Mood(Normal)` after the monologue |
| Password gate result (ali.ron `ENTER THE PASSWORD:`) | correct → `Banner("ACCESS GRANTED")` + `Flash(Green)`; wrong → `Banner("DENIED")` + `Flash(Red)` |
| Deleting the Tranava files | `Banner("DELETING")` + `Flash(White)` + brief `Mood(Corruption)` |
| Self-destruct countdown (bader.ron `SELF DESTRUCTING IN 3...`) | `Mood(RedAlert)`, `Flash(Red)` on each of 3/2/1, `Banner` on detonation, then `Mood(Normal)` |
| Breaches (ali.ron "Another breach?!") | `Glitch(900)` + `Flash(Cyan)` |
| Jaffar hex lines pre-decode | `Mood(Corruption)` / `Glitch` while he speaks; settles after `UnlockDecode` |
| Ending self-deletion (ending.ron) | `Banner` + fade via `Mood` then `Normal` |

These are additive RON edits; no structural content change. All existing
content tests still parse (the `Effect` variant is new but unused by current
fixtures).

## 10. Testing strategy

- **smobb-core:**
  - `effect.rs` / scene: RON round-trip of `Step::Effect(EffectKind)` incl. each
    `Mood` and `Flash(Color)`.
  - engine: `Step::Effect` emits exactly one `Output::Effect`, with **no**
    `Sleep`, and is emitted even with `skip == true`.
  - update the one exhaustive-match test.
- **smobb-tui-core (`App`):**
  - `tick` on `Output::Effect`: flash sets `flash_until` + blocks ~`FLASH_MS`;
    banner sets `banner` + blocks ~`BANNER_MS`; glitch is non-blocking; mood
    sets `mood`.
  - `enqueue` clears all effect state (including mood).
  - hold-Space caps a banner/flash block to ~`FAST_STEP_MS`.
  - `reduced_effects` suppresses flash/glitch but keeps banner + mood.
  - F3 toggles + sets `dirty`; snapshot/restore round-trips `reduced_effects`.
- **ui.rs (TestBackend buffer dump):**
  - banner text appears centered over the story area while active.
  - speaker gutter appears on speaker change, absent on same-speaker lines.
- **smobb-web:** no automated DOM test (WASM); manual verification checklist in
  the plan. Keep the bridge tiny and obviously correct.

## 11. Phasing

1. **Core effect model** — `effect.rs`, `Step::Effect`, `Output::Effect`,
   engine pass-through, exhaustive-match fixes + tests. *(No visible change.)*
2. **Speaker identity + structural polish** — `Output::Line.speaker`, grid
   header/grouping/separators/sidebar/journal styling in `ui.rs`. *(Visible
   immediately in the native TUI; cross-platform.)*
3. **Grid effect runtime + rendering** — `App` effect state/timing, F3 +
   persistence, banner/flash/glitch/mood grid rendering, reduced-motion logic.
4. **Web shell** — CRT baseline CSS, effect classes, `main.rs` bridge,
   `prefers-reduced-motion`.
5. **Content annotation** — apply the §9 effect map to the real beats; tune
   intensity in the running app.

Each phase is independently testable and leaves the game working. Phases 1–3 are
verified by `cargo test`; phase 4 by `trunk serve` + the manual checklist;
phase 5 by playthrough.

## 12. Risks & mitigations

- **Test churn from `Output::Line.speaker`** — mechanical; isolate in phase 2,
  run `cargo test` after.
- **Effects nauseating / inaccessible** — `prefers-reduced-motion` + F3 toggle +
  restrained defaults.
- **Banner ASCII font scope creep** — ship a minimal 5-row A–Z/0–9 set; plain
  centered fallback for anything missing. Don't gold-plate.
- **ratzilla cell-styling limits** — anything the grid can't express (RGB-split,
  true bloom) lives in the CSS layer; grid keeps a correct-but-simpler version so
  native TUI still reads well.
- **Mood leaking across scenes** — `enqueue()` hard-resets effect state.
```
