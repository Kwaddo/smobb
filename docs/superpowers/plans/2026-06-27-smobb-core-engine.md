# S.M.O.B.B. Core Engine + Content Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `smobb-core` — a pure, non-blocking Rust game engine — and port the entire S.M.O.B.B. narrative from Java into RON data files, producing a headless playable game with full tests.

**Architecture:** Content (RON data) ↔ engine (pure state machine, no I/O) ↔ a thin headless driver. The engine exposes `advance(input) -> Vec<Output>`; the host owns timing and I/O. This is the portability fix that lets the same engine later drive a ratatui TUI and a WASM/web terminal.

**Tech Stack:** Rust (edition 2021, stable ≥ 1.74), `serde` (derive), `ron` (content), `std` only otherwise. No async, no network, no Swing/GUI.

## Global Constraints

- Rust edition **2021**, toolchain **stable ≥ 1.74**.
- Dependencies limited to: `serde` (1, derive), `ron` (0.8). Dev-deps: none beyond std.
- Engine crate `smobb-core` is **pure**: no file/network/clock/stdin calls. All I/O is via `Output` values and the `Storage` trait. (Timing is emitted as `Output::Sleep`, never `thread::sleep`.)
- **Text fidelity:** port dialogue **verbatim** from `src/com/company/*.java`, applying ONLY the fixes in the audit §1 table (`docs/superpowers/specs/2026-06-27-smobb-story-audit.md`) and the approved new canon (`docs/superpowers/specs/2026-06-27-smobb-new-canon-drafts.md`). Preserve stylized spellings (`Go0_dby3`, `⬛⬛⬛⬛⬛`), all hexadecimal Jaffar lines, and all profanity.
- **Casing rule:** S.M.O.B.B., Jaffar, and system messages stay UPPERCASE; humans stay lowercase — exactly as in the source.
- Speaker → color table is the single source of truth (port of `Bits.java`); never prefix color by hand.
- Content is embedded at build time via `include_str!` (no loose runtime files).
- Every task ends green: `cargo test -p smobb-core`.
- Commit after every task. Branch off `main` first (do not commit to `main`).

## File Structure

```
Cargo.toml                      # workspace
crates/smobb-core/
  Cargo.toml
  src/lib.rs                    # re-exports; crate docs
  src/speaker.rs                # Speaker, Color, mappings (port of Bits)
  src/scene.rs                  # Step, Branch, Target, Scene, Location, Content
  src/content.rs                # include_str! registry + Content::load + lookups
  src/recap.rs                  # Recap notebook (append+dedup) + Storage trait
  src/engine.rs                 # Engine state machine: advance(), Output
  examples/play.rs              # headless stdin/stdout driver (playable)
  tests/fixtures.rs             # shared test RON fixtures
  tests/engine_flow.rs          # integration: scripted playthrough
content/
  hub.ron ali.ron bader.ron bevil.ron hasan.ron josh.ron
  qassim.ron smobb.ron extra.ron ending.ron guide.ron
```

**Standard TDD cycle** (every implementation task): write failing test → run, confirm fail → minimal impl → run, confirm pass → commit. Steps below show the actual test and impl code.

---

# PART A — Engine (`smobb-core`)

## Task 1: Workspace + crate skeleton

**Files:**
- Create: `Cargo.toml` (workspace), `crates/smobb-core/Cargo.toml`, `crates/smobb-core/src/lib.rs`

**Interfaces:**
- Produces: the `smobb-core` crate compiling under a workspace.

- [ ] **Step 1: Create workspace `Cargo.toml`**

```toml
[workspace]
members = ["crates/smobb-core"]
resolver = "2"
```

- [ ] **Step 2: Create `crates/smobb-core/Cargo.toml`**

```toml
[package]
name = "smobb-core"
version = "0.1.0"
edition = "2021"
rust-version = "1.74"

[dependencies]
serde = { version = "1", features = ["derive"] }
ron = "0.8"
```

- [ ] **Step 3: Create `crates/smobb-core/src/lib.rs`**

```rust
//! S.M.O.B.B. core engine: pure, non-blocking narrative state machine.
pub mod speaker;
pub mod scene;
pub mod content;
pub mod recap;
pub mod engine;

#[cfg(test)]
mod smoke {
    #[test]
    fn builds() {
        assert_eq!(2 + 2, 4);
    }
}
```

- [ ] **Step 4: Create empty module files so it compiles**

Create `speaker.rs`, `scene.rs`, `content.rs`, `recap.rs`, `engine.rs` each containing only `// placeholder` for now.

- [ ] **Step 5: Run**

Run: `cargo test -p smobb-core`
Expected: PASS (1 test `builds`).

- [ ] **Step 6: Commit**

```bash
git checkout -b smobb-rust-port
git add Cargo.toml crates/
git commit -m "chore: scaffold smobb-core workspace crate"
```

---

## Task 2: Speaker + Color model (port of `Bits.java`)

**Files:**
- Modify: `crates/smobb-core/src/speaker.rs`

**Interfaces:**
- Produces:
  - `enum Speaker { Ali, Bader, BaderMc, Bevil, Hasan, Josh, JoshMc, Qassim, Smobb, Jaffar, Mavis, System }` (serde, Copy, Eq)
  - `enum Color { Black, Red, Green, Yellow, Blue, Purple, Cyan, Aqua, White, Salmon, Salmon2, Gold, LightGreen, Silver, Aquamarine }` (Copy, Eq)
  - `fn Speaker::color(self) -> Color`
  - `fn Color::ansi(self) -> &'static str` (24-bit/256 escape, for terminal)
  - `fn Color::hex(self) -> &'static str` (CSS hex, for the later web frontend)

- [ ] **Step 1: Write failing tests**

```rust
// in speaker.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn speaker_colors_match_bits() {
        assert_eq!(Speaker::Ali.color(), Color::Aqua);
        assert_eq!(Speaker::Bader.color(), Color::Salmon);
        assert_eq!(Speaker::BaderMc.color(), Color::Salmon2);
        assert_eq!(Speaker::Bevil.color(), Color::Gold);
        assert_eq!(Speaker::Hasan.color(), Color::Purple);
        assert_eq!(Speaker::Josh.color(), Color::LightGreen);
        assert_eq!(Speaker::JoshMc.color(), Color::Aquamarine);
        assert_eq!(Speaker::Qassim.color(), Color::Silver);
        assert_eq!(Speaker::Smobb.color(), Color::White);
        assert_eq!(Speaker::Jaffar.color(), Color::Red);
        assert_eq!(Speaker::Mavis.color(), Color::Blue);
        assert_eq!(Speaker::System.color(), Color::White);
    }
    #[test]
    fn color_codes_present() {
        assert_eq!(Color::Aqua.ansi(), "\u{1b}[38;5;14m");
        assert_eq!(Color::White.hex(), "#ffffff");
    }
}
```

- [ ] **Step 2: Run, confirm fail**

Run: `cargo test -p smobb-core speaker`
Expected: FAIL (types not defined).

- [ ] **Step 3: Implement**

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Speaker {
    Ali, Bader, BaderMc, Bevil, Hasan, Josh, JoshMc, Qassim, Smobb, Jaffar, Mavis, System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black, Red, Green, Yellow, Blue, Purple, Cyan, Aqua, White,
    Salmon, Salmon2, Gold, LightGreen, Silver, Aquamarine,
}

impl Speaker {
    pub fn color(self) -> Color {
        match self {
            Speaker::Ali => Color::Aqua,
            Speaker::Bader => Color::Salmon,
            Speaker::BaderMc => Color::Salmon2,
            Speaker::Bevil => Color::Gold,
            Speaker::Hasan => Color::Purple,
            Speaker::Josh => Color::LightGreen,
            Speaker::JoshMc => Color::Aquamarine,
            Speaker::Qassim => Color::Silver,
            Speaker::Smobb => Color::White,
            Speaker::Jaffar => Color::Red,
            Speaker::Mavis => Color::Blue,
            Speaker::System => Color::White,
        }
    }
}

impl Color {
    /// ANSI escape (matches Bits.java codes).
    pub fn ansi(self) -> &'static str {
        match self {
            Color::Black => "\u{1b}[30m",
            Color::Red => "\u{1b}[31m",
            Color::Green => "\u{1b}[32m",
            Color::Yellow => "\u{1b}[33m",
            Color::Blue => "\u{1b}[34m",
            Color::Purple => "\u{1b}[35m",
            Color::Cyan => "\u{1b}[36m",
            Color::Aqua => "\u{1b}[38;5;14m",
            Color::White => "\u{1b}[38;5;15m",
            Color::Salmon => "\u{1b}[38;5;209m",
            Color::Salmon2 => "\u{1b}[38;5;210m",
            Color::Gold => "\u{1b}[38;5;220m",
            Color::LightGreen => "\u{1b}[38;5;119m",
            Color::Silver => "\u{1b}[38;5;7m",
            Color::Aquamarine => "\u{1b}[38;5;122m",
        }
    }
    /// CSS hex (xterm 256 approximations) for the future web frontend.
    pub fn hex(self) -> &'static str {
        match self {
            Color::Black => "#000000",
            Color::Red => "#cd0000",
            Color::Green => "#00cd00",
            Color::Yellow => "#cdcd00",
            Color::Blue => "#0000ee",
            Color::Purple => "#cd00cd",
            Color::Cyan => "#00cdcd",
            Color::Aqua => "#00ffff",
            Color::White => "#ffffff",
            Color::Salmon => "#ff875f",
            Color::Salmon2 => "#ff8787",
            Color::Gold => "#ffd700",
            Color::LightGreen => "#87ff5f",
            Color::Silver => "#c0c0c0",
            Color::Aquamarine => "#87ffd7",
        }
    }
}
```

- [ ] **Step 4: Run, confirm pass**

Run: `cargo test -p smobb-core speaker`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/speaker.rs
git commit -m "feat(core): speaker + color model ported from Bits.java"
```

---

## Task 3: Scene/Step data types + RON deserialization

**Files:**
- Modify: `crates/smobb-core/src/scene.rs`

**Interfaces:**
- Produces (all `#[derive(Debug, Clone, Deserialize)]`):
  - `enum Step { Line(Speaker, String), Pause, Recap(String), Prompt(Vec<Branch>), Goto(String), Return, Enter(String), Quit }`
  - `struct Branch { pub keys: Vec<String>, pub target: Target }`  (empty `keys` = default/else)
  - `enum Target { Scene(String), Steps(Vec<Step>) }`
  - `struct Scene { pub steps: Vec<Step> }`
  - `struct Location { pub intro: Vec<Step>, pub prompt: Vec<Step>, pub commands: Vec<Branch> }`
  - `struct Content { pub start: String, pub locations: std::collections::HashMap<String, Location>, pub scenes: std::collections::HashMap<String, Scene> }`

Semantics (consumed by Task 5+): `Enter(id)` pushes a location; `Quit` ends the program; `Return` ends the current scene; location `commands` reuse `Branch` (a `Branch` whose `keys` match the typed command).

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::speaker::Speaker;

    const SAMPLE: &str = r#"
Scene(steps: [
    Line(Ali, "hello"),
    Recap("clue one"),
    Pause,
    Prompt([
        Branch(keys: ["yes","y"], target: Scene("yes_path")),
        Branch(keys: [], target: Steps([ Line(Smobb, "DEFAULT") ])),
    ]),
    Return,
])
"#;

    #[test]
    fn parses_scene_ron() {
        let s: Scene = ron::from_str(SAMPLE).unwrap();
        assert_eq!(s.steps.len(), 5);
        match &s.steps[0] {
            Step::Line(sp, t) => { assert_eq!(*sp, Speaker::Ali); assert_eq!(t, "hello"); }
            _ => panic!("expected Line"),
        }
        match &s.steps[3] {
            Step::Prompt(bs) => {
                assert_eq!(bs[0].keys, vec!["yes", "y"]);
                assert!(bs[1].keys.is_empty());
            }
            _ => panic!("expected Prompt"),
        }
    }
}
```

- [ ] **Step 2: Run, confirm fail**

Run: `cargo test -p smobb-core scene`
Expected: FAIL.

- [ ] **Step 3: Implement**

```rust
use serde::Deserialize;
use std::collections::HashMap;
use crate::speaker::Speaker;

#[derive(Debug, Clone, Deserialize)]
pub enum Step {
    Line(Speaker, String),
    Pause,
    Recap(String),
    Prompt(Vec<Branch>),
    Goto(String),
    Return,
    Enter(String),
    Quit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Branch {
    pub keys: Vec<String>,
    pub target: Target,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Target {
    Scene(String),
    Steps(Vec<Step>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Location {
    #[serde(default)]
    pub intro: Vec<Step>,
    #[serde(default)]
    pub prompt: Vec<Step>,
    pub commands: Vec<Branch>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Content {
    pub start: String,
    pub locations: HashMap<String, Location>,
    pub scenes: HashMap<String, Scene>,
}
```

- [ ] **Step 4: Run, confirm pass** — `cargo test -p smobb-core scene` → PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/scene.rs
git commit -m "feat(core): scene/step data model with RON deserialization"
```

---

## Task 4: Content registry + embedding

**Files:**
- Modify: `crates/smobb-core/src/content.rs`
- Create: `content/_fixture.ron` (temporary tiny content used until real port lands)

**Interfaces:**
- Consumes: `scene::Content`.
- Produces: `fn content::load() -> Content` (embeds + merges all `content/*.ron`), and `fn content::from_str(parts: &[&str]) -> Content` for test injection.

Design: each `content/<name>.ron` is a `ContentPart(locations: {...}, scenes: {...})`. `load()` parses each embedded part and merges into one `Content`. `start` is fixed to `"hub"`.

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn merges_parts() {
        let a = r#"ContentPart(
            locations: { "hub": (commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ]) },
            scenes: {},
        )"#;
        let b = r#"ContentPart(
            locations: { "ali": (commands: [ Branch(keys:["end"], target: Steps([ Return ])) ]) },
            scenes: { "x": (steps: [ Return ]) },
        )"#;
        let c = from_str(&[a, b]);
        assert!(c.locations.contains_key("hub"));
        assert!(c.locations.contains_key("ali"));
        assert!(c.scenes.contains_key("x"));
        assert_eq!(c.start, "hub");
    }
}
```

- [ ] **Step 2: Run, confirm fail** — `cargo test -p smobb-core content` → FAIL.

- [ ] **Step 3: Implement**

```rust
use serde::Deserialize;
use crate::scene::{Content, Location, Scene};

#[derive(Debug, Deserialize)]
struct ContentPart {
    #[serde(default)]
    locations: std::collections::HashMap<String, Location>,
    #[serde(default)]
    scenes: std::collections::HashMap<String, Scene>,
}

pub fn from_str(parts: &[&str]) -> Content {
    let mut locations = std::collections::HashMap::new();
    let mut scenes = std::collections::HashMap::new();
    for p in parts {
        let part: ContentPart = ron::from_str(p)
            .unwrap_or_else(|e| panic!("content parse error: {e}"));
        locations.extend(part.locations);
        scenes.extend(part.scenes);
    }
    Content { start: "hub".to_string(), locations, scenes }
}

/// Embeds every content/*.ron at build time. New files MUST be added here.
pub fn load() -> Content {
    from_str(&[
        include_str!("../../../content/hub.ron"),
        include_str!("../../../content/ali.ron"),
        include_str!("../../../content/bader.ron"),
        include_str!("../../../content/bevil.ron"),
        include_str!("../../../content/hasan.ron"),
        include_str!("../../../content/josh.ron"),
        include_str!("../../../content/qassim.ron"),
        include_str!("../../../content/smobb.ron"),
        include_str!("../../../content/extra.ron"),
        include_str!("../../../content/ending.ron"),
        include_str!("../../../content/guide.ron"),
    ])
}
```

- [ ] **Step 4: Create placeholder content files so `load()` compiles**

For EACH of the 11 files listed above, create `content/<name>.ron` with:

```ron
ContentPart(locations: {}, scenes: {})
```

(These are replaced with real content in Part B. `hub.ron` must define `hub` before integration tests in Task 13 — but unit tests here use `from_str`.)

- [ ] **Step 5: Run, confirm pass** — `cargo test -p smobb-core content` → PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/smobb-core/src/content.rs content/
git commit -m "feat(core): content registry, RON merge, embed scaffolding"
```

---

## Task 5: Output type + Engine entering a location

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `Content`, `Location`, `Step`, `Speaker`, `Color`.
- Produces:
  - `enum Output { Line { text: String, color: Color }, Sleep(u32), AwaitInput, Recap(String), End }`
  - `struct Engine { /* private */ }`
  - `fn Engine::new(content: Content) -> Engine` — enters `start` location, returns nothing; first outputs come from `start()`.
  - `fn Engine::start(&mut self) -> Vec<Output>` — emits the start location intro + prompt + `AwaitInput`.

Engine internals: `content: Content`, `loc_stack: Vec<String>`, `frames: Vec<Frame>` where `struct Frame { steps: Vec<Step>, idx: usize }`, `interval_ms: u32` (default 4250, from `Bits.interval`), `skip: bool` (default false), `recap: Recap` (added Task 11; for now omit and add the field then).

- [ ] **Step 1: Write failing test** (fixture content with an intro + prompt)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::content;

    fn fx() -> crate::scene::Content {
        content::from_str(&[r#"ContentPart(
            locations: { "hub": (
                intro: [ Line(Smobb, "HELLO.") ],
                prompt: [ Line(Smobb, "HOW CAN I HELP YOU?") ],
                commands: [],
            ) },
            scenes: {},
        )"#])
    }

    #[test]
    fn start_emits_intro_then_prompt_then_await() {
        let mut e = Engine::new(fx());
        let out = e.start();
        // first line is intro, then prompt line, then AwaitInput
        assert!(matches!(out[0], Output::Line { .. }));
        assert!(matches!(out.last().unwrap(), Output::AwaitInput));
        let texts: Vec<_> = out.iter().filter_map(|o| match o {
            Output::Line { text, .. } => Some(text.clone()), _ => None
        }).collect();
        assert_eq!(texts, vec!["HELLO.", "HOW CAN I HELP YOU?"]);
    }
}
```

- [ ] **Step 2: Run, confirm fail** — `cargo test -p smobb-core engine` → FAIL.

- [ ] **Step 3: Implement** (Line/Pause/Recap handling is added in Task 6; here implement only Line + emitting prompt + AwaitInput; `interval` Sleep is added in Task 6, so this test ignores Sleep ordering by checking only Line texts + final AwaitInput)

```rust
use crate::scene::{Content, Step, Target};
use crate::speaker::Color;

#[derive(Debug, Clone, PartialEq)]
pub enum Output {
    Line { text: String, color: Color },
    Sleep(u32),
    AwaitInput,
    Recap(String),
    End,
}

struct Frame { steps: Vec<Step>, idx: usize }

pub struct Engine {
    content: Content,
    loc_stack: Vec<String>,
    frames: Vec<Frame>,
    interval_ms: u32,
    skip: bool,
}

impl Engine {
    pub fn new(content: Content) -> Self {
        let start = content.start.clone();
        Engine { content, loc_stack: vec![start], frames: Vec::new(), interval_ms: 4250, skip: false }
    }

    pub fn start(&mut self) -> Vec<Output> {
        let mut out = Vec::new();
        let loc = self.content.locations.get(self.loc_stack.last().unwrap())
            .expect("start location missing").clone();
        for s in &loc.intro { self.emit_step_lines(s, &mut out); }
        out.extend(self.menu_prompt());
        out
    }

    fn menu_prompt(&self) -> Vec<Output> {
        let mut out = Vec::new();
        let loc = self.content.locations.get(self.loc_stack.last().unwrap()).unwrap();
        for s in &loc.prompt { self.emit_step_lines(s, &mut out); }
        out.push(Output::AwaitInput);
        out
    }

    // Minimal Line-only emitter; extended in Task 6.
    fn emit_step_lines(&self, step: &Step, out: &mut Vec<Output>) {
        if let Step::Line(sp, text) = step {
            out.push(Output::Line { text: text.clone(), color: sp.color() });
        }
    }
}
```

> Note: `menu_prompt` takes `&self` but `emit_step_lines` is `&self` — fine. `start()` clones the location to avoid borrow conflicts; acceptable (locations are small).

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): Output type + engine location entry"
```

---

## Task 6: Engine scene execution — Line pacing, Recap, Return, end-of-scene

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: `fn Engine::advance(&mut self, input: &str) -> Vec<Output>`. For now handles: a location command whose target is `Steps`/`Scene` containing `Line`, `Recap`, `Return`. After a scene ends, re-emits the menu prompt + `AwaitInput`. Each `Line` is followed by `Sleep(interval_ms)` unless `skip`.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn runs_a_command_scene_then_returns_to_menu() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": (
            prompt: [ Line(Smobb, "MENU") ],
            commands: [ Branch(keys: ["go"], target: Steps([
                Line(Ali, "line a"),
                Recap("note"),
                Return,
            ])) ],
        ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("go");
    // Expect: Line "line a", Sleep(interval), Recap("note"), then menu "MENU", AwaitInput
    let kinds: Vec<&str> = out.iter().map(|o| match o {
        Output::Line{..} => "L", Output::Sleep(_) => "S",
        Output::Recap(_) => "R", Output::AwaitInput => "A", Output::End => "E",
    }).collect();
    assert_eq!(kinds, vec!["L","S","R","L","A"]);
    assert!(out.contains(&Output::Recap("note".into())));
}

#[test]
fn unknown_command_reprompts() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt: [ Line(Smobb, "MENU") ], commands: [] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("nope");
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL (`advance` not defined).

- [ ] **Step 3: Implement** — add `advance` + a `run` loop + step handling. Replace the minimal `emit_step_lines` usage by routing scene steps through `run`.

```rust
impl Engine {
    pub fn advance(&mut self, input: &str) -> Vec<Output> {
        let input = input.trim().to_lowercase();
        // If mid-scene awaiting a prompt, Task 8 handles it. For now we are at a menu.
        if self.frames.is_empty() {
            return self.handle_command(&input);
        }
        // Prompt handling added in Task 8.
        self.run()
    }

    fn handle_command(&mut self, input: &str) -> Vec<Output> {
        let loc = self.content.locations.get(self.loc_stack.last().unwrap()).unwrap().clone();
        for b in &loc.commands {
            if b.keys.iter().any(|k| k == input) {
                self.push_target(&b.target);
                let mut out = self.run();
                if self.frames.is_empty() {
                    out.extend(self.menu_prompt());
                }
                return out;
            }
        }
        // unknown command: just re-prompt
        self.menu_prompt()
    }

    fn push_target(&mut self, target: &Target) {
        match target {
            Target::Steps(steps) => self.frames.push(Frame { steps: steps.clone(), idx: 0 }),
            Target::Scene(id) => {
                let sc = self.content.scenes.get(id)
                    .unwrap_or_else(|| panic!("missing scene: {id}")).clone();
                self.frames.push(Frame { steps: sc.steps, idx: 0 });
            }
        }
    }

    /// Executes frames until a Prompt (AwaitInput, Task 8) or all frames end.
    fn run(&mut self) -> Vec<Output> {
        let mut out = Vec::new();
        while let Some(frame) = self.frames.last_mut() {
            if frame.idx >= frame.steps.len() { self.frames.pop(); continue; }
            let step = frame.steps[frame.idx].clone();
            frame.idx += 1;
            match step {
                Step::Line(sp, text) => {
                    out.push(Output::Line { text, color: sp.color() });
                    if !self.skip { out.push(Output::Sleep(self.interval_ms)); }
                }
                Step::Recap(t) => out.push(Output::Recap(t)),
                Step::Pause => self.emit_pause(&mut out),  // Task 7
                Step::Return => { self.frames.clear(); break; }
                Step::Goto(id) => self.push_target(&Target::Scene(&id_clone(&id))), // Task 7 refines
                Step::Prompt(_) => { /* Task 8 */ out.push(Output::AwaitInput); return out; }
                Step::Enter(_) => { /* Task 9 */ }
                Step::Quit => { out.push(Output::End); return out; }
            }
        }
        out
    }

    fn emit_pause(&self, _out: &mut Vec<Output>) { /* Task 7 */ }
}

fn id_clone(s: &str) -> String { s.to_string() } // temporary helper; removed in Task 7
```

> The `Goto` line above is a temporary stub that will not compile cleanly because `Target::Scene` needs an owned `String`. Fix in this step: change the `Goto` arm to:
> ```rust
> Step::Goto(id) => { let sc = self.content.scenes.get(&id).unwrap_or_else(|| panic!("missing scene: {id}")).clone(); self.frames.push(Frame{ steps: sc.steps, idx: 0 }); }
> ```
> and delete `id_clone`. (Pause stays a no-op until Task 7.)

- [ ] **Step 4: Run, confirm pass** — both tests PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): command dispatch + scene execution (Line/Recap/Return)"
```

---

## Task 7: Pause + Goto

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: `Pause` emits three `Line(".")` each followed by `Sleep(2000)` (port of `Bits.TD()`); `Goto(id)` pushes the target scene's steps as a new frame.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn pause_emits_three_dotted_sleeps() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Pause, Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("go");
    let sleeps: Vec<u32> = out.iter().filter_map(|o| match o { Output::Sleep(ms) => Some(*ms), _ => None }).collect();
    assert_eq!(sleeps, vec![2000, 2000, 2000]);
}

#[test]
fn goto_runs_target_scene() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Goto("dest") ])) ] ) },
        scenes: { "dest": ( steps: [ Line(Smobb, "ARRIVED"), Return ] ) },
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("go");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="ARRIVED")));
}
```

- [ ] **Step 2: Run, confirm fail** — `pause_emits...` FAILS (Pause no-op).

- [ ] **Step 3: Implement**

```rust
fn emit_pause(&self, out: &mut Vec<Output>) {
    for _ in 0..3 {
        out.push(Output::Line { text: ".".into(), color: crate::speaker::Color::White });
        out.push(Output::Sleep(2000));
    }
}
```

(Confirm the `Goto` arm from Task 6 is the corrected version that pushes the scene frame.)

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): Pause (TD) + Goto scene bridging"
```

---

## Task 8: Prompt branching (+ default) and nested prompts

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: when `run()` hits `Step::Prompt(branches)`, it emits `AwaitInput` and suspends. The NEXT `advance(input)` matches `input` (lowercased) against each branch's `keys`; a branch with empty `keys` is the default. The matched target is pushed as a frame and execution resumes. If no branch matches and there is no default, the prompt re-awaits.

Add engine field: `pending_prompt: Option<Vec<Branch>>`.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn prompt_branches_match_and_default() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Prompt([
                Branch(keys:["yes","y"], target: Steps([ Line(Smobb,"YESPATH"), Return ])),
                Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])),
            ]),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("go");
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    let out = e.advance("y");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="YESPATH")));

    // re-run, take default
    let mut e2 = Engine::new(content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Prompt([ Branch(keys:["yes"], target: Steps([ Line(Smobb,"Y"), Return ])),
                     Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])) ]),
        ])) ] ) }, scenes: {},
    )"#]));
    let _ = e2.start();
    let _ = e2.advance("go");
    let out = e2.advance("whatever");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="DEFAULT")));
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL.

- [ ] **Step 3: Implement** — add field, set it on Prompt, consume it in `advance`.

In `Engine` struct add: `pending_prompt: Option<Vec<crate::scene::Branch>>,` and initialise `None` in `new`.

Replace the `Step::Prompt` arm in `run`:

```rust
Step::Prompt(branches) => {
    self.pending_prompt = Some(branches);
    out.push(Output::AwaitInput);
    return out;
}
```

Update `advance`:

```rust
pub fn advance(&mut self, input: &str) -> Vec<Output> {
    let input = input.trim().to_lowercase();
    if let Some(branches) = self.pending_prompt.take() {
        let chosen = branches.iter()
            .find(|b| b.keys.iter().any(|k| k.to_lowercase() == input))
            .or_else(|| branches.iter().find(|b| b.keys.is_empty()));
        match chosen {
            Some(b) => {
                let t = b.target.clone();
                self.push_target(&t);
                let mut out = self.run();
                if self.frames.is_empty() { out.extend(self.menu_prompt()); }
                return out;
            }
            None => {
                // no match, no default: re-await same prompt
                self.pending_prompt = Some(branches);
                return vec![Output::AwaitInput];
            }
        }
    }
    if self.frames.is_empty() {
        return self.handle_command(&input);
    }
    self.run()
}
```

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): prompt branching with default + nested support"
```

---

## Task 9: Location navigation (`Enter`) + location stack + `end`/`Quit`

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: `Step::Enter(id)` pushes a new location onto `loc_stack`, emits its `intro` + menu prompt. A command resolving to `Return` at a non-root location pops back to the parent location (emitting that parent's prompt). `Step::Quit` (and `Return` at the root) emits `Output::End`.

Convention from the port: every location's `commands` include an `end` branch → `Steps([ Return ])`. `Return` pops one location; at root it ends the program.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn enter_and_exit_locations() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: {
            "hub": ( prompt:[Line(Smobb,"HUB")], commands: [
                Branch(keys:["ali"], target: Steps([ Enter("ali") ])),
                Branch(keys:["end"], target: Steps([ Quit ])),
            ]),
            "ali": ( intro:[Line(Ali,"hi")], prompt:[Line(Ali,"ALIMENU")], commands: [
                Branch(keys:["end"], target: Steps([ Return ])),
            ]),
        },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("ali");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="hi")));
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="ALIMENU")));
    let out = e.advance("end"); // pop back to hub
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="HUB")));
    let out = e.advance("end"); // quit at hub
    assert!(out.contains(&Output::End));
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL.

- [ ] **Step 3: Implement** — handle `Enter` and root-aware `Return` in `run`.

```rust
Step::Enter(id) => {
    self.frames.clear();
    self.loc_stack.push(id.clone());
    let loc = self.content.locations.get(&id)
        .unwrap_or_else(|| panic!("missing location: {id}")).clone();
    for s in &loc.intro {
        if let Step::Line(sp, text) = s {
            out.push(Output::Line { text: text.clone(), color: sp.color() });
            if !self.skip { out.push(Output::Sleep(self.interval_ms)); }
        }
    }
    out.extend(self.menu_prompt());
    return out;
}
Step::Return => {
    self.frames.clear();
    if self.loc_stack.len() > 1 {
        self.loc_stack.pop();
        out.extend(self.menu_prompt());
    } else {
        out.push(Output::End);
    }
    return out;
}
Step::Quit => { out.push(Output::End); return out; }
```

(Remove the earlier simple `Return`/`Quit` arms from Task 6 — these replace them. `Enter` previously was a no-op.)

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): location stack navigation (Enter/Return/Quit)"
```

---

## Task 10: Settings commands (interval / skipmode)

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: two engine methods callable by the host/built-ins:
  - `fn Engine::set_skip(&mut self, on: bool)`
  - `fn Engine::set_interval(&mut self, ms: u32)`
  - `fn Engine::skip(&self) -> bool` and `fn Engine::interval(&self) -> u32` (getters for the driver to display).

These back the `skipmode`/`interval` hub commands. (The hub command wiring that calls them lives in the headless driver — Task 13 — and later in the real frontends, since interval needs a numeric sub-prompt.)

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn skip_suppresses_interval_sleeps() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"a"), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("go");
    assert!(!out.iter().any(|o| matches!(o, Output::Sleep(_))));
}

#[test]
fn interval_value_used() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Line(Ali,"a"), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_interval(100);
    let _ = e.start();
    let out = e.advance("go");
    assert!(out.contains(&Output::Sleep(100)));
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL.

- [ ] **Step 3: Implement**

```rust
impl Engine {
    pub fn set_skip(&mut self, on: bool) { self.skip = on; }
    pub fn set_interval(&mut self, ms: u32) { self.interval_ms = ms; }
    pub fn skip(&self) -> bool { self.skip }
    pub fn interval(&self) -> u32 { self.interval_ms }
}
```

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): interval/skipmode settings"
```

---

## Task 11: Recap notebook (append + dedup) + Storage trait

**Files:**
- Modify: `crates/smobb-core/src/recap.rs`, `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces:
  - `trait Storage { fn read(&self) -> Vec<String>; fn write(&mut self, lines: &[String]); }`
  - `struct MemoryStorage { lines: Vec<String> }` impl `Storage` + `Default`.
  - `struct Recap { entries: Vec<String> }` with `fn add(&mut self, line: &str) -> bool` (returns false if duplicate — fixes the Java truncate/dedup bug), `fn entries(&self) -> &[String]`, `fn load(storage: &dyn Storage) -> Recap`, `fn persist(&self, storage: &mut dyn Storage)`.
- Wire `Output::Recap(t)` handling: the engine adds to its internal `Recap` (dedup) AND still emits `Output::Recap(t)` so the driver can persist. Add `fn Engine::recap_entries(&self) -> &[String]`.

- [ ] **Step 1: Write failing tests** (recap.rs)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dedups() {
        let mut r = Recap::default();
        assert!(r.add("a"));
        assert!(r.add("b"));
        assert!(!r.add("a")); // duplicate
        assert_eq!(r.entries(), &["a".to_string(), "b".to_string()]);
    }
    #[test]
    fn round_trips_through_storage() {
        let mut s = MemoryStorage::default();
        let mut r = Recap::default();
        r.add("x"); r.add("y");
        r.persist(&mut s);
        let r2 = Recap::load(&s);
        assert_eq!(r2.entries(), &["x".to_string(), "y".to_string()]);
    }
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL.

- [ ] **Step 3: Implement** (recap.rs)

```rust
pub trait Storage {
    fn read(&self) -> Vec<String>;
    fn write(&mut self, lines: &[String]);
}

#[derive(Default)]
pub struct MemoryStorage { pub lines: Vec<String> }
impl Storage for MemoryStorage {
    fn read(&self) -> Vec<String> { self.lines.clone() }
    fn write(&mut self, lines: &[String]) { self.lines = lines.to_vec(); }
}

#[derive(Default)]
pub struct Recap { entries: Vec<String> }
impl Recap {
    pub fn add(&mut self, line: &str) -> bool {
        if self.entries.iter().any(|e| e == line) { return false; }
        self.entries.push(line.to_string());
        true
    }
    pub fn entries(&self) -> &[String] { &self.entries }
    pub fn load(storage: &dyn Storage) -> Recap {
        let mut r = Recap::default();
        for l in storage.read() { r.add(&l); }
        r
    }
    pub fn persist(&self, storage: &mut dyn Storage) { storage.write(&self.entries); }
}
```

- [ ] **Step 4: Wire engine** — in `Engine` add `recap: crate::recap::Recap` (init `Recap::default()` in `new`), update the `Recap` arm in `run`:

```rust
Step::Recap(t) => { self.recap.add(&t); out.push(Output::Recap(t)); }
```

add getter:

```rust
impl Engine { pub fn recap_entries(&self) -> &[String] { self.recap.entries() } }
```

- [ ] **Step 5: Add engine test**

```rust
#[test]
fn engine_collects_recap_dedup() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Recap("clue"), Recap("clue"), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("go");
    assert_eq!(e.recap_entries(), &["clue".to_string()]);
}
```

- [ ] **Step 6: Run, confirm pass** — `cargo test -p smobb-core` → PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/smobb-core/src/recap.rs crates/smobb-core/src/engine.rs crates/smobb-core/src/lib.rs
git commit -m "feat(core): recap notebook with dedup + storage trait"
```

---

## Task 12: Built-in commands data — guide/info/shortcuts/recap-view

**Files:**
- Modify: `content/hub.ron` (later, Part B), `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: a single special target the driver can render: `fn Engine::recap_view(&self) -> Vec<Output>` returns each recap entry as an `Output::Line { color: White }`, or one line `"(empty)"` if none. (Guide/info/shortcuts are plain content scenes authored in Part B — no engine code needed; they are just `Steps([...Line...])`.)

This task exists so the recap viewer is engine-backed (the data lives in the engine, not content).

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn recap_view_lists_entries() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([ Recap("one"), Return ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("go");
    let view = e.recap_view();
    assert!(view.iter().any(|o| matches!(o, Output::Line{text,..} if text=="one")));
}

#[test]
fn recap_view_empty() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(locations:{"hub":(commands:[])},scenes:{})"#]);
    let e = Engine::new(c);
    let view = e.recap_view();
    assert!(view.iter().any(|o| matches!(o, Output::Line{text,..} if text=="(empty)")));
}
```

- [ ] **Step 2: Run, confirm fail** — FAIL.

- [ ] **Step 3: Implement**

```rust
impl Engine {
    pub fn recap_view(&self) -> Vec<Output> {
        let c = crate::speaker::Color::White;
        if self.recap.entries().is_empty() {
            return vec![Output::Line { text: "(empty)".into(), color: c }];
        }
        self.recap.entries().iter()
            .map(|t| Output::Line { text: t.clone(), color: c })
            .collect()
    }
}
```

- [ ] **Step 4: Run, confirm pass** — PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): engine-backed recap viewer"
```

---

## Task 13: Headless driver example + scripted integration test

**Files:**
- Create: `crates/smobb-core/examples/play.rs`
- Create: `crates/smobb-core/tests/engine_flow.rs`

**Interfaces:**
- Consumes: `Engine`, `Output`, `content::load()`.
- Produces: a runnable headless game (`cargo run -p smobb-core --example play`) and an integration test that drives the engine with scripted input and asserts an `End` is reachable. The example renders `Output::Line` with `color.ansi()`, treats `Sleep(ms)` with `std::thread::sleep`, and on `AwaitInput` reads a line from stdin. Built-in hub commands `skipmode`, `recap` are handled in the driver (calling `Engine::set_skip`, `Engine::recap_view`).

- [ ] **Step 1: Write the example driver**

```rust
use smobb_core::content;
use smobb_core::engine::{Engine, Output};
use std::io::{self, Write, BufRead};
use std::{thread, time::Duration};

fn render(out: &[Output], e: &Engine) -> bool {
    for o in out {
        match o {
            Output::Line { text, color } => println!("{}{}\u{1b}[0m", color.ansi(), text),
            Output::Sleep(ms) => thread::sleep(Duration::from_millis(*ms as u64)),
            Output::Recap(_) => {}
            Output::AwaitInput => { print!("> "); io::stdout().flush().unwrap(); }
            Output::End => return true,
        }
    }
    let _ = e; false
}

fn main() {
    let mut e = Engine::new(content::load());
    if render(&e.start(), &e) { return; }
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let input = line.unwrap_or_default();
        let trimmed = input.trim().to_lowercase();
        let out = match trimmed.as_str() {
            "skipmode" | "sm" => { e.set_skip(!e.skip()); vec![] }
            "recap" | "rp" => e.recap_view(),
            _ => e.advance(&input),
        };
        if render(&out, &e) { break; }
    }
}
```

- [ ] **Step 2: Write the integration test**

```rust
use smobb_core::content;
use smobb_core::engine::{Engine, Output};

#[test]
fn fixture_game_reaches_end() {
    // minimal self-contained content with a path to Quit
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": (
            intro: [ Line(Smobb, "HELLO.") ],
            prompt: [ Line(Smobb, "HOW CAN I HELP YOU?") ],
            commands: [
                Branch(keys: ["smobb"], target: Steps([ Enter("smobb_loc") ])),
                Branch(keys: ["end"], target: Steps([ Quit ])),
            ],
        ),
        "smobb_loc": (
            prompt: [ Line(Smobb, "WHAT WOULD YOU LIKE TO KNOW?") ],
            commands: [
                Branch(keys: ["ali"], target: Steps([ Line(Smobb, "INCOMPETENT."), Return ])),
                Branch(keys: ["end"], target: Steps([ Return ])),
            ],
        ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("smobb");
    let out = e.advance("ali");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text=="INCOMPETENT.")));
    let _ = e.advance("end"); // back to hub
    let out = e.advance("end"); // quit
    assert!(out.contains(&Output::End));
}
```

- [ ] **Step 3: Run** — `cargo test -p smobb-core` PASS; `cargo build --example play -p smobb-core` compiles. (Running `play` needs real `content/` — works after Part B; with placeholder content `hub` is empty, so `start()` panics. Acceptable until Part B; the integration test does not use `content::load()`.)

- [ ] **Step 4: Commit**

```bash
git add crates/smobb-core/examples/play.rs crates/smobb-core/tests/engine_flow.rs
git commit -m "feat(core): headless driver example + scripted integration test"
```

---

# PART B — Content Port (`content/*.ron`)

**How to port each file:** open the named Java source. For each `switch` case, write a `Branch` whose `keys` are the case labels (lowercased) and whose `target` is `Steps([...])` (or `Scene("id")` when shared). Each `C.PS("...")` / `System.out.println(...)` line becomes `Line(<Speaker>, "...")`; `C.TD()`/`C.WTD()` becomes `Pause`; `recap.write("...")` becomes `Recap("...")`; nested `kb.next()` + `if/switch` becomes `Prompt([...])`. Apply the audit §1 fixes and the approved canon. Speaker is chosen from the color the line used (the `Bits` comment table) — e.g. a `C.AQUA` line is `Line(Ali, ...)`, `C.SALMON2` is `Line(BaderMc, ...)`, red/hex is `Line(Jaffar, ...)`, white system text is `Line(System, ...)`.

**Per-file test pattern** (each content task adds one test to `crates/smobb-core/tests/content_<name>.rs`):

```rust
use smobb_core::content;
#[test]
fn <name>_loads_and_has_known_line() {
    let c = content::load();              // parses ALL embedded files; fails loudly on RON error
    // assert a representative scene/line exists for this file
}
```

> Because `content::load()` parses every file, each content task's test guards the whole set. Add a focused assertion per file (a known command key or recap string).

---

## Task 14: Port `hub.ron` (from `Main.java`)

**Files:** Create real `content/hub.ron` (replace placeholder); Test: `crates/smobb-core/tests/content_hub.rs`

Source: `src/com/company/Main.java`. Define location `hub` with `intro` = `[ Pause, Line(System,"HELLO. THANK YOU FOR USING S.M.O.B.B.") ]`, `prompt` = `[ Line(System,"HOW CAN I HELP YOU? IF CONFUSED, TYPE 'HELP'.") ]`, and `commands` for every case: `balls/ball`, `shortcut(s)`, `guide/help` → `Enter("guide")`, `recap/rp` (handled by driver/built-in — still add a `Branch` → `Steps([])` no-op so it does not "unknown"), `skipmode/sm` & `interval/in` (driver built-ins — add no-op branches), `play/pl` → guide text scene, `hi/hey`, `jaffar` (hex lines, `Line(Jaffar,...)` + `Pause`), `info/information`, character names → `Enter("ali")` … `Enter("smobb")`, `9285005`, `extra(s)` → `Enter("extra")`, `end7312` → `Goto("ending")`, `!save`, `!break`, `mavis/m.a.v.i.s/m.a.v.i.s.`, `end` → `Steps([ Quit ])`.

Apply fixes: none in Main except the general casing. Keep hex verbatim.

- [ ] **Step 1: Write `content/hub.ron`** transcribing all `Main.java` cases per the rules above. (Use `Line(System, ...)` for the white `C.WHITE`/`System.out` lines, `Line(Jaffar, ...)` for the red hex blocks.)
- [ ] **Step 2: Write test**

```rust
use smobb_core::content;
#[test]
fn hub_loads() {
    let c = content::load();
    let hub = c.locations.get("hub").expect("hub location");
    assert!(hub.commands.iter().any(|b| b.keys.iter().any(|k| k=="mavis")));
    assert!(hub.commands.iter().any(|b| b.keys.iter().any(|k| k=="end")));
}
```

- [ ] **Step 3: Run** — `cargo test -p smobb-core content_hub` → PASS.
- [ ] **Step 4: Commit** — `git add content/hub.ron crates/smobb-core/tests/content_hub.rs && git commit -m "content: port hub (Main.java) to RON"`

---

## Task 15: Port `ali.ron` (from `Ali.java`) + new `quint9440` + master-control split + mavis signpost

**Files:** Create `content/ali.ron`; Test: `crates/smobb-core/tests/content_ali.rs`

Source: `src/com/company/Ali.java`. Location `ali`: `intro` = the four opening AQUA lines (after a `Pause`), `prompt` = `[ Line(System, "ENTER THE PASSWORD:") ]`. Port every case: `!save`, `k1ll`, `qassim`, `mavis`, `jaffar`, `end7312`, `!help`, `brain`, `7549009`, `9285005`, `josh`, `bader` (→ `BaderAli` lines), `bevil`, `hasan`, `plan`, `end` → `Return`.

Apply fixes (audit §1): `runied→ruined` (was Ali:63), `mutliple→multiple` (246), `completedly→completely` (155), `downard→downward` (129), `decided take action→decided to take action` (59), `I HIGH HOPES→I HAVE HIGH HOPES` (285).

Refactor for canon #3: extract the existing `7549009` body from `"MASTER CONTROL RECEIVED."` onward into a shared `scenes: { "ali_master_control": (...) }`. The `7549009` command's `Steps` end by leading into it (the lines before MASTER CONTROL stay inline, then `Goto("ali_master_control")`). Add the NEW command:

```ron
Branch(keys: ["quint9440"], target: Steps([
    Line(Ali, "Username's set. 'Quint9440'. He won't figure out it's me."),
    Line(Ali, "Alright. The Tranava files. Let's fucking end this."),
    Line(Ali, "...There's a guard? Bader hired someone to sit on a folder? Seriously?"),
    Line(Josh, "Hold it right there, big guy! Boss says nobody gets past me!"),
    Line(Ali, "I don't have time for this. Move."),
    Line(Ali, "...He just let me through. The hell?"),
    Line(Ali, "Of course. Bader WANTED me in here. This was the plan the whole time."),
    Line(Ali, "Fine. I'm already in. And I've got the password now: 7549009."),
    Line(Ali, "Bader, you absolute child. You used your own house address as the password."),
    Line(Ali, "No more waiting. I'm taking master control."),
    Goto("ali_master_control"),
])),
```

Canon #1 (mavis signpost): at the END of the `!help` case (the husk reveal), before its final `Pause`, add:

```ron
Recap("He called me M.A.V.I.S. — that's what I am? I should ask the others about 'mavis'."),
```

The nested `7549009` yes/no and name-check become `Prompt`s (keys lowercased; the long yes/no lists collapse to e.g. `keys: ["yes","y","yeah","yep","yup","yessir"]` and `keys: ["no","n","nope","nah","nuhhuh"]`; the killer/murderer and jaffar name checks become inner `Prompt`s with a default branch).

- [ ] **Step 1: Write `content/ali.ron`** (full transcription + fixes + canon above).
- [ ] **Step 2: Write test**

```rust
use smobb_core::content;
#[test]
fn ali_loads_with_canon() {
    let c = content::load();
    assert!(c.scenes.contains_key("ali_master_control"));
    let ali = c.locations.get("ali").unwrap();
    assert!(ali.commands.iter().any(|b| b.keys.iter().any(|k| k=="quint9440")));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Ali.java + quint9440 scene + mavis signpost`

---

## Task 16: Port `bader.ron` (from `Bader.java`)

**Files:** Create `content/bader.ron`; Test: `content_bader.rs`

Source: `Bader.java`. Location `bader`: `intro` = `[ Line(System,"WELCOME TO THE TRANAVA FILES.") ]`, `prompt` = `[ Line(System,"WHAT ACTION WOULD YOU LIKE TO START?") ]`. Cases: `!save`, `7549009`, `9285005`, `k1ll` (uses `BaderMc` for the SALMON2 line "I killed him..."), `delete`, `josh` (→ `BaderJosh` lines, see Task 19 — inline the Bader-side lines here, the Josh-side lines belong to josh content; keep as authored: this case is Bader talking, so `Line(Josh,...)`/`Line(Bader,...)` per original `BaderJosh` which is actually in Josh.java — port `BaderJosh` in Task 19 and have this case `Goto("bader_josh")`), `games/game` (Prompt with default), `ali` (→ `AliBader` lines), `hack`, `hasan`, `tranavaloop`, `bevil`, `qassim`, `end`→`Return`, `jaffar/ja3ffar/ja3ffer` (Prompt for the breach `<---------->`, hex default).

Apply fix: `beacuse→because` (Bader:180). Flagged-but-not-fixed (leave verbatim, add a `// FLAG` RON comment): `chibdi` (106), the "no games" gag (116).

- [ ] **Step 1: Write `content/bader.ron`.**
- [ ] **Step 2: Test**

```rust
#[test]
fn bader_loads() {
    let c = smobb_core::content::load();
    let b = c.locations.get("bader").unwrap();
    assert!(b.commands.iter().any(|x| x.keys.iter().any(|k| k=="tranavaloop")));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Bader.java to RON`

---

## Task 17: Port `bevil.ron` (from `Bevil.java`)

**Files:** Create `content/bevil.ron`; Test: `content_bevil.rs`

Source: `Bevil.java`. Location `bevil`: `prompt` = `[ Line(Bevil,"Hey I'm about to take a shower, what do you need?") ]`. Cases: `!save`, `7789980` (the big farewell + MORP backstory; Pause at start/end), `7549009`, `9285005`, `jaffar/ja3ffar/ja3ffer` (includes the converter URL recap), `tranavaloop`, `josh`, `breach` (Prompt `<---------->`, hex default; recap the breach code), `ali`, `hasan` (recap password; Prompt breach), `bader`, `end`→`Return`.

Apply fixes: `tell li→tell Ali` (169), `as many help→as much help` (159), strip stray `]` in `]That's all though` (141).

- [ ] **Step 1: Write `content/bevil.ron`.**
- [ ] **Step 2: Test** — assert `bevil` has command key `7789980`.
- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Bevil.java to RON`

---

## Task 18: Port `hasan.ron` (from `Hasan.java`) + cutoff fix

**Files:** Create `content/hasan.ron`; Test: `content_hasan.rs`

Source: `Hasan.java`. Location `hasan`: `prompt` = `[ Line(Hasan,"Hello? How may I help you?") ]`. Cases: `!save`, `7549009`, `9285005`, `k1ll`, `smobb`, `ja3ffer/ja3ffar/jaffar` (nested double breach `Prompt`s, hex defaults), `ali` (recap password), `bader`, `josh`, `qassim`, `mavis/m.a.v.i.s/m.a.v.i.s.`, `bevil` (recap breach code; Prompt), `end`→`Return`.

Apply fixes: `com.company.Bader→Bader` (225); **cutoff** at Hasan:89 `"I'm sorry that ."` → `"I'm sorry that it had to come to this."` (approved canon #6).

- [ ] **Step 1: Write `content/hasan.ron`.**
- [ ] **Step 2: Test**

```rust
#[test]
fn hasan_cutoff_filled_and_no_pkg_leak() {
    let raw = include_str!("../../../content/hasan.ron");
    assert!(raw.contains("I'm sorry that it had to come to this."));
    assert!(!raw.contains("com.company"));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Hasan.java + fill cutoff + strip pkg leak`

---

## Task 19: Port `josh.ron` (from `Josh.java`) + `bader_josh` scene + new `josh_fate`

**Files:** Create `content/josh.ron`; Test: `content_josh.rs`

Source: `Josh.java`. Location `josh`: `intro` = `[ Line(Josh,"Hmm? Who might you be?") ]`, `prompt` = `[ Line(Josh,"Well, what do you really want?") ]`. Cases: `7549009` (ends discovering S.M.O.B.B.; append a `Goto("josh_fate")` per canon #2), `!save`, `9285005` (JoshMc aquamarine line; recap K1LL clue), `k1ll` (Jaffar hex meltdown; recap `end7312` clue), `bomb/explosion`, `jaffar`, `ali`, `bader`, `bevil`, `qassim`, `tranava/tranavafiles`, `hasan` (Prompt breach), `end`→`Return`.

Add shared scene `bader_josh` (port of `BaderJosh()` — Bader↔Josh, includes the Prompt breach + recap of `7549009`). Add NEW scene `josh_fate` (canon #2 draft — use `JoshMc` for the mind-controlled lines, `Line(Jaffar, "5448452054455354...")` hex line as drafted, `Line(Smobb,...)` for the closing).

Apply fix: `mean much to be but→mean much to me but` (Josh:33).

- [ ] **Step 1: Write `content/josh.ron`** (location + `bader_josh` + `josh_fate`).
- [ ] **Step 2: Test**

```rust
#[test]
fn josh_fate_exists() {
    let c = smobb_core::content::load();
    assert!(c.scenes.contains_key("josh_fate"));
    assert!(c.scenes.contains_key("bader_josh"));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Josh.java + bader_josh + josh_fate canon`

---

## Task 20: Port `qassim.ron` (from `Qassim.java`)

**Files:** Create `content/qassim.ron`; Test: `content_qassim.rs`

Source: `Qassim.java`. Location `qassim`: `prompt` = `[ Line(Qassim,"...") ]`. Cases: `7549009` (death recording; recap `9285005`; Pause; ends mid-murder), `k1ll`, `9285005` (the SMOBB-naming scene with the anime acronym lines), `ali`, `bader`, `brain`, `smobb`, `josh`, `bomb/explosion` (long pre-death monologue, Pause), `jaffar`, `end`→`Return`. No §1 text fixes in this file.

- [ ] **Step 1: Write `content/qassim.ron`.**
- [ ] **Step 2: Test** — assert `qassim` has key `7549009` and a `Recap` containing `9285005` appears in the raw file.
- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Qassim.java to RON`

---

## Task 21: Port `smobb.ron` (from `Smobb.java`)

**Files:** Create `content/smobb.ron`; Test: `content_smobb.rs`

Source: `Smobb.java`. Location `smobb`: `intro` = `[ Line(System,"ME? INTRIGUING."), Pause ]` (the 3000ms sleep → `Pause`), `prompt` = `[ Line(System,"WHAT WOULD YOU LIKE TO KNOW?") ]`. Cases: `end`→`Return`, `ali`, `9285005`, `bader`, `bevil`, `hasan`, `qassim`, `josh`, `mavis`, `!save`, `recap`, `end7312`, `7549009`, `jaffar`. All S.M.O.B.B. lines are `Line(Smobb, ...)` (white). No §1 fixes.

- [ ] **Step 1: Write `content/smobb.ron`.**
- [ ] **Step 2: Test** — assert `smobb` has key `mavis`.
- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Smobb.java to RON`

---

## Task 22: Port `extra.ron` (from `Extra.java`) + `parable_map` scene

**Files:** Create `content/extra.ron`; Test: `content_extra.rs`

Source: `Extra.java`. Location `extra`: `prompt` = `[ Line(System,"WHAT WOULD YOU LIKE TO SEARCH FOR?"), Line(System,"OPTIONS ARE: STORIES/END.") ]`. Command `story/stories` → a `Prompt` that selects parables 1–12 (keys `["1"]`…`["12"]`), each a `Steps([ Pause, Line(System, "..."), ..., Pause ])` transcribing the 12 stories verbatim. `end`→`Return`.

Add NEW scene `parable_map` (canon #4 reveal). Wire its reachability in Task 23/ending or as a `smobb` command — for now author the scene; expose it via a new `extra` command key `mapping` that is only meaningful post-ending (no gating logic required; it is content the player can find). Use the approved table; mark low-confidence entries with a RON `// ?` comment but keep the line text committed.

- [ ] **Step 1: Write `content/extra.ron`** (12 stories + `parable_map`).
- [ ] **Step 2: Test**

```rust
#[test]
fn extra_has_twelve_and_map() {
    let raw = include_str!("../../../content/extra.ron");
    let c = smobb_core::content::load();
    assert!(c.scenes.contains_key("parable_map"));
    // crude check that all 12 story numbers are present as prompt keys
    for n in 1..=12 { assert!(raw.contains(&format!("\"{n}\""))); }
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Extra.java 12 parables + parable_map canon`

---

## Task 23: Port `ending.ron` (from `Ending.java`) + Hameed fix

**Files:** Create `content/ending.ron`; Test: `content_ending.rs`

Source: `Ending.java`. Scene `ending` (reached via `end7312` from hub/smobb): the full final monologue as `Line(System, ...)` / `Line(Smobb, ...)` (all white), `Pause` for `TD()`, then a `Prompt`: keys `["save","try"]` → the save path (`Recap("I can save them. I will make the ability now. It is '!SAVE'. I will warn everyone before it happens.")` + closing lines); default branch → the die path (same recap per audit §2.F flag — keep both as-is) + "WELCOME TO HELL." + `Pause`.

Apply fix (canon §2.A): `QASSIM HUJAIR→QASSIM HAMEED` (Ending:20).

- [ ] **Step 1: Write `content/ending.ron`.**
- [ ] **Step 2: Test**

```rust
#[test]
fn ending_uses_hameed() {
    let raw = include_str!("../../../content/ending.ron");
    assert!(raw.contains("QASSIM HAMEED"));
    assert!(!raw.contains("QASSIM HUJAIR"));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Ending.java + Hameed continuity fix`

---

## Task 24: Port `guide.ron` (from `Guide.java`) — Swing dropped

**Files:** Create `content/guide.ron`; Test: `content_guide.rs`

Source: `Guide.java`. Location `guide`: `prompt` = `[ Line(System,"YOUR OPTIONS ARE: NAMES/GAME/SETTINGS/RECAP/END.") ]`. Commands: `names/name` → `Enter("guide_names")` (or a `Prompt` of the character bios — port the NameGuide bios verbatim, with `[REDATCTED]→[REDACTED]` fix at Guide:82, and Qassim bio surname → `HAMEED` to match canon), `game/play` → a TEXT how-to-play scene (REPLACE the Swing PNG popups with in-terminal instructions — write a concise 6–8 line how-to derived from the README + game rules: one-word answers, passwords via recap, hex converter, settings), `settings` → text explaining interval/skipmode (driver-handled), `recap` → instruct the player to type `recap` at the hub, `end`→`Return`.

This task fulfills the design's "guide → in-terminal text, PNGs dropped" decision. `guide1-3.png` are NOT referenced.

- [ ] **Step 1: Write `content/guide.ron`** (names bios + text how-to-play + settings text + recap help).
- [ ] **Step 2: Test**

```rust
#[test]
fn guide_has_no_swing_and_redacted_fixed() {
    let raw = include_str!("../../../content/guide.ron");
    assert!(!raw.contains(".png"));
    assert!(!raw.contains("REDATCTED"));
    assert!(raw.contains("HAMEED"));
}
```

- [ ] **Step 3: Run** PASS. **Step 4: Commit** `content: port Guide.java to in-terminal text (drop Swing PNGs)`

---

## Task 25: Full playthrough integration test + README + cleanup

**Files:**
- Create: `crates/smobb-core/tests/playthrough.rs`
- Modify: `README.md`

**Interfaces:** Consumes `content::load()` and `Engine`.

- [ ] **Step 1: Write a real-content playthrough test** that drives the embedded content along the verified password chain and asserts no panic + reaching the ending.

```rust
use smobb_core::content;
use smobb_core::engine::{Engine, Output};

#[test]
fn real_content_password_chain_runs() {
    let mut e = Engine::new(content::load());
    let _ = e.start();
    // hub -> hasan -> ali (learn 7549009)
    let _ = e.advance("hasan");
    let _ = e.advance("ali");
    let _ = e.advance("end");
    // hub -> qassim -> 7549009 (learn 9285005)
    let _ = e.advance("qassim");
    let _ = e.advance("7549009");
    let _ = e.advance("end");
    // hub -> josh -> 9285005 (learn K1LL) -> k1ll (learn end7312)
    let _ = e.advance("josh");
    let _ = e.advance("9285005");
    let _ = e.advance("k1ll");
    let _ = e.advance("end");
    // hub -> end7312 (the ending), choose save
    let _ = e.advance("end7312");
    let out = e.advance("save");
    // recap should now contain the !SAVE unlock line
    assert!(e.recap_entries().iter().any(|r| r.contains("!SAVE")));
    let _ = out;
}
```

- [ ] **Step 2: Run** — `cargo test -p smobb-core` (all unit + integration) → PASS. Fix any RON parse errors surfaced by `content::load()` here.
- [ ] **Step 3: Manual smoke** — `cargo run -p smobb-core --example play`, walk a few commands, confirm colors + pacing + recap (`recap` command) work.
- [ ] **Step 4: Update `README.md`** — add a "Build & run (Rust)" section: `cargo run -p smobb-core --example play`; note the engine is the basis for the upcoming TUI + web frontends; note the Java version is preserved under `src/` as the original artifact.
- [ ] **Step 5: Commit**

```bash
git add crates/smobb-core/tests/playthrough.rs README.md
git commit -m "test(core): full real-content playthrough; docs: Rust build instructions"
```

---

## Self-Review (completed against the spec)

- **Spec coverage:** Java→Rust engine (Tasks 1–13) ✓; separation of text→data (Tasks 3,4,14–24) ✓; recap fix (Task 11) ✓; guide→text, Swing dropped (Task 24) ✓; settings preserved (Task 10) ✓; audit §1 fixes mapped to Tasks 15–24 ✓; approved canon — mavis signpost (15), josh_fate (19), quint9440 (15), parable_map (22), Hameed (23) ✓. Web/TUI frontends are explicitly out of scope (follow-on plans).
- **Placeholder scan:** none — every step has concrete code or an exact source-file + fix mapping. (Content tasks reference real, existing source text plus approved drafts.)
- **Type consistency:** `Engine`, `Output`, `Step`, `Branch`, `Target`, `Content`, `Recap`, `Storage` names and signatures are consistent across tasks; `set_skip/set_interval/skip/interval/recap_entries/recap_view/start/advance/new` used consistently by the driver (Task 13) and tests.

## Follow-on plans (not in this plan)
- Phase 3: `smobb-tui` (ratatui) consuming `smobb-core`.
- Phase 4: `smobb-wasm` + `web/` (xterm.js + PWA), reusing `Color::hex()` + the `Output` stream + a `localStorage` `Storage` impl.
- Phase 5: packaging (desktop binaries, web deploy).
