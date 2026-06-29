# Command Layer Phase 3 — Content Migration + Signposting + Help-Text Truth — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the confusing "dot/dash" breach events and other earned-code gates onto the `Gate` step, surface earned codes in the journal, and make the on-screen help text true (typed `skipmode`/`recap` actually fire; dead/misleading copy fixed).

**Architecture:** Pure content (`.ron`) edits plus a small frontend interception in the shared `App::submit`. Each breach `Prompt` (a `System` Morse-noise line followed by a two-branch prompt) becomes one `Gate`: the `["<---------->"]` branch's steps become `on_deploy`, the empty-keys default branch's steps become `on_missing` and `on_decline`. The frontend intercepts the now-truthful meta-commands via the Phase-1 `vocab` registry before calling the engine. Depends on Phase 1 (`vocab`) and Phase 2 (`Gate`/`Condition`).

**Tech Stack:** RON content under `content/`, Rust frontend `crates/smobb-tui-core/src/app.rs`, tests via `cargo test --workspace`.

## Global Constraints

- Phases 1 and 2 must be merged first (`vocab` registry; `Gate`/`GateSpec`/`Condition`/`If` in `smobb-core`).
- Preserve all existing dialogue verbatim when wrapping it into `Gate` branches — only the surrounding structure changes, never the words.
- Use ONE uniform signpost string for every breach gate (drift-proofing): `"S.M.O.B.B.: BREACH DETECTED — deploy prevention? (yes / no)"`.
- Secret codes (`9285005`, `!save`, `!break`, `!help`) are NOT touched, NOT signposted, NOT added to any registry.
- After each content file edit, run that file's content test (`cargo test -p smobb-core --test content_<name>`) — these parse the RON and will catch a malformed `Gate`.
- Run `cargo fmt`/`cargo test --workspace` before every commit.

---

### Task 1: Migrate breach `Prompt`s to `Gate` (content)

**Files:**
- Modify: `content/ali.ron`, `content/bader.ron`, `content/bevil.ron`, `content/hasan.ron`, `content/josh.ron`
- Test: `crates/smobb-core/tests/content_*.rs` (existing per-file parse tests) + a new structural test (Step 5)

**Interfaces:**
- Consumes: `Step::Gate`, `GateSpec` (Phase 2).
- Produces: every dot/dash breach event resolved by a `Gate`.

- [ ] **Step 1: Locate every breach prompt**

Run: `grep -rn 'keys: \["<---------->"\]' content/` and, for each hit, open the surrounding block. A site qualifies for migration when the `["<---------->"]` branch sits **inside a `Prompt([...])`** that also has an empty-keys default branch (these are the event gates, usually preceded by a `Line(System, "<dots/dashes>")`). A bare `["<---------->"]` branch in a location's `commands` list (typed as a normal command) is **left unchanged** — the literal code still works there.

- [ ] **Step 2: Apply the transform — worked example (bevil.ron `"breach"` command)**

BEFORE (the tail of the `breach` branch, `content/bevil.ron`):

```ron
                    Line(System, "--._-...__----"),
                    Prompt([
                        Branch(keys: ["<---------->"], target: Steps([
                            Line(Bevil, "Yeah, that noise! That's the breach!"),
                            Line(Bevil, "That's all though. Goodnight baby girl."),
                        ])),
                        Branch(keys: [], target: Steps([
                            Pause,
                            Line(Jaffar, "49276D206C61746520616761696E21"),
                            Line(Jaffar, "44616D6E69742120"),
                            Pause,
                        ])),
                    ]),
```

AFTER:

```ron
                    Line(System, "--._-...__----"),
                    Gate(GateSpec(
                        code: "<---------->",
                        signpost: "S.M.O.B.B.: BREACH DETECTED — deploy prevention? (yes / no)",
                        on_deploy: [
                            Line(Bevil, "Yeah, that noise! That's the breach!"),
                            Line(Bevil, "That's all though. Goodnight baby girl."),
                        ],
                        on_missing: [
                            Pause,
                            Line(Jaffar, "49276D206C61746520616761696E21"),
                            Line(Jaffar, "44616D6E69742120"),
                            Pause,
                        ],
                        on_decline: [
                            Pause,
                            Line(Jaffar, "49276D206C61746520616761696E21"),
                            Line(Jaffar, "44616D6E69742120"),
                            Pause,
                        ],
                    )),
```

Transform rule, applied identically at every qualifying site:
- The `["<---------->"]` branch's `Steps([...])` → `on_deploy: [...]` (same lines).
- The empty-keys default branch's `Steps([...])` → BOTH `on_missing: [...]` and `on_decline: [...]` (same lines — a player who declines or lacks the code gets the original break-through outcome).
- Keep the preceding `Line(System, "<noise>")` as-is (flavor).
- Use the uniform `signpost` string from Global Constraints.

- [ ] **Step 3: Migrate each qualifying site**

Work one file at a time; after each, run its parse test. Check off each site as its `Prompt` is converted:

- [ ] `content/bevil.ron` — both breach prompts (the `breach` command ~line 119, and the `hasan` command ~line 168)
- [ ] `content/josh.ron` — breach prompt(s) (~line 122 and the nested one ~line 155)
- [ ] `content/hasan.ron` — breach prompt(s) (~line 69, ~87, ~207)
- [ ] `content/ali.ron` — breach prompt(s) (~line 26, ~266)
- [ ] `content/bader.ron` — breach prompt (~line 190)

(Line numbers are pre-edit anchors from the audit; confirm each by the `Prompt`+`["<---------->"]`+empty-default shape, not the number.)

- [ ] **Step 4: Run each file's parse test as you go**

Run (example): `cargo test -p smobb-core --test content_bevil`
Expected: PASS — RON parses; a malformed `Gate` fails here with a parse error pointing at the file.

- [ ] **Step 5: Add a structural test that the gates landed**

Create `crates/smobb-core/tests/breach_gates.rs`:

```rust
use smobb_core::content;
use smobb_core::scene::Step;

/// Recursively count Gate steps in a step list.
fn count_gates(steps: &[Step]) -> usize {
    let mut n = 0;
    for s in steps {
        match s {
            Step::Gate(_) => n += 1,
            Step::If(_, a, b) => n += count_gates(a) + count_gates(b),
            Step::Prompt(branches) => {
                for br in branches {
                    if let smobb_core::scene::Target::Steps(inner) = &br.target {
                        n += count_gates(inner);
                    }
                }
            }
            _ => {}
        }
    }
    n
}

#[test]
fn real_content_has_breach_gates() {
    let c = content::load();
    let mut total = 0;
    for loc in c.locations.values() {
        for b in &loc.commands {
            if let smobb_core::scene::Target::Steps(steps) = &b.target {
                total += count_gates(steps);
            }
        }
    }
    assert!(total >= 5, "expected the migrated breach gates, found {total}");
}
```

If `Branch.target` / `Target` / `Scene` fields are not all `pub`, this test will not compile — in that case assert via a simpler reachable path, or make the needed fields `pub` in `scene.rs` (they are already `pub` per the model). Adjust the recursion to also descend `Branch` targets inside command branches as shown.

- [ ] **Step 6: Run it**

Run: `cargo test -p smobb-core --test breach_gates`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add content/ crates/smobb-core/tests/breach_gates.rs
git commit -m "feat(content): migrate dot/dash breach events to signposted Gate"
```

---

### Task 2: Fix `end7312` availability where it is delivered

**Files:**
- Modify: `content/josh.ron`
- Test: `crates/smobb-core/tests/content_josh.rs` (parse)

**Interfaces:** none new.

**Decision (documented deviation from spec §4):** `end7312` is a typed password command, not an event prompt — wrapping it in a `Gate` would force a "deploy? yes" confirm *after* the player already typed the exact code, which reads worse. So `end7312` stays a direct command; the real bug (audit #6) is that Jaffar delivers it in `josh` but it is not keyed there. Fix = add the branch in `josh`. The literal code keeps working everywhere it is already keyed (`hub`, `ali`).

- [ ] **Step 1: Write/extend the parse-and-route expectation**

Confirm the current behavior first. Run: `grep -n 'end7312' content/josh.ron` — expect only the Jaffar dialogue line, no `keys: ["end7312"]`.

- [ ] **Step 2: Add the command branch in `josh`**

In `content/josh.ron`, inside the `josh` location's `commands` list, add (next to the other code commands):

```ron
                Branch(keys: ["end7312"], target: Steps([
                    Goto("ending"),
                ])),
```

- [ ] **Step 3: Verify it parses and routes**

Run: `cargo test -p smobb-core --test content_josh`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add content/josh.ron
git commit -m "fix(content): honor 'end7312' in josh where Jaffar delivers it"
```

---

### Task 3: Review `k1ll`/`kill` and route consistently

**Files:**
- Read: `content/ali.ron`, `content/bader.ron`, `content/hasan.ron`, `content/josh.ron`, `content/qassim.ron`
- Possibly modify: `crates/smobb-core/src/vocab.rs` (only if `k1ll`/`kill` should be a registry group)
- Test: `crates/smobb-core/src/vocab.rs`

**Interfaces:** possibly a new registry group.

- [ ] **Step 1: Read every `k1ll`/`kill` usage**

Run: `grep -rn 'k1ll\|kill\|murder' content/` and read each branch. Determine: is `k1ll` a *learned* code the player types (treat like a password — leave exact, do nothing), or are `k1ll`/`kill`/`murderer` meant to be interchangeable phrasings of one action (then group them in the registry so they route identically everywhere)?

- [ ] **Step 2a: If interchangeable phrasings — add a registry group**

In `vocab.rs` `GROUPS`, add (only the spellings the content actually uses):

```ron
    Group { canonical: "kill", label: "Kill", aliases: &["kill", "k1ll", "murder", "murderer"] },
```

Then add a `vocab` test:

```rust
#[test]
fn kill_aliases_group() {
    assert_eq!(canonical("k1ll"), Some("kill"));
    assert_eq!(canonical("murderer"), Some("kill"));
}
```

Run: `cargo test -p smobb-core kill_aliases_group` → Expected: PASS.

- [ ] **Step 2b: If `k1ll` is a discovered secret code — leave it**

Do NOT add it to the registry (it must stay exact-typed and hidden, like other secret codes). Record the decision in the commit message. No code change.

- [ ] **Step 3: Run the suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
cargo fmt
git add -A
git commit -m "chore(core): route k1ll/kill per its actual semantics (see body)"
```

---

### Task 4: Make help text true — frontend meta-command interception + copy fixes

**Files:**
- Modify: `crates/smobb-tui-core/src/app.rs` (`submit`)
- Modify: `content/hub.ron` (the `interval`/`shortcut`/`play` copy), `content/guide.ron` (the `interval` line)
- Modify: `crates/smobb-core/src/engine.rs` (prune dead `NONRECORD` entries)
- Test: `crates/smobb-tui-core/src/app.rs`

**Interfaces:**
- Consumes: `smobb_core::vocab::canonical`.
- Produces: typed `skipmode`/`sm` toggles skip and `recap`/`rp` opens the journal, on both frontends (shared `App`).

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `app.rs`:

```rust
#[test]
fn typed_skipmode_toggles_skip_without_engine_advance() {
    use smobb_core::content;
    use smobb_core::engine::Engine;
    use web_time::Instant;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt:[Line(Smobb,"P")], commands: [] ) },
        scenes: {},
    )"#]);
    let mut app = App::new(Engine::new(c)); // skip starts off
    app.tick(Instant::now());
    assert!(app.awaiting());
    app.submit_for_test("sm"); // alias of skipmode
    // still awaiting (no engine advance / no menu re-run), and skip flipped on
    assert!(app.awaiting());
    app.submit_for_test("sm"); // toggle back off
    assert!(app.awaiting());
}

#[test]
fn typed_recap_opens_journal_tab() {
    use smobb_core::content;
    use smobb_core::engine::Engine;
    use web_time::Instant;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt:[Line(Smobb,"P")], commands: [] ) },
        scenes: {},
    )"#]);
    let mut app = App::new(Engine::new(c));
    app.tick(Instant::now());
    app.submit_for_test("rp"); // alias of recap
    assert_eq!(app.tab(), Tab::Journal);
    assert!(app.sidebar_visible());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-tui-core typed_skipmode_toggles_skip_without_engine_advance typed_recap_opens_journal_tab`
Expected: FAIL — currently `submit` passes `sm`/`rp` straight to the engine (empty-Steps no-op).

- [ ] **Step 3: Intercept meta-commands in `submit`**

In `app.rs`, replace the body of `submit`:

```rust
    pub fn submit(&mut self) {
        if !self.awaiting {
            return;
        }
        let cmd = std::mem::take(&mut self.input);
        // Frontend-owned meta-commands, resolved through the canonical registry
        // so every alias (sm, rp, ...) works and matches the help text.
        match smobb_core::vocab::canonical(&cmd) {
            Some("skipmode") => {
                let s = self.engine.skip();
                self.engine.set_skip(!s);
                self.transcript.push(Span {
                    text: format!("  \u{2699} skip mode: {}", if !s { "on" } else { "off" }),
                    color: Color::White,
                    dim: true,
                    speaker: None,
                });
                self.dirty = true;
                return;
            }
            Some("recap") | Some("journal") => {
                self.tab = Tab::Journal;
                self.sidebar_visible = true;
                return;
            }
            _ => {}
        }
        let outputs = self.engine.advance(&cmd);
        self.enqueue(outputs);
        self.dirty = true; // a command may have changed journal state
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-tui-core`
Expected: PASS — new tests pass; existing `App` tests stay green.

- [ ] **Step 5: Make the remaining copy honest**

In `content/hub.ron`, the `interval` branch currently says interval is configurable. Replace its line with the truth (interval is changed via the skip/fast keys, not a typed command):

```ron
                Branch(keys: ["interval", "in"], target: Steps([
                    Line(System, "Reading speed: hold Space (or Shift) to fast-forward; press F2 for instant text."),
                ])),
```

In `content/guide.ron`, replace the line describing a typed `interval` command with the same guidance (hold Space / F2). Leave the `recap`/`skipmode` guide lines as-is — they are now TRUE.

- [ ] **Step 6: Prune dead `NONRECORD` entries**

In `engine.rs`, the `NONRECORD` list contains commands that are not wired as topics. Remove the entries that are not real commands anywhere (`"options"`, `"journal"`, `"j"`, `"look"`, `"?"`) so the list only names actual non-recording commands. Keep `recap`/`rp`/`skipmode`/`sm`/`interval`/`in`/`guide`/`help`/`play`/`pl`/`info`/`information`/`shortcut`/`shortcuts`. This is a pure cleanup; behavior is unchanged because those tokens never matched a branch anyway.

- [ ] **Step 7: Run the full suite**

Run: `cargo test --workspace`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt --all
git add crates/smobb-tui-core/src/app.rs content/hub.ron content/guide.ron crates/smobb-core/src/engine.rs
git commit -m "fix: typed skipmode/recap fire real actions; honest interval copy; prune NONRECORD"
```

---

### Task 5: Full-suite verification + manual smoke

**Files:** none (verification)

- [ ] **Step 1: Whole workspace**

Run: `cargo test --workspace && cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: PASS, no fmt diff, no new clippy errors.

- [ ] **Step 2: Manual smoke of a breach**

Launch the TUI/web and reach a breach (e.g. via the Tranava-files path). Confirm: the signpost line appears; typing `yes` with the code known shows the prevention-success lines; typing `yes` without it (fresh run) shows the Jaffar break-through; `no` declines; `leave` aborts to the menu; the literal `<---------->` still deploys.

- [ ] **Step 3: Manual smoke of help truth**

At the hub, type `sm` (skip toggles, note appears), `rp` (journal opens), `stories` (enters Stories), a garbage word (the "I don't understand" feedback shows).

- [ ] **Step 4: Final commit if anything changed**

```bash
cargo fmt --all
git add -A
git commit -m "chore: phase 3 command-layer verification pass"
```

---

## Self-Review

**Spec coverage (design §4 Phase 3 + remaining §3.1 audit items):**
- Breach `Prompt`s → `Gate` with noise kept as flavor → Task 1. ✅
- `end7312` honored where delivered (audit #6) → Task 2. ✅ (Documented deviation: kept as a typed command, not a `Gate`, with rationale.)
- `k1ll`/`kill` consistent routing (open question from spec §9) → Task 3, decided by reading actual semantics. ✅
- Typed `recap`/`skipmode` fire real actions; honest `interval` copy; pruned `NONRECORD` (audit #11, #12, #9, #8) → Task 4. ✅
- `!save` in `qassim` (audit #7): `!save` is a secret/exact code; its absence in `qassim` is a missing content branch. **Add it during Task 4 Step 5** if desired — add `Branch(keys: ["!save"], target: Steps([...]))` mirroring another location's `!save` block. (Listed here so it is not lost; it is a one-line content add, secret behavior unchanged.)
- Journal/options signposting of earned codes: already satisfied — discovered codes (incl. `<---------->`) surface under the journal's `[Passwords]` and `options_view`'s "Known passwords" via Phase 1's `discovered` handling. No extra task needed.

**Placeholder scan:** the per-site line numbers in Task 1 Step 3 are anchors, not placeholders — each site is identified structurally (the `Prompt`+`["<---------->"]`+empty-default shape) and the full transform is given in Step 2. The `!save`@qassim add references "mirroring another location's `!save` block" — the implementer copies an existing block verbatim; flagged rather than duplicated here to avoid guessing qassim's exact dialogue.

**Type consistency:** `GateSpec` fields and `Step::Gate`/`Step::If`/`Target::Steps` match Phase 2 + the existing `scene.rs` model. `smobb_core::vocab::canonical` returns `Option<&'static str>` matched against `Some("skipmode")`/`Some("recap")`/`Some("journal")`. `Span { text, color, dim, speaker }`, `Tab::Journal`, `Color::White` match `app.rs`.

**Cross-phase dependency:** requires Phase 1 (`vocab`) and Phase 2 (`Gate`/`Condition`) merged.
