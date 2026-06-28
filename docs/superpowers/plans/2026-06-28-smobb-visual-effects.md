# S.M.O.B.B. Visual Polish & Dramatic Event Effects — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add baseline visual polish (speaker grouping, CRT atmosphere) and a small vocabulary of engine-authored dramatic effects (flash, glitch, banner, mood) fired at the game's major story beats.

**Architecture:** Effects are first-class content (`Step::Effect`) that flow through the pure engine as `Output::Effect`, are timed by the `smobb-tui-core` `App`, and render per-platform — a correct grid version in `ui.rs` (native TUI + web) plus a CSS enhancement layer in the web shell. Speaker identity is threaded onto `Output::Line` so the renderer can group and label turns.

**Tech Stack:** Rust, ratatui (grid render), ratzilla (WASM DOM backend), serde + ron (content), web-sys (web DOM bridge), CSS (web atmosphere).

## Global Constraints

- No new crate dependencies. `ron`, `serde`, `serde_json`, `web-sys`, `web-time`, `ratatui`, `ratzilla` are already present.
- Effects are emitted **regardless of engine skip mode**; skip only suppresses `Sleep`. Pacing/timing is the renderer's job.
- Every effect MUST have a cross-platform grid implementation. The web CSS layer only enhances; the native TUI is never left blank.
- Restrained intensity defaults — constants: `FLASH_MS = 120`, `BANNER_MS = 1400`.
- Accessibility is required: a `reduced_effects` toggle (key `F3`, persisted like `skip`) plus CSS `@media (prefers-reduced-motion: reduce)`.
- `enqueue()` hard-resets all effect state (no mood leaking between commands).
- TDD: write the failing test first, watch it fail, implement minimally, watch it pass, commit. Run `cargo test -p <crate>` from the repo root.
- Commit after every task.

---

## File Structure

**smobb-core**
- Create `crates/smobb-core/src/effect.rs` — `EffectKind`, `Mood` enums (the effect vocabulary).
- Modify `crates/smobb-core/src/lib.rs` — export `effect`.
- Modify `crates/smobb-core/src/speaker.rs` — derive `Deserialize` on `Color`; add `Speaker::label`.
- Modify `crates/smobb-core/src/scene.rs` — `Step::Effect`.
- Modify `crates/smobb-core/src/engine.rs` — `Output::Effect`, `Output::Line.speaker`, pass-through, `reduced_effects` field + accessors + snapshot/restore.
- Modify `crates/smobb-core/src/save.rs` — `reduced_effects` field.

**smobb-tui-core**
- Modify `crates/smobb-tui-core/src/app.rs` — effect runtime (state, tick handling, getters, enqueue reset), `Span.speaker`, `Key::F3`.
- Modify `crates/smobb-tui-core/src/ui.rs` — speaker grouping/labels, sidebar styling, banner/flash/mood/glitch grid rendering.

**smobb-tui**
- Modify `crates/smobb-tui/src/main.rs` — map `F(3)` → `Key::F3`, add to debounce set.

**smobb-web**
- Modify `crates/smobb-web/src/main.rs` — map `F(3)`, effect→DOM-class bridge.
- Modify `crates/smobb-web/style.css` — CRT baseline + effect classes.
- Modify `crates/smobb-web/index.html` — flash/mood overlay divs + `FX` control button.

---

## Phase 1 — Core effect model

### Task 1: Effect vocabulary module

**Files:**
- Create: `crates/smobb-core/src/effect.rs`
- Modify: `crates/smobb-core/src/lib.rs:2-8` (add `pub mod effect;`)
- Modify: `crates/smobb-core/src/speaker.rs:19` (derive `Deserialize` on `Color`)
- Test: in `crates/smobb-core/src/effect.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `smobb_core::effect::EffectKind` = `Flash(Color) | Glitch(u16) | Banner(String) | Mood(Mood)`; `smobb_core::effect::Mood` = `Normal | RedAlert | ColdCyan | Corruption` (both `Deserialize`, `PartialEq`, `Eq`, `Clone`; `Mood` also `Copy`).

- [ ] **Step 1: Derive `Deserialize` on `Color`**

In `crates/smobb-core/src/speaker.rs`, change the `Color` derive line:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum Color {
```

- [ ] **Step 2: Write the effect module with a failing test**

Create `crates/smobb-core/src/effect.rs`:

```rust
//! The dramatic-effect vocabulary, authored into content and rendered per-platform.
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub enum EffectKind {
    /// One-shot screen flash (~120ms). Tint reuses the speaker palette.
    Flash(crate::speaker::Color),
    /// Corruption burst for N ms (char scramble + web RGB-split/jitter).
    Glitch(u16),
    /// Full-screen takeover headline for a fixed duration. Cross-platform.
    Banner(String),
    /// Sustained palette/tint until `Mood::Normal` or the next command.
    Mood(Mood),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Mood {
    Normal,
    RedAlert,
    ColdCyan,
    Corruption,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::Color;

    #[test]
    fn parses_each_effect_variant() {
        assert_eq!(ron::from_str::<EffectKind>("Flash(Red)").unwrap(), EffectKind::Flash(Color::Red));
        assert_eq!(ron::from_str::<EffectKind>("Glitch(900)").unwrap(), EffectKind::Glitch(900));
        assert_eq!(
            ron::from_str::<EffectKind>("Banner(\"DELETING\")").unwrap(),
            EffectKind::Banner("DELETING".to_string())
        );
        assert_eq!(ron::from_str::<EffectKind>("Mood(RedAlert)").unwrap(), EffectKind::Mood(Mood::RedAlert));
    }
}
```

- [ ] **Step 3: Export the module**

In `crates/smobb-core/src/lib.rs`, add after `pub mod decode;`:

```rust
pub mod effect;
```

- [ ] **Step 4: Run the test (expect FAIL first if run before Step 3, then PASS)**

Run: `cargo test -p smobb-core effect::tests::parses_each_effect_variant`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/effect.rs crates/smobb-core/src/lib.rs crates/smobb-core/src/speaker.rs
git commit -m "feat(core): effect vocabulary (EffectKind + Mood)"
```

---

### Task 2: `Step::Effect` content variant

**Files:**
- Modify: `crates/smobb-core/src/scene.rs:5-18` (add variant)
- Test: `crates/smobb-core/src/scene.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `crate::effect::EffectKind` (Task 1).
- Produces: `Step::Effect(crate::effect::EffectKind)` parseable from RON.

- [ ] **Step 1: Write the failing test**

Add to `crates/smobb-core/src/scene.rs` tests module:

```rust
#[test]
fn parses_effect_step() {
    let s: Step = ron::from_str(r#"Effect(Banner("BOOM"))"#).unwrap();
    assert!(matches!(s, Step::Effect(crate::effect::EffectKind::Banner(t)) if t == "BOOM"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core scene::tests::parses_effect_step`
Expected: FAIL (no variant `Effect`)

- [ ] **Step 3: Add the variant**

In `crates/smobb-core/src/scene.rs`, add to the `Step` enum (after `UnlockDecode,`):

```rust
    /// Fire a dramatic effect at this point in the stream.
    Effect(crate::effect::EffectKind),
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p smobb-core scene::tests::parses_effect_step`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/scene.rs
git commit -m "feat(core): Step::Effect content variant"
```

---

### Task 3: `Output::Effect` engine pass-through

**Files:**
- Modify: `crates/smobb-core/src/engine.rs:57-65` (Output enum), `:207-234` (emit_intro/emit_step_lines), `:446-529` (run match), `:716-748` (kinds-match test)
- Test: `crates/smobb-core/src/engine.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `Step::Effect` (Task 2).
- Produces: `Output::Effect(crate::effect::EffectKind)`; emitted once per `Step::Effect`, no `Sleep`, even when `skip == true`.

- [ ] **Step 1: Write the failing test**

Add to `crates/smobb-core/src/engine.rs` tests module:

```rust
#[test]
fn effect_step_emits_output_effect_even_when_skipping() {
    use crate::content;
    use crate::effect::EffectKind;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Effect(Flash(Red)), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("go");
    assert!(out.iter().any(|o| matches!(o, Output::Effect(EffectKind::Flash(_)))));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core engine::tests::effect_step_emits_output_effect_even_when_skipping`
Expected: FAIL (no variant `Output::Effect`)

- [ ] **Step 3: Add the `Output` variant**

In `crates/smobb-core/src/engine.rs`, add to the `Output` enum (after `End,`):

```rust
    Effect(crate::effect::EffectKind),
```

- [ ] **Step 4: Pass `Step::Effect` through in `run()`**

In `run()`, add a match arm (after the `Step::UnlockDecode => { ... }` arm, before the closing brace of the `match step`):

```rust
                Step::Effect(k) => {
                    out.push(Output::Effect(k));
                }
```

- [ ] **Step 5: Pass it through in `emit_intro` and `emit_step_lines`**

In `emit_intro`, add to its `match s` (alongside `Step::Pause`):

```rust
                Step::Effect(k) => out.push(Output::Effect(k.clone())),
```

In `emit_step_lines`, replace the body with:

```rust
    fn emit_step_lines(&self, step: &Step, out: &mut Vec<Output>) {
        match step {
            Step::Line(sp, text) => out.push(Output::Line {
                text: text.clone(),
                color: sp.color(),
            }),
            Step::Effect(k) => out.push(Output::Effect(k.clone())),
            _ => {}
        }
    }
```

- [ ] **Step 6: Fix the exhaustive-match test**

In `crates/smobb-core/src/engine.rs`, in test `runs_a_command_scene_then_returns_to_menu`, add an arm to the `.map(|o| match o { ... })`:

```rust
                Output::Effect(_) => "X",
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p smobb-core`
Expected: PASS (all, including the new test)

- [ ] **Step 8: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): Output::Effect engine pass-through (skip-independent)"
```

---

## Phase 2 — Speaker identity + structural polish

### Task 4: Thread speaker onto `Output::Line`

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (Output enum + all `Output::Line` construction sites; add `use crate::speaker::Speaker;`)
- Modify: `crates/smobb-tui-core/src/app.rs:151-156,216-220` (tick/skip_reveal patterns)
- Modify: `crates/smobb-tui-core/src/ui.rs:32-38` (output_lines_to_text pattern)
- Test: `crates/smobb-core/src/engine.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `Output::Line { text: String, color: Color, speaker: Option<Speaker> }`. Dialogue lines carry `Some(sp)`; UI/menu/system-framing lines carry `None`.

- [ ] **Step 1: Write the failing test**

Add to `crates/smobb-core/src/engine.rs` tests module:

```rust
#[test]
fn dialogue_line_carries_speaker_journal_line_does_not() {
    use crate::content;
    use crate::speaker::Speaker;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt:[Line(Smobb,"P")],
            commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"hi"), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("go");
    assert!(out.iter().any(|o| matches!(o, Output::Line { speaker: Some(Speaker::Ali), .. })));
    assert!(e.journal_view().iter().all(|o| matches!(o, Output::Line { speaker: None, .. })));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core engine::tests::dialogue_line_carries_speaker_journal_line_does_not`
Expected: FAIL (no field `speaker`)

- [ ] **Step 3: Add the field + import**

In `crates/smobb-core/src/engine.rs`, change the import line `use crate::speaker::Color;` to:

```rust
use crate::speaker::{Color, Speaker};
```

Change the `Output::Line` variant to:

```rust
    Line { text: String, color: Color, speaker: Option<Speaker> },
```

- [ ] **Step 4: Set `Some(..)` at the three dialogue emit sites**

In `emit_intro`, the `Step::Line` arm:

```rust
                Step::Line(sp, text) => {
                    out.push(Output::Line {
                        text: text.clone(),
                        color: sp.color(),
                        speaker: Some(*sp),
                    });
                    if !self.skip {
                        out.push(Output::Sleep(self.interval_ms));
                    }
                }
```

In `emit_step_lines`, the `Step::Line` arm:

```rust
            Step::Line(sp, text) => out.push(Output::Line {
                text: text.clone(),
                color: sp.color(),
                speaker: Some(*sp),
            }),
```

In `run()`, the `Step::Line` arm — change only the `out.push(Output::Line { ... })` head to include `speaker: Some(sp),` (keep the rest of that arm's decode/sleep logic intact):

```rust
                    out.push(Output::Line {
                        text: text.clone(),
                        color: sp.color(),
                        speaker: Some(sp),
                    });
```

- [ ] **Step 5: Set `speaker: None` at every UI/framing site**

Add `speaker: None,` to **every remaining** `Output::Line { text: …, color: … }` literal in `engine.rs`. These are in: `options_view` (the `"You can:"`, `"  - {label}"`, `"Known passwords (try these):"`, `known`, and `"(no options)"` lines), `journal_view` (all section headers + items), `recap_view` (the `"(empty)"` and per-entry lines), and `emit_pause` (the `"."` dots → `speaker: None`). Example for `emit_pause`:

```rust
    fn emit_pause(&self, out: &mut Vec<Output>) {
        for _ in 0..3 {
            out.push(Output::Line {
                text: ".".into(),
                color: crate::speaker::Color::White,
                speaker: None,
            });
            if !self.skip {
                out.push(Output::Sleep(PAUSE_STEP_MS));
            }
        }
    }
```

- [ ] **Step 6: Make the cross-crate consumers compile (ignore speaker for now)**

In `crates/smobb-tui-core/src/ui.rs`, `output_lines_to_text`, change the pattern to:

```rust
            Output::Line { text, color, .. } => Some(Line::from(RSpan::styled(
```

In `crates/smobb-tui-core/src/app.rs`, `tick`, change the arm head to:

```rust
                Output::Line { text, color, .. } => {
```

and in `skip_reveal`:

```rust
                Output::Line { text, color, .. } => self.transcript.push(Span {
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p smobb-core && cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add crates/smobb-core/src/engine.rs crates/smobb-tui-core/src/app.rs crates/smobb-tui-core/src/ui.rs
git commit -m "feat(core): thread speaker identity onto Output::Line"
```

---

### Task 5: Speaker labels + turn grouping in the grid

**Files:**
- Modify: `crates/smobb-core/src/speaker.rs` (add `Speaker::label`)
- Modify: `crates/smobb-tui-core/src/app.rs:12-17` (`Span.speaker`), `:151-183` (tick), `:216-230` (skip_reveal); add `Speaker` import
- Modify: `crates/smobb-tui-core/src/ui.rs` (extract + use `story_lines`)
- Test: `crates/smobb-core/src/speaker.rs`, `crates/smobb-tui-core/src/ui.rs`

**Interfaces:**
- Consumes: `Output::Line.speaker` (Task 4).
- Produces: `Speaker::label(self) -> &'static str`; `ui::story_lines(transcript: &[Span]) -> Vec<Line<'static>>` rendering a dim header line on each speaker change.

- [ ] **Step 1: Write the failing test for `Speaker::label`**

Add to `crates/smobb-core/src/speaker.rs` tests module:

```rust
#[test]
fn speaker_labels() {
    assert_eq!(Speaker::Ali.label(), "ALI");
    assert_eq!(Speaker::Smobb.label(), "SMOBB");
    assert_eq!(Speaker::BaderMc.label(), "BADER");
    assert_eq!(Speaker::System.label(), ""); // system has no banner label
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core speaker::tests::speaker_labels`
Expected: FAIL (no method `label`)

- [ ] **Step 3: Implement `Speaker::label`**

In `crates/smobb-core/src/speaker.rs`, add to `impl Speaker` (next to `color`):

```rust
    /// Short upper-case display label for the transcript turn header.
    /// Empty for `System` (narration shows no header).
    pub fn label(self) -> &'static str {
        match self {
            Speaker::Ali => "ALI",
            Speaker::Bader | Speaker::BaderMc => "BADER",
            Speaker::Bevil => "BEVIL",
            Speaker::Hasan => "HASAN",
            Speaker::Josh | Speaker::JoshMc => "JOSH",
            Speaker::Qassim => "QASSIM",
            Speaker::Smobb => "SMOBB",
            Speaker::Jaffar => "JAFFAR",
            Speaker::Mavis => "MAVIS",
            Speaker::System => "",
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p smobb-core speaker::tests::speaker_labels`
Expected: PASS

- [ ] **Step 5: Store speaker on `Span`**

In `crates/smobb-tui-core/src/app.rs`, change the import `use smobb_core::speaker::Color;` to:

```rust
use smobb_core::speaker::{Color, Speaker};
```

Add a field to `Span`:

```rust
#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub color: Color,
    pub dim: bool,
    pub speaker: Option<Speaker>,
}
```

In `tick`, the `Output::Line` arm now captures speaker:

```rust
                Output::Line { text, color, speaker } => {
                    self.transcript.push(Span { text, color, dim: false, speaker });
                    self.reveal_at = None;
                }
```

Set `speaker: None` on the `Translation` and `Recap` pushes in `tick` (add the field to those two `Span { .. }` literals). Do the same for all `Span { .. }` literals in `skip_reveal` (`Output::Line { text, color, speaker } => self.transcript.push(Span { text, color, dim: false, speaker })`, and `speaker: None` for Translation/Recap).

- [ ] **Step 6: Write the failing test for `story_lines`**

Add to `crates/smobb-tui-core/src/ui.rs` tests module:

```rust
    #[test]
    fn speaker_change_inserts_one_header_per_turn() {
        use crate::app::Span;
        use smobb_core::speaker::{Color, Speaker};
        let t = vec![
            Span { text: "hi".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
            Span { text: "there".into(), color: Color::Aqua, dim: false, speaker: Some(Speaker::Ali) },
            Span { text: "YO".into(), color: Color::Salmon, dim: false, speaker: Some(Speaker::Bader) },
        ];
        let lines = super::story_lines(&t);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>())
            .collect();
        assert_eq!(flat.iter().filter(|s| s.trim() == "ALI").count(), 1);
        assert_eq!(flat.iter().filter(|s| s.trim() == "BADER").count(), 1);
        assert!(flat.iter().any(|s| s.contains("there")));
    }
```

- [ ] **Step 7: Run test to verify it fails**

Run: `cargo test -p smobb-tui-core ui::tests::speaker_change_inserts_one_header_per_turn`
Expected: FAIL (no function `story_lines`)

- [ ] **Step 8: Implement `story_lines` and use it in `render`**

In `crates/smobb-tui-core/src/ui.rs`, add the function (above `render`):

```rust
/// Build the transcript as ratatui lines, inserting a dim speaker header line
/// (and a blank spacer) whenever the speaker changes. `System`/`None` lines get
/// no header. This breaks the wall of same-color text into readable turns.
pub fn story_lines(transcript: &[crate::app::Span]) -> Vec<Line<'static>> {
    use smobb_core::speaker::Speaker;
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut prev: Option<Speaker> = None;
    for s in transcript {
        let label = s.speaker.map(|sp| sp.label()).unwrap_or("");
        let changed = s.speaker.is_some() && s.speaker != prev && !label.is_empty();
        if changed {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(RSpan::styled(
                label.to_string(),
                Style::default().fg(RColor::Rgb(120, 120, 120)).add_modifier(Modifier::DIM | Modifier::BOLD),
            )));
        }
        let mut style = Style::default().fg(to_rgb(s.color));
        if s.dim {
            style = style.add_modifier(Modifier::DIM).fg(RColor::Rgb(160, 160, 160));
        }
        lines.push(Line::from(RSpan::styled(s.text.clone(), style)));
        if s.speaker.is_some() && !label.is_empty() {
            prev = s.speaker;
        }
    }
    lines
}
```

In `render`, replace the `let story_lines: Vec<Line> = app.transcript().iter().map(...).collect();` block with:

```rust
    let story_lines: Vec<Line> = story_lines(app.transcript());
```

- [ ] **Step 9: Run tests**

Run: `cargo test -p smobb-tui-core`
Expected: PASS (including the existing `auto_scrolls_to_latest_line` and render tests)

- [ ] **Step 10: Commit**

```bash
git add crates/smobb-core/src/speaker.rs crates/smobb-tui-core/src/app.rs crates/smobb-tui-core/src/ui.rs
git commit -m "feat(tui): speaker labels + turn grouping in transcript"
```

---

### Task 6: Sidebar / journal emphasis + input dimming

**Files:**
- Modify: `crates/smobb-tui-core/src/ui.rs` (`output_lines_to_text` → emphasis; input hint style)
- Test: `crates/smobb-tui-core/src/ui.rs`

**Interfaces:**
- Produces: `ui::sidebar_line(o: &Output) -> Line<'static>` bolding section headers (`You can:`, `Known passwords…`, `[Clues]`-style brackets, `==`-wrapped titles).

- [ ] **Step 1: Write the failing test**

Add to `crates/smobb-tui-core/src/ui.rs` tests module:

```rust
    #[test]
    fn sidebar_headers_are_bold() {
        use smobb_core::engine::Output;
        use smobb_core::speaker::Color;
        let header = Output::Line { text: "[Clues]".into(), color: Color::White, speaker: None };
        let item = Output::Line { text: "  - a clue".into(), color: Color::White, speaker: None };
        let h = super::sidebar_line(&header);
        let i = super::sidebar_line(&item);
        let bold = |l: &ratatui::text::Line| {
            l.spans.iter().any(|s| s.style.add_modifier.contains(ratatui::style::Modifier::BOLD))
        };
        assert!(bold(&h), "section header should be bold");
        assert!(!bold(&i), "list item should not be bold");
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-tui-core ui::tests::sidebar_headers_are_bold`
Expected: FAIL (no function `sidebar_line`)

- [ ] **Step 3: Implement `sidebar_line` and route the sidebar through it**

In `crates/smobb-tui-core/src/ui.rs`, add:

```rust
/// Render one sidebar/journal line, emphasising section headers.
pub fn sidebar_line(o: &Output) -> Line<'static> {
    match o {
        Output::Line { text, color, .. } => {
            let trimmed = text.trim_start();
            let is_header = trimmed.starts_with('[')
                || trimmed.starts_with("==")
                || trimmed.ends_with(':');
            let mut style = Style::default().fg(to_rgb(*color));
            if is_header {
                style = style.add_modifier(Modifier::BOLD);
            } else if trimmed.starts_with('-') || trimmed.starts_with("(") {
                style = style.add_modifier(Modifier::DIM);
            }
            Line::from(RSpan::styled(text.clone(), style))
        }
        _ => Line::from(""),
    }
}
```

Replace `output_lines_to_text` usage in the sidebar block of `render` with a map over `sidebar_line`:

```rust
        let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
        let side = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().title(title));
```

(Leave `output_lines_to_text` in place if other callers exist; otherwise it may be removed.)

- [ ] **Step 4: Dim the input hint**

In `render`, replace the input line block with a styled two-span line so the prompt stands out and the hint recedes:

```rust
    let hint = "[Tab] sidebar  [hold Space] fast  [F2] instant  [F3] effects  ['leave' exits]  [Esc] quit";
    let input = Paragraph::new(Line::from(vec![
        RSpan::styled(format!("> {}", app.input()), Style::default().fg(RColor::Rgb(220, 220, 220))),
        RSpan::styled(format!("    {}", hint), Style::default().fg(RColor::Rgb(110, 110, 110)).add_modifier(Modifier::DIM)),
    ]));
    f.render_widget(input, input_area);
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/smobb-tui-core/src/ui.rs
git commit -m "feat(tui): sidebar header emphasis + dimmed input hint"
```

---

## Phase 3 — Grid effect runtime + rendering

### Task 7: App effect runtime (state, tick, getters, reset)

**Files:**
- Modify: `crates/smobb-tui-core/src/app.rs` (imports, fields, constants, `new`, `enqueue`, `tick`, getters)
- Test: `crates/smobb-tui-core/src/app.rs`

**Interfaces:**
- Consumes: `Output::Effect` (Task 3), `EffectKind`/`Mood` (Task 1).
- Produces on `App`: `flash(&self) -> Option<Color>`, `glitch(&self) -> bool`, `banner(&self) -> Option<&str>`, `mood(&self) -> Mood`. `tick` consumes `Output::Effect`; `enqueue` clears all effect state.

- [ ] **Step 1: Write the failing tests**

Add to `crates/smobb-tui-core/src/app.rs` tests module:

```rust
    #[test]
    fn flash_blocks_briefly_then_reveals() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use std::time::Duration;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Effect(Flash(Red)), Line(Ali,"after"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        let mut app = App::new(engine);
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0); // hits Effect(Flash) -> blocks ~120ms
        assert!(app.flash().is_some(), "flash active during the block");
        assert!(!app.transcript().iter().any(|s| s.text == "after"), "line withheld during flash");
        app.tick(t0 + Duration::from_millis(200));
        assert!(app.flash().is_none(), "flash expired");
        assert!(app.transcript().iter().any(|s| s.text == "after"), "line revealed after flash");
    }

    #[test]
    fn banner_shows_text_then_clears() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use std::time::Duration;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Effect(Banner("DELETING")), Line(Ali,"x"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        let mut app = App::new(engine);
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0);
        assert_eq!(app.banner(), Some("DELETING"));
        app.tick(t0 + Duration::from_millis(2000));
        assert_eq!(app.banner(), None);
    }

    #[test]
    fn glitch_is_non_blocking() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Effect(Glitch(900)), Line(Ali,"x"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        let mut app = App::new(engine);
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0);
        assert!(app.glitch(), "glitch active");
        assert!(app.transcript().iter().any(|s| s.text == "x"), "glitch does not block reveal");
    }

    #[test]
    fn mood_sets_and_enqueue_clears() {
        use smobb_core::content;
        use smobb_core::effect::Mood;
        use smobb_core::engine::Engine;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Effect(Mood(RedAlert)), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        let mut app = App::new(engine);
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0);
        assert_eq!(app.mood(), Mood::RedAlert);
        app.submit_for_test("go"); // re-enqueue -> reset
        assert_eq!(app.mood(), Mood::Normal);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-tui-core app::tests::mood_sets_and_enqueue_clears`
Expected: FAIL (no method `mood`)

- [ ] **Step 3: Add imports, constants, and fields**

In `crates/smobb-tui-core/src/app.rs`, add to the imports:

```rust
use smobb_core::effect::{EffectKind, Mood};
```

Add constants near `FAST_WINDOW_MS`:

```rust
/// One-shot screen-flash duration (ms).
const FLASH_MS: u64 = 120;
/// Banner takeover duration (ms).
const BANNER_MS: u64 = 1400;
```

Add fields to `struct App` (after `dirty: bool,`):

```rust
    flash_until: Option<Instant>,
    flash_tone: Color,
    glitch_until: Option<Instant>,
    banner: Option<String>,
    banner_until: Option<Instant>,
    mood: Mood,
    last_tick: Instant,
```

In `App::new`, add to the struct literal initialisation (before `app.enqueue(outputs);`):

```rust
            flash_until: None,
            flash_tone: Color::White,
            glitch_until: None,
            banner: None,
            banner_until: None,
            mood: Mood::Normal,
            last_tick: Instant::now(),
```

- [ ] **Step 4: Reset effects in `enqueue`**

Add to the end of `enqueue`:

```rust
        self.flash_until = None;
        self.glitch_until = None;
        self.banner = None;
        self.banner_until = None;
        self.mood = Mood::Normal;
```

- [ ] **Step 5: Handle `Output::Effect` in `tick`**

At the very top of `tick` (before the `if self.awaiting || self.quit` guard) record the clock:

```rust
        self.last_tick = now;
```

Add an arm to the `match item` in the reveal loop (alongside the other `Output::` arms):

```rust
                Output::Effect(kind) => match kind {
                    EffectKind::Flash(tone) => {
                        self.flash_tone = tone;
                        let dur = if fast { FLASH_MS.min(FAST_STEP_MS) } else { FLASH_MS };
                        self.flash_until = Some(now + std::time::Duration::from_millis(dur));
                        self.reveal_at = Some(now + std::time::Duration::from_millis(dur));
                        return;
                    }
                    EffectKind::Banner(text) => {
                        self.banner = Some(text);
                        let dur = if fast { BANNER_MS.min(FAST_STEP_MS) } else { BANNER_MS };
                        self.banner_until = Some(now + std::time::Duration::from_millis(dur));
                        self.reveal_at = Some(now + std::time::Duration::from_millis(dur));
                        return;
                    }
                    EffectKind::Glitch(ms) => {
                        self.glitch_until = Some(now + std::time::Duration::from_millis(ms as u64));
                        // non-blocking: keep revealing
                    }
                    EffectKind::Mood(m) => {
                        self.mood = m;
                        // non-blocking, instantaneous
                    }
                },
```

Also add `Output::Effect(_) => {}` to the `match item` inside `skip_reveal` (effects are ignored by the test-only bulk drain).

- [ ] **Step 6: Add the getters**

Add to `impl App` (near the other getters):

```rust
    pub fn flash(&self) -> Option<Color> {
        if self.flash_until.is_some_and(|u| self.last_tick < u) {
            Some(self.flash_tone)
        } else {
            None
        }
    }
    pub fn glitch(&self) -> bool {
        self.glitch_until.is_some_and(|u| self.last_tick < u)
    }
    pub fn banner(&self) -> Option<&str> {
        if self.banner_until.is_some_and(|u| self.last_tick < u) {
            self.banner.as_deref()
        } else {
            None
        }
    }
    pub fn mood(&self) -> Mood {
        self.mood
    }
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 8: Commit**

```bash
git add crates/smobb-tui-core/src/app.rs
git commit -m "feat(tui): App effect runtime (flash/glitch/banner/mood) + reset"
```

---

### Task 8: `reduced_effects` toggle + persistence + F3

**Files:**
- Modify: `crates/smobb-core/src/save.rs:15-56` (field + Default)
- Modify: `crates/smobb-core/src/engine.rs` (field, accessors, snapshot, restore)
- Modify: `crates/smobb-tui-core/src/app.rs` (`Key::F3`, handling, reduced gating, accessor)
- Modify: `crates/smobb-tui/src/main.rs:24-25,106` (map F3, debounce)
- Modify: `crates/smobb-web/src/main.rs:40-41` (map F3)
- Test: `crates/smobb-core/src/engine.rs`, `crates/smobb-tui-core/src/app.rs`

**Interfaces:**
- Produces: `Engine::reduced_effects(&self) -> bool`, `Engine::set_reduced_effects(&mut self, bool)`; `SaveState.reduced_effects: bool`; `App::reduced_effects(&self) -> bool`; `Key::F3`.

- [ ] **Step 1: Write the failing tests**

Add to `crates/smobb-tui-core/src/app.rs` tests module:

```rust
    #[test]
    fn f3_toggles_reduced_effects_and_marks_dirty() {
        let mut app = app_fixture();
        assert!(!app.reduced_effects());
        app.key(Key::F3);
        assert!(app.reduced_effects());
        assert!(app.dirty());
    }

    #[test]
    fn reduced_effects_suppresses_flash_block() {
        use smobb_core::content;
        use smobb_core::engine::Engine;
        use web_time::Instant;
        let c = content::from_str(&[r#"ContentPart(
            locations: { "hub": ( prompt:[Line(Smobb,"P")],
                commands: [ Branch(keys:["go"], target: Steps([ Effect(Flash(Red)), Line(Ali,"after"), Return ])) ] ) },
            scenes: {},
        )"#]);
        let mut engine = Engine::new(c);
        engine.set_skip(true);
        engine.set_reduced_effects(true);
        let mut app = App::new(engine);
        let t0 = Instant::now();
        app.tick(t0);
        app.submit_for_test("go");
        app.tick(t0);
        assert!(app.flash().is_none(), "no flash under reduced effects");
        assert!(app.transcript().iter().any(|s| s.text == "after"), "no block under reduced effects");
    }
```

Add to `crates/smobb-core/src/engine.rs` tests module:

```rust
#[test]
fn reduced_effects_round_trips_through_snapshot() {
    use crate::content;
    let mut e = Engine::new(content::from_str(&[r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#]));
    e.set_reduced_effects(true);
    let snap = e.snapshot();
    assert!(snap.reduced_effects);
    let snap2 = crate::save::SaveState::from_json(&snap.to_json()).unwrap();
    let mut e2 = Engine::new(content::from_str(&[r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#]));
    e2.restore(snap2);
    assert!(e2.reduced_effects());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-core engine::tests::reduced_effects_round_trips_through_snapshot`
Expected: FAIL (no field `reduced_effects`)

- [ ] **Step 3: Add the SaveState field**

In `crates/smobb-core/src/save.rs`, add to the struct (after `decoding_unlocked`):

```rust
    /// Accessibility: disable motion-heavy effects (flash/glitch).
    #[serde(default)]
    pub reduced_effects: bool,
```

Add to `Default::default`:

```rust
            reduced_effects: false,
```

- [ ] **Step 4: Add the engine field, accessors, snapshot/restore**

In `crates/smobb-core/src/engine.rs`, add field to `struct Engine` (after `decoding_unlocked: bool,`):

```rust
    reduced_effects: bool,
```

Init in `Engine::new` struct literal:

```rust
            reduced_effects: false,
```

Add accessors (next to `skip`/`set_skip`):

```rust
    pub fn reduced_effects(&self) -> bool {
        self.reduced_effects
    }
    pub fn set_reduced_effects(&mut self, on: bool) {
        self.reduced_effects = on;
    }
```

In `snapshot`, add to the `SaveState { .. }` literal:

```rust
            reduced_effects: self.reduced_effects,
```

In `restore`, add:

```rust
        self.reduced_effects = s.reduced_effects;
```

- [ ] **Step 5: Add `Key::F3` + handling + accessor + reduced gating**

In `crates/smobb-tui-core/src/app.rs`, add `F3` to the `Key` enum (after `F2,`):

```rust
    F3,
```

In `key`, add an arm (after the `Key::F2 => { ... }` arm):

```rust
            Key::F3 => {
                let r = self.engine.reduced_effects();
                self.engine.set_reduced_effects(!r);
                self.dirty = true; // persist the accessibility setting
            }
```

Add an accessor:

```rust
    pub fn reduced_effects(&self) -> bool {
        self.engine.reduced_effects()
    }
```

Gate the motion effects in the `Output::Effect` handler from Task 7 — change the `Flash` and `Glitch` arms:

```rust
                    EffectKind::Flash(tone) => {
                        if self.engine.reduced_effects() {
                            continue; // no flash, no block
                        }
                        self.flash_tone = tone;
                        let dur = if fast { FLASH_MS.min(FAST_STEP_MS) } else { FLASH_MS };
                        self.flash_until = Some(now + std::time::Duration::from_millis(dur));
                        self.reveal_at = Some(now + std::time::Duration::from_millis(dur));
                        return;
                    }
                    ...
                    EffectKind::Glitch(ms) => {
                        if !self.engine.reduced_effects() {
                            self.glitch_until = Some(now + std::time::Duration::from_millis(ms as u64));
                        }
                    }
```

(Banner and Mood remain unconditional — they are information / static tint.)

- [ ] **Step 6: Map F3 in both frontends**

In `crates/smobb-tui/src/main.rs`, add to `to_key`'s match (after `KeyCode::F(2) => Key::F2,`):

```rust
        KeyCode::F(3) => Key::F3,
```

In `run`, include F3 in the debounce set — change the `toggling` line:

```rust
                        let toggling = matches!(key, Key::F1 | Key::F2 | Key::F3);
```

In `crates/smobb-web/src/main.rs`, add to `to_key`'s match (after `KeyCode::F(2) => Key::F2,`):

```rust
        KeyCode::F(3) => Key::F3,
```

- [ ] **Step 7: Run tests**

Run: `cargo test -p smobb-core && cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 8: Build the frontends to confirm they compile**

Run: `cargo build -p smobb-tui`
Expected: builds clean.

- [ ] **Step 9: Commit**

```bash
git add crates/smobb-core/src/save.rs crates/smobb-core/src/engine.rs crates/smobb-tui-core/src/app.rs crates/smobb-tui/src/main.rs crates/smobb-web/src/main.rs
git commit -m "feat: reduced-effects accessibility toggle (F3) + persistence"
```

---

### Task 9: Banner grid rendering

**Files:**
- Modify: `crates/smobb-tui-core/src/ui.rs` (`banner_lines` + render branch)
- Test: `crates/smobb-tui-core/src/ui.rs`

**Interfaces:**
- Consumes: `App::banner()` (Task 7), `App::mood()` (Task 7).
- Produces: `ui::banner_lines(text: &str, height: u16) -> Vec<Line<'static>>` — vertically centered, boxed, letter-spaced uppercase.

- [ ] **Step 1: Write the failing test**

Add to `crates/smobb-tui-core/src/ui.rs` tests module:

```rust
    #[test]
    fn banner_is_letterspaced_and_centered() {
        let lines = super::banner_lines("SELF", 11);
        let flat: Vec<String> = lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.to_string()).collect::<String>())
            .collect();
        // letter-spaced text present
        assert!(flat.iter().any(|s| s.contains("S E L F")));
        // vertically padded (more lines than just the text + box)
        assert_eq!(lines.len(), 11);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-tui-core ui::tests::banner_is_letterspaced_and_centered`
Expected: FAIL (no function `banner_lines`)

- [ ] **Step 3: Implement `banner_lines`**

In `crates/smobb-tui-core/src/ui.rs`, add:

```rust
/// Build a dramatic full-area banner: the text upper-cased and letter-spaced,
/// vertically centered. `height` is the available story-area inner height.
pub fn banner_lines(text: &str, height: u16) -> Vec<Line<'static>> {
    let spaced: String = text
        .to_uppercase()
        .chars()
        .collect::<Vec<_>>()
        .chunks(1)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ");
    let mut lines: Vec<Line<'static>> = Vec::new();
    let h = height as usize;
    let pad = h.saturating_sub(1) / 2;
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    lines.push(
        Line::from(RSpan::styled(
            spaced,
            Style::default().add_modifier(Modifier::BOLD),
        ))
        .alignment(ratatui::layout::Alignment::Center),
    );
    while (lines.len() as u16) < height {
        lines.push(Line::from(""));
    }
    lines
}
```

- [ ] **Step 4: Render the banner over the story area**

In `render`, after `story_area`/`sidebar_area` are computed and before building the transcript paragraph, branch on the banner:

```rust
    if let Some(text) = app.banner() {
        let tone = match app.mood() {
            smobb_core::effect::Mood::RedAlert => RColor::Rgb(255, 60, 60),
            smobb_core::effect::Mood::ColdCyan => RColor::Rgb(120, 220, 255),
            smobb_core::effect::Mood::Corruption => RColor::Rgb(120, 255, 120),
            smobb_core::effect::Mood::Normal => RColor::Rgb(255, 255, 255),
        };
        let inner_h = story_area.height.saturating_sub(2);
        let banner = Paragraph::new(banner_lines(text, inner_h))
            .style(Style::default().fg(tone))
            .block(Block::bordered())
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(banner, story_area);
        // still render the sidebar so the layout doesn't jump
        if let Some(sa) = sidebar_area {
            let (title, outs) = match app.tab() {
                crate::app::Tab::Options => (" OPTIONS ", app.options_lines()),
                crate::app::Tab::Journal => (" JOURNAL ", app.journal_lines()),
            };
            let lines: Vec<Line> = outs.iter().map(sidebar_line).collect();
            f.render_widget(
                Paragraph::new(lines).wrap(Wrap { trim: false }).block(Block::bordered().title(title)),
                sa,
            );
        }
        // input line
        let input = Paragraph::new(Line::from(format!("> {}", app.input())));
        f.render_widget(input, input_area);
        return;
    }
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/smobb-tui-core/src/ui.rs
git commit -m "feat(tui): full-area banner takeover rendering"
```

---

### Task 10: Flash / mood / glitch grid rendering

**Files:**
- Modify: `crates/smobb-tui-core/src/ui.rs` (`scramble` helper, story block style, glitch wiring)
- Test: `crates/smobb-tui-core/src/ui.rs`

**Interfaces:**
- Consumes: `App::flash()`, `App::mood()`, `App::glitch()`.
- Produces: `ui::scramble(s: &str, seed: usize) -> String` — deterministic, length-preserving corruption; `story_lines` gains a `glitch: bool` parameter.

- [ ] **Step 1: Write the failing test for `scramble`**

Add to `crates/smobb-tui-core/src/ui.rs` tests module:

```rust
    #[test]
    fn scramble_preserves_length_and_corrupts() {
        let s = "hello world";
        let g = super::scramble(s, 3);
        assert_eq!(g.chars().count(), s.chars().count());
        assert_ne!(g, s);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-tui-core ui::tests::scramble_preserves_length_and_corrupts`
Expected: FAIL (no function `scramble`)

- [ ] **Step 3: Implement `scramble` and add `glitch` to `story_lines`**

In `crates/smobb-tui-core/src/ui.rs`, add:

```rust
const GLITCH_GLYPHS: &[char] = &['#', '%', '@', '&', '░', '▒', '▓', '*'];

/// Deterministically corrupt ~1/3 of glyphs (no RNG — seeded by position).
pub fn scramble(s: &str, seed: usize) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_whitespace() {
                c
            } else if (i + seed) % 3 == 0 {
                GLITCH_GLYPHS[(i + seed) % GLITCH_GLYPHS.len()]
            } else {
                c
            }
        })
        .collect()
}
```

Change `story_lines` signature and corrupt the **last** transcript line when glitching:

```rust
pub fn story_lines(transcript: &[crate::app::Span], glitch: bool) -> Vec<Line<'static>> {
    use smobb_core::speaker::Speaker;
    let last = transcript.len().saturating_sub(1);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut prev: Option<Speaker> = None;
    for (idx, s) in transcript.iter().enumerate() {
        let label = s.speaker.map(|sp| sp.label()).unwrap_or("");
        let changed = s.speaker.is_some() && s.speaker != prev && !label.is_empty();
        if changed {
            if !lines.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(RSpan::styled(
                label.to_string(),
                Style::default().fg(RColor::Rgb(120, 120, 120)).add_modifier(Modifier::DIM | Modifier::BOLD),
            )));
        }
        let mut style = Style::default().fg(to_rgb(s.color));
        if s.dim {
            style = style.add_modifier(Modifier::DIM).fg(RColor::Rgb(160, 160, 160));
        }
        let text = if glitch && idx == last {
            scramble(&s.text, idx)
        } else {
            s.text.clone()
        };
        lines.push(Line::from(RSpan::styled(text, style)));
        if s.speaker.is_some() && !label.is_empty() {
            prev = s.speaker;
        }
    }
    lines
}
```

Update the existing Task-5 test `speaker_change_inserts_one_header_per_turn` call to `super::story_lines(&t, false)`.

- [ ] **Step 4: Wire flash + mood background and glitch into `render`**

In `render`, replace the transcript build + story widget construction with the glitch-aware version and a mood/flash background:

```rust
    let story_lines: Vec<Line> = story_lines(app.transcript(), app.glitch());
    let bg = if let Some(tone) = app.flash() {
        to_rgb(tone)
    } else {
        match app.mood() {
            smobb_core::effect::Mood::RedAlert => RColor::Rgb(24, 0, 0),
            smobb_core::effect::Mood::ColdCyan => RColor::Rgb(0, 14, 22),
            smobb_core::effect::Mood::Corruption => RColor::Rgb(0, 18, 0),
            smobb_core::effect::Mood::Normal => RColor::Reset,
        }
    };
```

Then add `.style(Style::default().bg(bg))` to the story `Paragraph` (the `story_base` builder), e.g.:

```rust
    let story_base = Paragraph::new(story_lines)
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(bg));
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p smobb-tui-core`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add crates/smobb-tui-core/src/ui.rs
git commit -m "feat(tui): flash background, mood tint, glitch scramble in grid"
```

---

## Phase 4 — Web shell (manual verification)

### Task 11: CRT baseline CSS (glow, scanlines, vignette)

**Files:**
- Modify: `crates/smobb-web/style.css`

**Interfaces:** none (pure styling). Verified manually.

- [ ] **Step 1: Add the CRT baseline**

Append to `crates/smobb-web/style.css`:

```css
/* --- CRT atmosphere ---------------------------------------------------- */

/* per-cell phosphor glow: currentColor = each cell's own color, so the
   per-speaker palette keeps its hue while glowing */
#grid span {
  text-shadow: 0 0 4px currentColor;
}

/* faint scanlines (kept clear of the control bar) */
body::before {
  content: "";
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  bottom: 52px;
  pointer-events: none;
  z-index: 9000;
  background: repeating-linear-gradient(
    0deg,
    rgba(0, 0, 0, 0.18) 0 1px,
    transparent 1px 3px
  );
}

/* vignette darkening the edges */
body::after {
  content: "";
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  bottom: 52px;
  pointer-events: none;
  z-index: 8999;
  background: radial-gradient(
    ellipse at center,
    transparent 55%,
    rgba(0, 0, 0, 0.55) 100%
  );
}

@media (min-width: 800px) {
  body::before, body::after { bottom: 40px; }
}

@media (prefers-reduced-motion: reduce) {
  /* no animated flicker is introduced here; scanlines remain static */
}
```

- [ ] **Step 2: Build the web bundle**

Run: `cd crates/smobb-web && trunk build`
Expected: builds clean (if `trunk` is unavailable, run `cargo build -p smobb-web --target wasm32-unknown-unknown` to confirm the crate compiles; CSS is verified in Step 3).

- [ ] **Step 3: Manual verification**

Run `trunk serve` in `crates/smobb-web`, open `localhost:8080`:
- Text has a soft glow matching each speaker color (not a global green).
- Subtle horizontal scanlines visible; edges darkened by vignette.
- Control bar at the bottom is unobstructed and clickable.

- [ ] **Step 4: Commit**

```bash
git add crates/smobb-web/style.css
git commit -m "feat(web): CRT baseline — phosphor glow, scanlines, vignette"
```

---

### Task 12: Web effect bridge + effect CSS classes

**Files:**
- Modify: `crates/smobb-web/src/main.rs` (effect→DOM-class sync)
- Modify: `crates/smobb-web/index.html` (overlay divs + FX button)
- Modify: `crates/smobb-web/style.css` (effect classes)

**Interfaces:**
- Consumes: `App::flash()`, `App::glitch()`, `App::mood()`, `App::reduced_effects()`.

- [ ] **Step 1: Add the overlay divs + FX button to `index.html`**

In `crates/smobb-web/index.html`, immediately after the opening `<body>` comment, add:

```html
    <!-- effect overlays toggled via <body> classes by the wasm bridge -->
    <div id="fx-flash" aria-hidden="true"></div>
    <div id="fx-mood" aria-hidden="true"></div>
```

In the `#controls` nav, add a button (after the `F2` button):

```html
      <button type="button" data-key="F3" title="Reduce effects">FX</button>
```

- [ ] **Step 2: Add the bridge to `main.rs`**

In `crates/smobb-web/src/main.rs`, add an import:

```rust
use smobb_core::effect::Mood;
```

Add a helper function (above `main`):

```rust
/// Mirror the App's effect state onto <body> classes so CSS can react.
fn sync_effects(app: &App) {
    let Some(body) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.body()) else {
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
```

In the `terminal.draw_web(...)` closure, call it after `tick` and before `ui::render`:

```rust
    terminal.draw_web(move |f| {
        {
            let mut a = app.borrow_mut();
            a.tick(web_time::Instant::now());
        }
        sync_effects(&app.borrow());
        ui::render(f, &app.borrow());
    });
```

- [ ] **Step 3: Add the effect CSS**

Append to `crates/smobb-web/style.css`:

```css
/* --- effect overlays --------------------------------------------------- */
#fx-flash, #fx-mood {
  position: fixed;
  left: 0;
  right: 0;
  top: 0;
  bottom: 52px;
  pointer-events: none;
  z-index: 9500;
  opacity: 0;
}
@media (min-width: 800px) { #fx-flash, #fx-mood { bottom: 40px; } }

/* one-shot flash */
body.fx-flash #fx-flash {
  background: #ffffff;
  animation: smobb-flash 120ms ease-out;
}
@keyframes smobb-flash {
  from { opacity: 0.85; }
  to   { opacity: 0; }
}

/* sustained mood tint */
body.mood-redalert   #fx-mood { background: #ff0000; opacity: 0.10; }
body.mood-coldcyan   #fx-mood { background: #00d0ff; opacity: 0.08; }
body.mood-corruption #fx-mood { background: #33ff33; opacity: 0.08; }

/* corruption glitch on the grid */
body.fx-glitch #grid {
  animation: smobb-glitch 180ms steps(2) infinite;
}
@keyframes smobb-glitch {
  0%   { transform: translate(0, 0); filter: none; }
  50%  { transform: translate(-2px, 1px); filter: hue-rotate(25deg) saturate(1.4); }
  100% { transform: translate(2px, -1px); filter: none; }
}

/* accessibility: kill all motion */
body.reduced #fx-flash,
body.reduced.fx-glitch #grid {
  animation: none !important;
}
body.reduced #fx-flash { opacity: 0; }
```

- [ ] **Step 4: Build**

Run: `cargo build -p smobb-web --target wasm32-unknown-unknown`
Expected: compiles clean.

- [ ] **Step 5: Manual verification**

`trunk serve`, then exercise a beat with effects (after Task 13, or temporarily add `Effect(...)` to a hub command):
- Flash briefly whitens the screen.
- A mood overlay tints the screen and lingers, clearing on the next command.
- Glitch jitters/hue-shifts the grid.
- Tapping `FX` (or pressing F3) stops flash/glitch motion (mood tint may remain).

- [ ] **Step 6: Commit**

```bash
git add crates/smobb-web/src/main.rs crates/smobb-web/index.html crates/smobb-web/style.css
git commit -m "feat(web): effect bridge + flash/glitch/mood CSS, FX toggle"
```

---

## Phase 5 — Content annotation

### Task 13: Annotate the major beats with effects

**Files:**
- Modify: `content/smobb.ron`, `content/ali.ron`, `content/bader.ron`, `content/ending.ron` (per the beat map)
- Test: `crates/smobb-core` content tests + manual playthrough

**Interfaces:**
- Consumes: `Step::Effect` (Task 2). All edits are additive RON; existing content tests must still parse.

Insert effects at these unique anchor lines (open each file, find the quoted anchor, insert the listed `Effect(...)` step(s) immediately before/after it, keeping RON comma syntax valid). Use restrained placement — only these.

- [ ] **Step 1: SMOBB cold-takeover mood (`content/smobb.ron`)**

At the start of S.M.O.B.B.'s all-caps monologue (its `intro` or first scene `steps`), insert as the first step:

```rust
Effect(Mood(ColdCyan)),
```

and as the last step before the monologue returns/awaits:

```rust
Effect(Mood(Normal)),
```

- [ ] **Step 2: Password gate result (`content/ali.ron`)**

Anchor: `Line(System, "ENTER THE PASSWORD:")` (around line 12) is followed by a `Prompt`. In the **correct-password** branch's steps, insert at the top:

```rust
Effect(Flash(Green)),
Effect(Banner("ACCESS GRANTED")),
```

In the **wrong-password** branch (the `Line(Ali, "Who are you! ...")` / default branch), insert at the top:

```rust
Effect(Flash(Red)),
Effect(Banner("DENIED")),
```

- [ ] **Step 3: Self-destruct countdown (`content/bader.ron`)**

Anchor: `Line(System, "SELF DESTRUCTING IN 3...")` (around line 43). Insert immediately **before** it:

```rust
Effect(Mood(RedAlert)),
Effect(Flash(Red)),
```

Add `Effect(Flash(Red)),` before each subsequent countdown line (the `"2..."`/`"1..."` lines if present), and after the final countdown line insert:

```rust
Effect(Banner("SELF DESTRUCT")),
Effect(Mood(Normal)),
```

- [ ] **Step 4: Deletion (`content/ali.ron` / wherever `DELETING`/`deleted the files` resolves)**

At the Tranava-files deletion beat (e.g. the branch behind `keys: ["delete"]` or the `"It is done."` resolution), insert before the confirming line:

```rust
Effect(Banner("DELETING")),
Effect(Flash(White)),
Effect(Mood(Corruption)),
```

and restore after:

```rust
Effect(Mood(Normal)),
```

- [ ] **Step 5: Breaches (`content/ali.ron`)**

Anchor: `Line(Ali, "My ears! Another breach?! Now?!")` (around line 29). Insert immediately **before** it:

```rust
Effect(Glitch(900)),
Effect(Flash(Cyan)),
```

- [ ] **Step 6: Ending self-deletion (`content/ending.ron`)**

Anchor: `Line(Smobb, "I WILL DELETE MYSELF. ...")` (around line 34). Insert **before** it:

```rust
Effect(Mood(Corruption)),
```

and after the final ending line, before its `Quit`/`Return`:

```rust
Effect(Banner("GOODBYE")),
Effect(Mood(Normal)),
```

- [ ] **Step 7: Verify content still parses**

Run: `cargo test -p smobb-core`
Expected: PASS (all content_*.rs integration tests + playthrough test still parse and run).

- [ ] **Step 8: Manual playthrough**

Run `cargo run -p smobb-tui` (native) and/or `trunk serve` (web). Walk each annotated beat and confirm the effect fires and reads well. Tune intensity by editing the `Effect(...)` placements only.

- [ ] **Step 9: Commit**

```bash
git add content/smobb.ron content/ali.ron content/bader.ron content/ending.ron
git commit -m "feat(content): annotate major beats with dramatic effects"
```

---

## Self-Review

**Spec coverage:**
- §4 effect model → Tasks 1–3. ✓
- §5 speaker labels → Tasks 4–5. ✓
- §6 effect runtime (state, timing, fast-forward cap, reduced, enqueue reset) → Tasks 7–8. ✓
- §7 grid rendering (polish + banner/flash/glitch/mood) → Tasks 5, 6, 9, 10. ✓
- §8 web shell (CRT baseline, effect classes, bridge, a11y) → Tasks 11–12. ✓
- §9 content authoring (beat map) → Task 13. ✓
- §10 testing → unit tests in every core/tui task; manual checklists for web. ✓
- §11 phasing → matches the five phases. ✓

**Placeholder scan:** No TBD/TODO; every code step shows complete code. Content edits (Task 13) use unique quoted anchors + exact `Effect(...)` snippets rather than vague instructions. ✓

**Type consistency:** `EffectKind`/`Mood` defined in Task 1, consumed by the same names everywhere. `Output::Line { text, color, speaker }` defined in Task 4, matched with `..` or full fields consistently. `story_lines` gains its `glitch` param in Task 10 with the Task-5 caller updated. `flash()`/`glitch()`/`banner()`/`mood()`/`reduced_effects()` signatures match between producer (App, Tasks 7–8) and consumers (ui.rs Tasks 9–10, web Task 12). ✓
