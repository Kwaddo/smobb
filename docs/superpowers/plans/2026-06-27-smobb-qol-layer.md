# S.M.O.B.B. QoL Layer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three engine-side quality-of-life features to `smobb-core` — inline hex→English decoding of Jaffar's lines, a discovered-options helper, and a structured journal — with ZERO content-file changes.

**Architecture:** All features live in the pure `smobb-core` engine so both the TUI (Phase 3) and web (Phase 4) frontends inherit them. Discovery/decoding are derived by heuristics from the existing RON data; no `content/*.ron` edits. The headless `examples/play.rs` is updated to demonstrate them.

**Tech Stack:** Rust (edition 2021, stable ≥ 1.74), existing `serde` + `ron` only. No new dependencies.

## Global Constraints

- `smobb-core` lib stays PURE: no file/network/clock/stdin; no `thread::sleep` in `crates/smobb-core/src/` (only in `examples/play.rs`). Timing remains `Output::Sleep`.
- No new dependencies. Edition 2021, rust-version 1.74.
- **ZERO changes to `content/*.ron`** and ZERO changes to the `scene.rs` content schema.
- Cargo runs with: `source "$HOME/.cargo/env" && CARGO_NET_OFFLINE=true cargo test -p smobb-core`.
- DO NOT COMMIT (the user handles all git). DO NOT run `git` at all. DO NOT run the `play` example (it blocks on stdin); only `cargo test` / `cargo build`.
- Each task ends green on `cargo test -p smobb-core`.

## File Structure

```
crates/smobb-core/src/
  decode.rs      # NEW: decode_hex util
  lib.rs         # MODIFY: pub mod decode;
  engine.rs      # MODIFY: Output::Translation; discovered/visited/decoded state;
                 #         Jaffar-hex hook, Recap/Enter hooks, discovered-on-use;
                 #         display_name/classify/branch_label; options_view; journal_view
  examples/play.rs  # MODIFY: intercept options/?/journal/j; render Translation
crates/smobb-core/tests/
  qol_flow.rs    # NEW: real-content integration test
```

**Standard TDD cycle each task:** write failing test → run, confirm fail → implement → run, confirm pass. NO commits (controller snapshots between tasks).

---

## Task 1: `decode_hex` util

**Files:**
- Create: `crates/smobb-core/src/decode.rs`
- Modify: `crates/smobb-core/src/lib.rs` (add `pub mod decode;`)

**Interfaces:**
- Produces: `pub fn decode_hex(s: &str) -> Option<String>` — `Some(plain)` iff `s` (trimmed) is non-empty, even-length, all ASCII hex digits, and decodes to valid UTF-8 containing no control chars except `\n`/`\t`; else `None`.

- [ ] **Step 1: Write failing tests** (in `decode.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_valid_hex() {
        assert_eq!(decode_hex("48656C6C6F").as_deref(), Some("Hello"));
        // real Jaffar line: "NO! HOW DID YOU KNOW?!"
        assert_eq!(
            decode_hex("4E4F2120484F572044494420594F55204B4E4F573F21").as_deref(),
            Some("NO! HOW DID YOU KNOW?!")
        );
    }
    #[test]
    fn rejects_non_hex() {
        assert_eq!(decode_hex("hello world"), None); // spaces + non-hex
        assert_eq!(decode_hex("ABC"), None);          // odd length
        assert_eq!(decode_hex(""), None);             // empty
        assert_eq!(decode_hex("00"), None);           // NUL control char -> not printable
    }
}
```

- [ ] **Step 2: Run, confirm fail** — `source "$HOME/.cargo/env" && CARGO_NET_OFFLINE=true cargo test -p smobb-core decode` → FAIL (module/fn missing).

- [ ] **Step 3: Implement** (`decode.rs`)

```rust
//! Hexadecimal → ASCII/UTF-8 decoding for Jaffar's cipher lines.

/// Decode an even-length all-hex string to printable UTF-8, else None.
pub fn decode_hex(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s.len() % 2 != 0 {
        return None;
    }
    if !s.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let mut bytes = Vec::with_capacity(s.len() / 2);
    let sb = s.as_bytes();
    let mut i = 0;
    while i < sb.len() {
        let hi = (sb[i] as char).to_digit(16)? as u8;
        let lo = (sb[i + 1] as char).to_digit(16)? as u8;
        bytes.push((hi << 4) | lo);
        i += 2;
    }
    let text = String::from_utf8(bytes).ok()?;
    if text.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') {
        Some(text)
    } else {
        None
    }
}
```

- [ ] **Step 4: Add module declaration** — in `crates/smobb-core/src/lib.rs`, add `pub mod decode;` alongside the other `pub mod` lines.

- [ ] **Step 5: Run, confirm pass** — `... cargo test -p smobb-core decode` → PASS. Then full suite `... cargo test -p smobb-core` → all pass. NO commit.

---

## Task 2: `Output::Translation` + Jaffar-hex decode hook

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `decode::decode_hex`, `speaker::Speaker::Jaffar`.
- Produces: new variant `Output::Translation(String)`; engine field `decoded: Vec<(String, String)>` (hex, plain), deduped by hex; when the run loop emits `Line(Jaffar, text)` and `decode_hex(&text)` is `Some(plain)`, a following `Output::Translation(plain)` is emitted and the pair recorded.

- [ ] **Step 1: Write failing test** (in `engine.rs` test module)

```rust
#[test]
fn jaffar_hex_line_emits_translation() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Line(Jaffar, "48656C6C6F"),
            Line(Ali, "48656C6C6F"),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true); // drop Sleeps for a clean sequence
    let _ = e.start();
    let out = e.advance("go");
    // Jaffar line -> Line then Translation("Hello"); Ali line -> Line only (no translation)
    let translations: Vec<String> = out.iter().filter_map(|o| match o {
        Output::Translation(t) => Some(t.clone()), _ => None
    }).collect();
    assert_eq!(translations, vec!["Hello".to_string()]);
}
```

- [ ] **Step 2: Run, confirm fail** — `... cargo test -p smobb-core jaffar_hex` → FAIL.

- [ ] **Step 3: Implement**

In the `Output` enum, add the variant (keep existing variants):
```rust
    Translation(String),
```
In the `Engine` struct add field `decoded: Vec<(String, String)>,` and initialise `decoded: Vec::new(),` in `Engine::new`.
Replace the `Step::Line` arm in `run` with:
```rust
                Step::Line(sp, text) => {
                    out.push(Output::Line { text: text.clone(), color: sp.color() });
                    if sp == crate::speaker::Speaker::Jaffar {
                        if let Some(plain) = crate::decode::decode_hex(&text) {
                            out.push(Output::Translation(plain.clone()));
                            if !self.decoded.iter().any(|(h, _)| h == &text) {
                                self.decoded.push((text.clone(), plain));
                            }
                        }
                    }
                    if !self.skip { out.push(Output::Sleep(self.interval_ms)); }
                }
```

- [ ] **Step 4: Run, confirm pass** — `... cargo test -p smobb-core jaffar_hex` → PASS; full suite green. NO commit.

---

## Task 3: Discovered passwords from recaps (`extract_quoted` + Recap hook)

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: engine field `discovered: std::collections::BTreeSet<String>`; a free fn `extract_quoted(s: &str) -> Vec<String>` that returns single-quoted tokens containing no whitespace (≤24 chars), correctly skipping contraction apostrophes; the `Step::Recap` arm adds extracted tokens to `discovered`. Internal accessor `fn discovered(&self) -> &std::collections::BTreeSet<String>` (used by later tasks/tests).

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn extract_quoted_handles_contractions() {
    // contraction apostrophe in "S.M.O.B.B.'s" must NOT break token capture
    assert_eq!(
        extract_quoted("The password for S.M.O.B.B.'s files is '9285005'."),
        vec!["9285005".to_string()]
    );
    assert_eq!(extract_quoted("code is '<---------->'."), vec!["<---------->".to_string()]);
    assert_eq!(extract_quoted("It is '!SAVE'. Done."), vec!["!SAVE".to_string()]);
    assert!(extract_quoted("no quotes here").is_empty());
}

#[test]
fn recap_populates_discovered() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Recap("The password for the Tranava files is '7549009'."),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("go");
    assert!(e.discovered().contains("7549009"));
}
```

- [ ] **Step 2: Run, confirm fail** — `... cargo test -p smobb-core 'extract_quoted\|recap_populates'` → FAIL.

- [ ] **Step 3: Implement**

Add to the `Engine` struct: `discovered: std::collections::BTreeSet<String>,` and init `discovered: std::collections::BTreeSet::new(),` in `new`.
Add the free function (module scope in `engine.rs`):
```rust
/// Single-quoted tokens with no whitespace (<=24 chars). Skips contraction apostrophes.
fn extract_quoted(s: &str) -> Vec<String> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\'' {
            if let Some(rel) = s[i + 1..].find('\'') {
                let content = &s[i + 1..i + 1 + rel];
                if !content.is_empty()
                    && content.len() <= 24
                    && !content.chars().any(|c| c.is_whitespace())
                {
                    out.push(content.to_string());
                    i = i + 1 + rel + 1;
                    continue;
                }
            }
            i += 1;
        } else {
            i += 1;
        }
    }
    out
}
```
Update the `Step::Recap` arm:
```rust
                Step::Recap(t) => {
                    self.recap.add(&t);
                    for tok in extract_quoted(&t) {
                        self.discovered.insert(tok);
                    }
                    out.push(Output::Recap(t));
                }
```
Add accessor:
```rust
impl Engine {
    pub(crate) fn discovered(&self) -> &std::collections::BTreeSet<String> { &self.discovered }
}
```

- [ ] **Step 4: Run, confirm pass** — focused tests PASS; full suite green. NO commit.

---

## Task 4: Visited characters (`Enter` hook)

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: engine field `visited: std::collections::BTreeSet<String>`; const `CHARACTERS: &[&str]` (= the seven character locations); the `Step::Enter(id)` arm inserts `id` into `visited` when `id ∈ CHARACTERS`. Accessor `pub(crate) fn visited(&self) -> &std::collections::BTreeSet<String>`.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn entering_a_character_records_visit() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: {
            "hub": ( commands: [ Branch(keys:["ali"], target: Steps([ Enter("ali") ])) ] ),
            "ali": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
        },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("ali");
    assert!(e.visited().contains("ali"));
}
```

- [ ] **Step 2: Run, confirm fail** — `... cargo test -p smobb-core entering_a_character` → FAIL.

- [ ] **Step 3: Implement**

Add field `visited: std::collections::BTreeSet<String>,` (+ init in `new`). Add const at module scope:
```rust
const CHARACTERS: &[&str] = &["ali", "bader", "bevil", "hasan", "josh", "qassim", "smobb"];
```
In the `Step::Enter(id)` arm, immediately after `self.loc_stack.push(id.clone());`, add:
```rust
                    if CHARACTERS.contains(&id.as_str()) {
                        self.visited.insert(id.clone());
                    }
```
Add accessor:
```rust
impl Engine {
    pub(crate) fn visited(&self) -> &std::collections::BTreeSet<String> { &self.visited }
}
```

- [ ] **Step 4: Run, confirm pass** — focused PASS; full suite green. NO commit.

---

## Task 5: Option classification + `options_view` + discovered-on-use

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces:
  - module consts `CHARACTER_NAMES: &[&str]`, `STRUCTURAL: &[&str]`; free fns `display_name(id: &str) -> String`, `structural_label(key: &str) -> String`.
  - `fn branch_label(&self, loc_id: &str, b: &crate::scene::Branch) -> Option<String>` — `Some(label)` if the branch is a public option, `None` if secret.
  - `pub fn options_view(&self) -> Vec<Output>` — "You can:" public labels for the current location + a "Known passwords (try these):" line from `discovered`.
  - discovered-on-use: in `handle_command`, a matched branch whose `branch_label` is `None` adds the typed key to `discovered`.

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn options_view_lists_public_and_known() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [
            Branch(keys:["ali"], target: Steps([ Enter("ali") ])),
            Branch(keys:["7549009"], target: Steps([ Line(System, "secret") ])),
            Branch(keys:["end"], target: Steps([ Quit ])),
        ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let text: Vec<String> = e.options_view().iter().filter_map(|o| match o {
        Output::Line{text,..} => Some(text.clone()), _ => None }).collect();
    let joined = text.join("\n");
    assert!(joined.contains("Ali"));        // nav label
    assert!(joined.contains("Quit"));       // exit label
    assert!(!joined.contains("7549009"));   // secret hidden (not yet discovered)
    assert!(joined.to_lowercase().contains("known passwords"));
}

#[test]
fn using_a_secret_key_discovers_it() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [
            Branch(keys:["7549009"], target: Steps([ Line(System, "ok") ])),
        ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let _ = e.advance("7549009");
    assert!(e.discovered().contains("7549009"));
}
```

- [ ] **Step 2: Run, confirm fail** — `... cargo test -p smobb-core 'options_view\|using_a_secret'` → FAIL.

- [ ] **Step 3: Implement**

Module-scope consts + helpers:
```rust
const CHARACTER_NAMES: &[&str] = &["ali","bader","bevil","hasan","josh","qassim","qasim","smobb"];
const STRUCTURAL: &[&str] = &["guide","extra","story","stories","names","settings"];

fn display_name(id: &str) -> String {
    match id {
        "hub" => "Hub", "ali" => "Ali", "bader" => "Bader", "bevil" => "Bevil",
        "hasan" => "Hasan", "josh" => "Josh", "qassim" | "qasim" => "Qassim",
        "smobb" => "S.M.O.B.B.", "mavis" => "M.A.V.I.S.", "guide" => "Guide",
        "guide_names" => "Names", "extra" => "Stories",
        other => return {
            let mut c = other.chars();
            match c.next() { Some(f) => f.to_uppercase().chain(c).collect(), None => String::new() }
        },
    }.to_string()
}

fn structural_label(key: &str) -> String {
    match key {
        "guide" => "Guide", "extra" | "story" | "stories" => "Stories",
        "names" => "Names", "settings" => "Settings", _ => key,
    }.to_string()
}
```
`branch_label`:
```rust
impl Engine {
    fn branch_label(&self, loc_id: &str, b: &crate::scene::Branch) -> Option<String> {
        use crate::scene::{Step, Target};
        if let Target::Steps(steps) = &b.target {
            for s in steps {
                if let Step::Enter(x) = s { return Some(display_name(x)); }
            }
        }
        if b.keys.iter().any(|k| k == "end" || k == "quit") {
            return Some(if loc_id == "hub" { "Quit".into() } else { "Leave".into() });
        }
        for k in &b.keys {
            if CHARACTER_NAMES.contains(&k.as_str()) {
                return Some(format!("Ask about {}", display_name(k)));
            }
        }
        for k in &b.keys {
            if STRUCTURAL.contains(&k.as_str()) { return Some(structural_label(k)); }
        }
        None
    }

    pub fn options_view(&self) -> Vec<Output> {
        let c = crate::speaker::Color::White;
        let loc_id = self.loc_stack.last().cloned().unwrap_or_default();
        let loc = match self.content.locations.get(&loc_id) {
            Some(l) => l, None => return vec![Output::Line { text: "(no options)".into(), color: c }],
        };
        let mut out = vec![Output::Line { text: "You can:".into(), color: c }];
        let mut seen = std::collections::BTreeSet::new();
        for b in &loc.commands {
            if let Some(label) = self.branch_label(&loc_id, b) {
                if seen.insert(label.clone()) {
                    out.push(Output::Line { text: format!("  - {}", label), color: c });
                }
            }
        }
        let known = if self.discovered.is_empty() {
            "  (none discovered yet)".to_string()
        } else {
            format!("  {}", self.discovered.iter().cloned().collect::<Vec<_>>().join(", "))
        };
        out.push(Output::Line { text: "Known passwords (try these):".into(), color: c });
        out.push(Output::Line { text: known, color: c });
        out
    }
}
```
discovered-on-use in `handle_command` — change the matched block. Capture the loc id and check `branch_label`:
```rust
    fn handle_command(&mut self, input: &str) -> Vec<Output> {
        let loc_id = self.loc_stack.last().cloned().unwrap_or_default();
        let loc = self.content.locations.get(&loc_id).unwrap().clone();
        for b in &loc.commands {
            if b.keys.iter().any(|k| k.to_lowercase() == input) {
                if self.branch_label(&loc_id, b).is_none() {
                    self.discovered.insert(input.to_string());
                }
                self.push_target(&b.target);
                let mut out = self.run();
                if self.frames.is_empty() { out.extend(self.menu_prompt()); }
                return out;
            }
        }
        self.menu_prompt()
    }
```
(Keep the rest of `handle_command` behavior identical; only the loc_id capture + the discovered-on-use line are new.)

- [ ] **Step 4: Run, confirm pass** — focused PASS; full suite green. NO commit.

---

## Task 6: `journal_view`

**Files:**
- Modify: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: `pub fn journal_view(&self) -> Vec<Output>` — four labeled sections: `[Clues]` (recap entries), `[Passwords]` (`discovered`), `[People]` (`visited` via `display_name`), `[Decoded]` (`decoded` as `hex → plain`). Each prints `(none yet)` when empty.

- [ ] **Step 1: Write failing test**

```rust
#[test]
fn journal_view_has_four_sections_and_data() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Recap("clue: the password is '7549009'."),
            Line(Jaffar, "48656C6C6F"),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("go");
    let joined: String = e.journal_view().iter().filter_map(|o| match o {
        Output::Line{text,..} => Some(text.clone()), _ => None }).collect::<Vec<_>>().join("\n");
    assert!(joined.contains("[Clues]"));
    assert!(joined.contains("[Passwords]"));
    assert!(joined.contains("[People]"));
    assert!(joined.contains("[Decoded]"));
    assert!(joined.contains("7549009"));         // password discovered + clue text
    assert!(joined.contains("Hello"));           // decoded jaffar line
}
```

- [ ] **Step 2: Run, confirm fail** — `... cargo test -p smobb-core journal_view_has_four` → FAIL.

- [ ] **Step 3: Implement**

```rust
impl Engine {
    pub fn journal_view(&self) -> Vec<Output> {
        let c = crate::speaker::Color::White;
        let mut out = Vec::new();
        let mut line = |s: String| out.push(Output::Line { text: s, color: c });
        line("== JOURNAL ==".into());

        line("[Clues]".into());
        if self.recap.entries().is_empty() { line("  (none yet)".into()); }
        else { for e in self.recap.entries() { line(format!("  - {}", e)); } }

        line("[Passwords]".into());
        if self.discovered.is_empty() { line("  (none yet)".into()); }
        else { for p in &self.discovered { line(format!("  - {}", p)); } }

        line("[People]".into());
        if self.visited.is_empty() { line("  (none yet)".into()); }
        else { for v in &self.visited { line(format!("  - {}", display_name(v))); } }

        line("[Decoded]".into());
        if self.decoded.is_empty() { line("  (none yet)".into()); }
        else { for (h, p) in &self.decoded { line(format!("  - {} → {}", h, p)); } }

        out
    }
}
```
(Note: the `let mut line = |s|` closure borrows `out` mutably; structure the method so all reads of `self.*` happen inside the `format!` args, which is fine since they are immutable borrows of disjoint fields. If the borrow checker complains, replace the closure with direct `out.push(Output::Line{ text: ..., color: c })` calls.)

- [ ] **Step 4: Run, confirm pass** — focused PASS; full suite green. NO commit.

---

## Task 7: Wire the headless driver + real-content integration test

**Files:**
- Modify: `crates/smobb-core/examples/play.rs`
- Create: `crates/smobb-core/tests/qol_flow.rs`

**Interfaces:**
- Consumes: `Engine::options_view`, `Engine::journal_view`, `Output::Translation`.
- Produces: the `play` driver intercepts `options`/`?`/`look` → `options_view()` and `journal`/`j` → `journal_view()` (globally, like `recap`/`skipmode`), and renders `Output::Translation` dimmed + indented. An integration test proves: learning a password (via the real content chain) makes it appear in both `options_view` and `journal_view`, and a traversed Jaffar line is decoded into the journal's `[Decoded]`.

- [ ] **Step 1: Update `examples/play.rs`**

In `render`, add a `Translation` arm (dimmed + indented) — full updated function:
```rust
fn render(out: &[Output], e: &Engine) -> bool {
    for o in out {
        match o {
            Output::Line { text, color } => println!("{}{}\u{1b}[0m", color.ansi(), text),
            Output::Translation(t) => println!("\u{1b}[2m  \u{21b7} \"{}\"\u{1b}[0m", t),
            Output::Sleep(ms) => thread::sleep(Duration::from_millis(*ms as u64)),
            Output::Recap(_) => {}
            Output::AwaitInput => { print!("> "); io::stdout().flush().unwrap(); }
            Output::End => return true,
        }
    }
    let _ = e; false
}
```
In `main`'s input loop, extend the built-in interception:
```rust
        let out = match trimmed.as_str() {
            "skipmode" | "sm" => { e.set_skip(!e.skip()); vec![] }
            "recap" | "rp" => e.recap_view(),
            "options" | "?" | "look" => e.options_view(),
            "journal" | "j" => e.journal_view(),
            _ => e.advance(&input),
        };
```

- [ ] **Step 2: Write the integration test** (`crates/smobb-core/tests/qol_flow.rs`)

```rust
use smobb_core::content;
use smobb_core::engine::{Engine, Output};

fn lines(out: &[Output]) -> String {
    out.iter().filter_map(|o| match o {
        Output::Line { text, .. } => Some(text.clone()), _ => None
    }).collect::<Vec<_>>().join("\n")
}

#[test]
fn learning_a_password_surfaces_in_options_and_journal() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    // hub -> hasan -> ali teaches the Tranava-files password 7549009
    let _ = e.advance("hasan");
    let _ = e.advance("ali");
    let _ = e.advance("end"); // back to hub
    let opts = lines(&e.options_view());
    assert!(opts.contains("7549009"), "options should list the learned password; got:\n{opts}");
    let jrnl = lines(&e.journal_view());
    assert!(jrnl.contains("7549009"), "journal [Passwords] should include 7549009");
    assert!(jrnl.contains("Hasan"), "journal [People] should include a visited character");
}

#[test]
fn jaffar_line_decodes_into_journal() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    // Ali's !save wrong-answer path emits a Jaffar hex line; drive to it:
    let _ = e.advance("ali");          // enter Ali
    let _ = e.advance("!save");        // the !save recording -> ends in a Prompt
    let out = e.advance("wrong");      // default branch -> Jaffar hex lines + Translation
    let has_translation = out.iter().any(|o| matches!(o, Output::Translation(_)));
    assert!(has_translation, "a wrong breach answer should emit Jaffar translations");
    let jrnl = lines(&e.journal_view());
    assert!(jrnl.contains("[Decoded]"));
}
```

- [ ] **Step 3: Run** — `source "$HOME/.cargo/env" && CARGO_NET_OFFLINE=true cargo test -p smobb-core` → ALL pass (incl. both new integration tests). If `jaffar_line_decodes_into_journal` finds no Translation, DIAGNOSE which Ali path emits a Jaffar hex line (read `content/ali.ron`) and adjust the test's command sequence to reach one — do NOT change engine/content to force it. Confirm with `cargo build --workspace` (do NOT run the example).

- [ ] **Step 4:** NO commit. Report results.

---

## Self-Review (against the QoL spec)

- **Spec coverage:** Feature 1 decoder → Tasks 1–2 ✓; Feature 2 discovered options → Tasks 3 (recap discovery) + 5 (classify, options_view, discovered-on-use) ✓; Feature 3 journal → Tasks 4 (visited) + 6 (journal_view) ✓; driver demo → Task 7 ✓; zero content changes ✓ (no task touches `content/*.ron` or `scene.rs`).
- **Placeholder scan:** none — every step has concrete code or an exact diagnose-and-adjust instruction (Task 7 step 3).
- **Type consistency:** `Output::Translation(String)`, `decoded: Vec<(String,String)>`, `discovered`/`visited: BTreeSet<String>`, `options_view`/`journal_view`/`branch_label`/`display_name`/`extract_quoted`/`structural_label`/`CHARACTERS`/`CHARACTER_NAMES`/`STRUCTURAL` used consistently across tasks and the driver.

## Note on commits
Per the user's standing instruction, NO task commits. The controller snapshots between tasks via `git write-tree` for review; the user commits the final result.
