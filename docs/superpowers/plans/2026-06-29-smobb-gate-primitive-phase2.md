# Command Layer Phase 2 — State-Conditional Primitive + Gate Step — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give content the ability to branch on what the player has learned, and a single drift-proof `Gate` construct for "you learned a code — deploy it?" interactions, without changing any visible behavior yet.

**Architecture:** Add a `Condition` enum (evaluated against state the engine already tracks: `discovered`, `visited`, `asked`, `decoding_unlocked`), a generic `If(Condition, then, else)` step, and a `Gate(GateSpec)` step that emits a signposted prompt, accepts an affirmative **or** the literal code, and routes to `on_deploy` / `on_missing` / `on_decline`. The `Gate` suspends on input via a new `pending_gate` slot mirroring the existing `pending_prompt` mechanism. Depends on Phase 1 (`vocab::canonical`, `vocab::key_matches`).

**Tech Stack:** Rust (`smobb-core`), `ron`/`serde` for content deserialization, co-located `#[cfg(test)]` tests, `cargo test -p smobb-core`.

## Global Constraints

- Phase 1 must be merged first (`vocab` module exists with `canonical`, `key_matches`).
- No content `.ron` migration in Phase 2 — only new engine capability + tests using inline content. Real content moves to `Gate` in Phase 3.
- Affirmative/negative are defined ONLY in the `vocab` registry (`yes`/`no` groups from Phase 1); the `Gate` resolver consults `vocab::canonical(input) == Some("yes")` / `Some("no")`. Do not hardcode a second yes/no list.
- The literal code path (`vocab::key_matches(gate.code, input)`) always counts as deploy, preserving "typed codes still work."
- `discovered` stores lowercased tokens; `Condition::Knows` must compare against a lowercased/canonical key.
- Run `cargo fmt` before every commit; keep all existing tests green.

---

### Task 1: `Condition` enum

**Files:**
- Modify: `crates/smobb-core/src/scene.rs` (add enum near `Step`)
- Test: `crates/smobb-core/src/scene.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `pub enum Condition { Knows(String), Visited(String), Asked(String, String), DecodeUnlocked }` (derives `Debug, Clone, Deserialize`). Consumed by `If`/`Gate` and `Engine::eval_condition`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `scene.rs`:

```rust
#[test]
fn parses_condition_variants() {
    assert!(matches!(
        ron::from_str::<Condition>(r#"Knows("7549009")"#).unwrap(),
        Condition::Knows(s) if s == "7549009"
    ));
    assert!(matches!(
        ron::from_str::<Condition>(r#"Visited("ali")"#).unwrap(),
        Condition::Visited(s) if s == "ali"
    ));
    assert!(matches!(
        ron::from_str::<Condition>("DecodeUnlocked").unwrap(),
        Condition::DecodeUnlocked
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core parses_condition_variants`
Expected: FAIL — `cannot find type 'Condition'`.

- [ ] **Step 3: Write minimal implementation**

Add to `scene.rs` (after the `Step` enum):

```rust
/// A predicate over progress the engine already tracks. Used by `Step::If`
/// and `Step::Gate` to branch on what the player has learned/done.
#[derive(Debug, Clone, Deserialize)]
pub enum Condition {
    /// Player has discovered this code/password (case-insensitive).
    Knows(String),
    /// Player has entered this location id.
    Visited(String),
    /// Player has asked `topic` at location `loc` — `Asked(loc, topic)`.
    Asked(String, String),
    /// Jaffar's hex has been revealed as decodable.
    DecodeUnlocked,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core parses_condition_variants`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/scene.rs
git commit -m "feat(core): add Condition enum for state-gated content"
```

---

### Task 2: `If` step + `eval_condition` + run handling

**Files:**
- Modify: `crates/smobb-core/src/scene.rs` (add `If` variant to `Step`)
- Modify: `crates/smobb-core/src/engine.rs` (`eval_condition` method; new arm in `run`)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `Condition`.
- Produces: `Step::If(Condition, Vec<Step>, Vec<Step>)`; private `fn eval_condition(&self, c: &Condition) -> bool`. Task 5 reuses `eval_condition`.

- [ ] **Step 1: Add the `If` variant**

In `scene.rs`, add to the `Step` enum (after `Effect`):

```rust
    /// Run `then` if the condition holds, else `else`. State-gated branching.
    If(Condition, Vec<Step>, Vec<Step>),
```

- [ ] **Step 2: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn if_step_branches_on_knows() {
    use crate::content;
    // Branch "withpw" learns the password (Recap) then If(Knows) -> KNOWN.
    // Branch "nopw" has not learned it -> UNKNOWN.
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [
            Branch(keys:["withpw"], target: Steps([
                Recap("The password for the Tranava files is '7549009'."),
                If(Knows("7549009"), [ Line(System,"KNOWN") ], [ Line(System,"UNKNOWN") ]),
                Return,
            ])),
            Branch(keys:["nopw"], target: Steps([
                If(Knows("7549009"), [ Line(System,"KNOWN") ], [ Line(System,"UNKNOWN") ]),
                Return,
            ])),
        ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("nopw");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "UNKNOWN")));
    let out = e.advance("withpw");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "KNOWN")));
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p smobb-core if_step_branches_on_knows`
Expected: FAIL — non-exhaustive match in `run` (compile error) until the arm is added.

- [ ] **Step 4: Add `eval_condition` and the `run` arm**

In `engine.rs`, add the method inside `impl Engine` (near `run`):

```rust
    fn eval_condition(&self, c: &crate::scene::Condition) -> bool {
        use crate::scene::Condition;
        match c {
            Condition::Knows(t) => {
                let key = crate::vocab::canonical(t)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| t.to_lowercase());
                self.discovered.contains(&key)
            }
            Condition::Visited(id) => self.visited.contains(id),
            Condition::Asked(loc, topic) => {
                self.asked.get(loc).is_some_and(|s| s.contains(topic))
            }
            Condition::DecodeUnlocked => self.decoding_unlocked,
        }
    }
```

Then add the new arm to the `match step` block inside `run` (alongside the other `Step::` arms):

```rust
                Step::If(cond, then_steps, else_steps) => {
                    let chosen = if self.eval_condition(&cond) {
                        then_steps
                    } else {
                        else_steps
                    };
                    self.frames.push(Frame {
                        steps: chosen,
                        idx: 0,
                    });
                }
```

Also add a no-op arm in `emit_intro` and `emit_step_lines` only if their matches are exhaustive — they use `_ => {}`, so no change needed.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — new test passes; all existing engine tests stay green.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/scene.rs crates/smobb-core/src/engine.rs
git commit -m "feat(core): add If step + eval_condition for state-gated branching"
```

---

### Task 3: `GateSpec` struct + `Gate` step variant (parse only)

**Files:**
- Modify: `crates/smobb-core/src/scene.rs` (`GateSpec` struct; `Gate` variant on `Step`)
- Test: `crates/smobb-core/src/scene.rs`

**Interfaces:**
- Produces: `pub struct GateSpec { code, signpost, on_deploy, on_missing, on_decline }` and `Step::Gate(GateSpec)`. Task 4–5 implement runtime behavior.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `scene.rs`:

```rust
#[test]
fn parses_gate_step() {
    let s: Step = ron::from_str(r#"Gate(GateSpec(
        code: "<---------->",
        signpost: "BREACH DETECTED",
        on_deploy: [ Line(System, "STOPPED") ],
        on_missing: [ Line(System, "NO CODE") ],
        on_decline: [ Line(System, "IGNORED") ],
    ))"#).unwrap();
    match s {
        Step::Gate(g) => {
            assert_eq!(g.code, "<---------->");
            assert_eq!(g.signpost, "BREACH DETECTED");
            assert_eq!(g.on_deploy.len(), 1);
            assert_eq!(g.on_missing.len(), 1);
            assert_eq!(g.on_decline.len(), 1);
        }
        _ => panic!("expected Gate"),
    }
}

#[test]
fn gate_optional_branches_default_empty() {
    let s: Step = ron::from_str(r#"Gate(GateSpec(
        code: "x", signpost: "S", on_deploy: [ Line(System, "D") ],
    ))"#).unwrap();
    match s {
        Step::Gate(g) => {
            assert!(g.on_missing.is_empty());
            assert!(g.on_decline.is_empty());
        }
        _ => panic!("expected Gate"),
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core parses_gate_step`
Expected: FAIL — `cannot find ... GateSpec` / non-exhaustive matches.

- [ ] **Step 3: Write minimal implementation**

In `scene.rs`, add the struct and variant:

```rust
/// A "you learned a code — deploy it?" interaction. Emits `signpost`, then
/// waits for input: an affirmative (or the literal `code`) routes to
/// `on_deploy` when the player knows `code`, else `on_missing`; a negative
/// routes to `on_decline`.
#[derive(Debug, Clone, Deserialize)]
pub struct GateSpec {
    pub code: String,
    pub signpost: String,
    pub on_deploy: Vec<Step>,
    #[serde(default)]
    pub on_missing: Vec<Step>,
    #[serde(default)]
    pub on_decline: Vec<Step>,
}
```

And add to the `Step` enum:

```rust
    /// Signposted, state-driven code deployment (see `GateSpec`).
    Gate(GateSpec),
```

- [ ] **Step 4: Add a compile-only no-op arm so the crate builds**

`run` in `engine.rs` matches `Step` exhaustively. Add a temporary arm to keep it compiling (real behavior lands in Task 4):

```rust
                Step::Gate(_) => {}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p smobb-core parses_gate_step gate_optional_branches_default_empty`
Expected: PASS (both parse tests).

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/scene.rs crates/smobb-core/src/engine.rs
git commit -m "feat(core): add GateSpec + Gate step (parse)"
```

---

### Task 4: `pending_gate` slot + signpost emission

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (struct field, `new`, `run` arm)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: `pending_gate: Option<crate::scene::GateSpec>` engine field; `run`'s `Gate` arm emits the signpost `Line` (as `System`) + `AwaitInput` and stashes the spec. Task 5 consumes `pending_gate`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn gate_emits_signpost_then_awaits() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Gate(GateSpec(
                code: "<---------->",
                signpost: "BREACH DETECTED — deploy prevention? (yes)",
                on_deploy: [ Line(System, "STOPPED"), Return ],
                on_missing: [ Line(System, "NO CODE"), Return ],
                on_decline: [ Line(System, "IGNORED"), Return ],
            )),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("go");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text.contains("BREACH DETECTED"))));
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    // not yet resolved: none of the outcome lines emitted
    assert!(!out.iter().any(|o| matches!(o, Output::Line{text,..}
        if text == "STOPPED" || text == "NO CODE" || text == "IGNORED")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core gate_emits_signpost_then_awaits`
Expected: FAIL — the temporary `Step::Gate(_) => {}` emits nothing.

- [ ] **Step 3: Add the field and initialize it**

In the `Engine` struct add:

```rust
    pending_gate: Option<crate::scene::GateSpec>,
```

In `Engine::new`, add to the constructed value (next to `pending_prompt: None,`):

```rust
            pending_gate: None,
```

- [ ] **Step 4: Replace the temporary `Gate` arm**

In `run`, replace `Step::Gate(_) => {}` with:

```rust
                Step::Gate(spec) => {
                    out.push(Output::Line {
                        text: spec.signpost.clone(),
                        color: crate::speaker::Speaker::System.color(),
                        speaker: Some(crate::speaker::Speaker::System),
                    });
                    self.pending_gate = Some(spec);
                    out.push(Output::AwaitInput);
                    return out;
                }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p smobb-core gate_emits_signpost_then_awaits`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): Gate emits signpost and suspends on pending_gate"
```

---

### Task 5: Gate resolution in `advance` (deploy / missing / decline / re-await / escape)

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (`advance`: universal-escape clause + new `pending_gate` resolution block)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `pending_gate`, `eval_condition`, `vocab::canonical`, `vocab::key_matches`.
- Produces: full gate behavior. No new public signatures.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `engine.rs`:

```rust
fn gate_fixture() -> crate::scene::Content {
    crate::content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt:[Line(Smobb,"MENU")], commands: [
            Branch(keys:["learn"], target: Steps([
                Recap("The breach prevention code is '<---------->'."),
            ])),
            Branch(keys:["breach"], target: Steps([
                Gate(GateSpec(
                    code: "<---------->",
                    signpost: "BREACH — deploy prevention? (yes)",
                    on_deploy: [ Line(System, "STOPPED"), Return ],
                    on_missing: [ Line(System, "NO CODE"), Return ],
                    on_decline: [ Line(System, "IGNORED"), Return ],
                )),
            ])),
        ] ) },
        scenes: {},
    )"#])
}

#[test]
fn gate_deploys_when_known_and_affirmative() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("learn");  // discover the code
    let _ = e.advance("breach"); // signpost + await
    let out = e.advance("yes");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "STOPPED")));
}

#[test]
fn gate_missing_when_affirmative_without_code() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("breach"); // never learned the code
    let out = e.advance("yes");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "NO CODE")));
}

#[test]
fn gate_declines_on_negative() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("learn");
    let _ = e.advance("breach");
    let out = e.advance("no");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "IGNORED")));
}

#[test]
fn gate_literal_code_deploys() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("breach"); // even without "learn", typing the code proves knowledge
    let out = e.advance("<---------->");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "STOPPED")));
}

#[test]
fn gate_reawaits_on_unrecognized_input() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("breach");
    let out = e.advance("banana");
    // re-emits signpost, still awaiting, no outcome line
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text.contains("BREACH"))));
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
    assert!(!out.iter().any(|o| matches!(o, Output::Line{text,..}
        if text == "STOPPED" || text == "NO CODE" || text == "IGNORED")));
}

#[test]
fn gate_universal_escape_aborts_to_menu() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("breach");
    let out = e.advance("leave");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "MENU")));
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-core gate_`
Expected: FAIL — no resolution logic; affirmatives currently do nothing useful.

- [ ] **Step 3: Extend the universal-escape clause**

In `advance`, the escape clause currently reads:

```rust
        if matches!(input.as_str(), "leave" | "exit" | "back" | "end") {
            if self.pending_prompt.is_some() || !self.frames.is_empty() {
                self.pending_prompt = None;
                self.frames.clear();
                return self.menu_prompt();
            }
```

Change the first two lines of the inner block to also clear a pending gate:

```rust
        if matches!(input.as_str(), "leave" | "exit" | "back" | "end") {
            if self.pending_prompt.is_some() || self.pending_gate.is_some() || !self.frames.is_empty() {
                self.pending_prompt = None;
                self.pending_gate = None;
                self.frames.clear();
                return self.menu_prompt();
            }
```

- [ ] **Step 4: Add the gate-resolution block**

In `advance`, immediately AFTER the universal-escape `if` block and BEFORE `if let Some(branches) = self.pending_prompt.take()`, insert:

```rust
        if let Some(gate) = self.pending_gate.take() {
            let aff = crate::vocab::canonical(&input) == Some("yes");
            let neg = crate::vocab::canonical(&input) == Some("no");
            let typed_code = crate::vocab::key_matches(&gate.code, &input);
            let knows = self.eval_condition(&crate::scene::Condition::Knows(gate.code.clone()));
            let steps = if typed_code || (aff && knows) {
                Some(gate.on_deploy.clone())
            } else if aff {
                Some(gate.on_missing.clone())
            } else if neg {
                Some(gate.on_decline.clone())
            } else {
                None
            };
            match steps {
                Some(s) => {
                    self.frames.push(Frame { steps: s, idx: 0 });
                    let mut out = self.run();
                    if self.frames.is_empty()
                        && !out.contains(&Output::End)
                        && !matches!(out.last(), Some(Output::AwaitInput))
                    {
                        out.extend(self.menu_prompt());
                    }
                    return out;
                }
                None => {
                    // unrecognized: re-emit the signpost and keep waiting.
                    let signpost = gate.signpost.clone();
                    self.pending_gate = Some(gate);
                    return vec![
                        Output::Line {
                            text: signpost,
                            color: crate::speaker::Speaker::System.color(),
                            speaker: Some(crate::speaker::Speaker::System),
                        },
                        Output::AwaitInput,
                    ];
                }
            }
        }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — all six gate tests plus the full existing suite.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): Gate resolution (deploy/missing/decline/re-await/escape)"
```

---

### Task 6: Snapshot safety + full-suite verification

**Files:**
- Test: `crates/smobb-core/src/engine.rs` (one guard test)

**Interfaces:** none new.

- [ ] **Step 1: Guard that an open gate does not corrupt a snapshot**

`pending_gate` is transient runtime state and is intentionally NOT persisted (mirrors `pending_prompt`). Add a test confirming `snapshot()` still round-trips while a gate is open:

```rust
#[test]
fn snapshot_works_with_open_gate() {
    let mut e = Engine::new(gate_fixture());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("learn");
    let _ = e.advance("breach"); // gate now open
    let snap = e.snapshot();
    assert!(snap.discovered.contains(&"<---------->".to_string()));
    let snap2 = crate::save::SaveState::from_json(&snap.to_json()).unwrap();
    assert_eq!(snap, snap2);
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p smobb-core snapshot_works_with_open_gate`
Expected: PASS — `SaveState` has no gate field, so the open gate is irrelevant to persistence.

- [ ] **Step 3: Full workspace suite + lint**

Run: `cargo test --workspace && cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: PASS, no fmt diff, no new clippy errors. The frontends are unaffected (no new `Output` variant). Fix any warning inline.

- [ ] **Step 4: Commit**

```bash
cargo fmt --all
git add crates/smobb-core/src/engine.rs
git commit -m "test(core): snapshot is safe with an open gate; phase 2 verification"
```

---

## Self-Review

**Spec coverage (design §4 Phase 2):**
- `Condition` enum with Knows/Visited/Asked/DecodeUnlocked → Task 1 + `eval_condition` Task 2. ✅
- `Step::Gate(GateSpec{code, signpost, on_deploy, on_missing, on_decline})` → Tasks 3–5. ✅
- Affirmative or literal code → deploy; lacks code → missing; decline; re-await on other → Task 5. ✅
- `Step::If(Condition, then, else)` thin conditional → Task 2. ✅
- No visible content change; covered by unit tests only → all tasks use inline content. ✅

**Placeholder scan:** none — every code step shows complete code. The temporary `Step::Gate(_) => {}` in Task 3 is explicitly replaced in Task 4.

**Type consistency:** `Condition` (Task 1) is used by `eval_condition` (Task 2) and the Task 5 resolver. `GateSpec` fields (`code`, `signpost`, `on_deploy`, `on_missing`, `on_decline`) are defined in Task 3 and read identically in Tasks 4–5. `pending_gate: Option<GateSpec>` (Task 4) is `.take()`n in Task 5. `Output::Line { text, color, speaker }` and `Speaker::System.color()` match the engine's existing usage. `Frame { steps, idx }` matches the private struct in `engine.rs`.

**Dependency note:** `vocab::canonical`/`vocab::key_matches` come from Phase 1 — verify Phase 1 is merged before starting.
