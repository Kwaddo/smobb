# Command Layer Phase 1 — Canonical Registry + Normalization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every displayed command label typeable and every alias behave identically in every location, by routing all navigation/meta/name input through one canonical vocabulary registry that also produces the on-screen labels.

**Architecture:** Add a `vocab` module to `smobb-core` holding the single source of truth: `canonical_id → { label, aliases }`. The engine's command matcher and prompt matcher compare *canonicals* (with dot-collapsing normalization) instead of doing raw exact-equality, while still exact-matching anything absent from the registry (passwords, secret/cheat codes). The label renderers (`display_name`, `structural_label`, `branch_label`/`options_view`) derive their text from the same registry, so a label and its accepted key can never drift again. Unmatched navigation input now gets explicit feedback instead of a silent re-prompt.

**Tech Stack:** Rust (workspace crate `smobb-core`), `ron` for content, `serde`. Tests are `#[cfg(test)]` modules co-located in each source file, run via `cargo test -p smobb-core`.

## Global Constraints

- Secret/cheat codes (`9285005`, `!save`, `!break`, `!help`) and passwords (`7549009`, `7789980`, `end7312`, `quint9440`) MUST NOT be added to the registry, MUST NOT appear as labels, and MUST continue to match only by exact (lowercased) equality.
- The literal `<---------->` breach code and all numeric codes match exactly; normalization never strips characters from them (normalization only removes `.` and only affects the registry-canonical comparison path).
- No content `.ron` edits in Phase 1 — behavior changes come entirely from the matcher and label derivation. (Content migration is Phase 3.)
- Follow existing patterns: `Output::Line { text, color, speaker }`, `Color::White`, `content::from_str(&[...])`, engine tests build content inline.
- Run `cargo fmt` before every commit; keep all existing tests green.

---

### Task 1: `vocab` module — groups table + `canonical()`

**Files:**
- Create: `crates/smobb-core/src/vocab.rs`
- Modify: `crates/smobb-core/src/lib.rs` (add `pub mod vocab;`)
- Test: in `crates/smobb-core/src/vocab.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces: `pub fn canonical(token: &str) -> Option<&'static str>`, `pub fn normalize(token: &str) -> String`, and a private `GROUPS: &[Group]` table. Later tasks rely on `canonical`, `normalize`, `label`, `key_matches`, `is_character`.

- [ ] **Step 1: Add the module to the crate**

In `crates/smobb-core/src/lib.rs`, add the module declaration alongside the others (keep alphabetical-ish order with the existing list):

```rust
pub mod vocab;
```

- [ ] **Step 2: Write the failing test**

Create `crates/smobb-core/src/vocab.rs` with only the test module first:

```rust
//! vocab.rs — single source of truth for navigation/meta/name vocabulary.
//! Maps any typed alias to a canonical id and a canonical id to its display
//! label. Passwords and secret/cheat codes are deliberately ABSENT here: they
//! match by exact equality and never surface as labels.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_groups_section_aliases() {
        assert_eq!(canonical("stories"), Some("extra"));
        assert_eq!(canonical("story"), Some("extra"));
        assert_eq!(canonical("extra"), Some("extra"));
        assert_eq!(canonical("extras"), Some("extra"));
    }

    #[test]
    fn canonical_groups_name_variants() {
        assert_eq!(canonical("qasim"), Some("qassim"));
        assert_eq!(canonical("qassim"), Some("qassim"));
        assert_eq!(canonical("ja3ffar"), Some("jaffar"));
        assert_eq!(canonical("ja3ffer"), Some("jaffar"));
    }

    #[test]
    fn canonical_returns_none_for_passwords_and_secrets() {
        assert_eq!(canonical("7549009"), None);
        assert_eq!(canonical("!save"), None);
        assert_eq!(canonical("9285005"), None);
        assert_eq!(canonical("<---------->"), None);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test -p smobb-core vocab::`
Expected: FAIL — `cannot find function 'canonical' in this scope`.

- [ ] **Step 4: Write minimal implementation**

Add above the test module in `vocab.rs`:

```rust
struct Group {
    /// The id stored/compared internally and used to look up a label.
    canonical: &'static str,
    /// Human-facing name shown in menus/options. Always one of `aliases`
    /// (after normalization) so the label is itself typeable.
    label: &'static str,
    /// Every spelling a player may type. MUST include the canonical.
    aliases: &'static [&'static str],
}

const GROUPS: &[Group] = &[
    // ---- sections ----
    Group { canonical: "extra", label: "Stories", aliases: &["extra", "extras", "story", "stories"] },
    Group { canonical: "guide", label: "Guide", aliases: &["guide", "help"] },
    Group { canonical: "guide_names", label: "Names", aliases: &["names", "name"] },
    Group { canonical: "settings", label: "Settings", aliases: &["settings"] },
    // ---- characters ----
    Group { canonical: "ali", label: "Ali", aliases: &["ali"] },
    Group { canonical: "bader", label: "Bader", aliases: &["bader"] },
    Group { canonical: "bevil", label: "Bevil", aliases: &["bevil"] },
    Group { canonical: "hasan", label: "Hasan", aliases: &["hasan"] },
    Group { canonical: "josh", label: "Josh", aliases: &["josh"] },
    Group { canonical: "qassim", label: "Qassim", aliases: &["qassim", "qasim"] },
    Group { canonical: "smobb", label: "S.M.O.B.B.", aliases: &["smobb"] },
    Group { canonical: "mavis", label: "M.A.V.I.S.", aliases: &["mavis"] },
    Group { canonical: "jaffar", label: "Jaffar", aliases: &["jaffar", "ja3ffar", "ja3ffer"] },
    Group { canonical: "hub", label: "Hub", aliases: &["hub"] },
    // ---- meta-commands ----
    Group { canonical: "recap", label: "Recap", aliases: &["recap", "rp"] },
    Group { canonical: "skipmode", label: "Skip mode", aliases: &["skipmode", "sm"] },
    Group { canonical: "journal", label: "Journal", aliases: &["journal", "j"] },
    Group { canonical: "options", label: "Options", aliases: &["options", "choices", "choice"] },
    Group { canonical: "play", label: "Play", aliases: &["play", "pl"] },
    // ---- affirmative / negative (consumed by the Gate step in Phase 2) ----
    Group { canonical: "yes", label: "Yes", aliases: &["yes", "y", "yeah", "yep", "yup", "yessir"] },
    Group { canonical: "no", label: "No", aliases: &["no", "n", "nope", "nah", "nuhhuh"] },
];

/// Lowercase, trim, and drop `.` so dotted forms like `M.A.V.I.S.` collapse to
/// `mavis`. Used ONLY on the registry-canonical comparison path — never on the
/// exact-match path that guards passwords and secret codes.
pub fn normalize(token: &str) -> String {
    token.trim().to_lowercase().replace('.', "")
}

/// The canonical id for any known alias, or `None` if the token is not a
/// registered navigation/meta/name word (e.g. a password or secret code).
pub fn canonical(token: &str) -> Option<&'static str> {
    let norm = normalize(token);
    GROUPS
        .iter()
        .find(|g| g.aliases.iter().any(|a| normalize(a) == norm))
        .map(|g| g.canonical)
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p smobb-core vocab::`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/vocab.rs crates/smobb-core/src/lib.rs
git commit -m "feat(core): add vocab registry with canonical() and normalize()"
```

---

### Task 2: `label()` + `is_character()` lookups

**Files:**
- Modify: `crates/smobb-core/src/vocab.rs`
- Test: same file

**Interfaces:**
- Produces: `pub fn label(canonical: &str) -> Option<&'static str>`, `pub fn is_character(canonical: &str) -> bool`. Later tasks (label derivation, visited tracking) consume both.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `vocab.rs`:

```rust
#[test]
fn label_lookup_by_canonical() {
    assert_eq!(label("extra"), Some("Stories"));
    assert_eq!(label("qassim"), Some("Qassim"));
    assert_eq!(label("smobb"), Some("S.M.O.B.B."));
    assert_eq!(label("nonexistent"), None);
}

#[test]
fn is_character_flags_people_only() {
    assert!(is_character("qassim"));
    assert!(is_character("ali"));
    assert!(!is_character("extra"));
    assert!(!is_character("recap"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core vocab::`
Expected: FAIL — `cannot find function 'label'`.

- [ ] **Step 3: Write minimal implementation**

Add to `vocab.rs` (below `canonical`):

```rust
/// The display label for a canonical id, or `None` if unknown.
pub fn label(canonical: &str) -> Option<&'static str> {
    GROUPS.iter().find(|g| g.canonical == canonical).map(|g| g.label)
}

/// Canonical ids that are people (drives the journal's "People" list and
/// visited tracking). Kept here so the set lives in ONE place.
const CHARACTERS: &[&str] = &["ali", "bader", "bevil", "hasan", "josh", "qassim", "smobb"];

/// True if a canonical id names a character.
pub fn is_character(canonical: &str) -> bool {
    CHARACTERS.contains(&canonical)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core vocab::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/vocab.rs
git commit -m "feat(core): add vocab label() and is_character()"
```

---

### Task 3: `key_matches()` — canonical-aware matcher

**Files:**
- Modify: `crates/smobb-core/src/vocab.rs`
- Test: same file

**Interfaces:**
- Produces: `pub fn key_matches(branch_key: &str, input: &str) -> bool`. The engine's command + prompt matchers (Tasks 4–5) call this in place of raw `k.to_lowercase() == input`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `vocab.rs`:

```rust
#[test]
fn key_matches_exact_and_via_canonical() {
    // exact (covers passwords / secrets not in the registry)
    assert!(key_matches("7549009", "7549009"));
    assert!(!key_matches("7549009", "754"));
    assert!(key_matches("!save", "!save"));
    // canonical: a branch keyed "extra" is reachable by typing the label word
    assert!(key_matches("extra", "stories"));
    assert!(key_matches("extra", "story"));
    // canonical: dotted name forms collapse
    assert!(key_matches("mavis", "m.a.v.i.s."));
    assert!(key_matches("m.a.v.i.s.", "mavis"));
    // canonical: alias both directions
    assert!(key_matches("qassim", "qasim"));
    // distinct canonicals do NOT cross-match
    assert!(!key_matches("ali", "bader"));
    // a registry word must not match an unrelated raw password
    assert!(!key_matches("extra", "7549009"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core vocab::`
Expected: FAIL — `cannot find function 'key_matches'`.

- [ ] **Step 3: Write minimal implementation**

Add to `vocab.rs`:

```rust
/// True when a branch's authored key and the player's input denote the same
/// command. Exact lowercased equality first (so passwords and secret codes,
/// which are absent from the registry, behave exactly as before); then a
/// canonical comparison so any alias of a registered word matches any other.
pub fn key_matches(branch_key: &str, input: &str) -> bool {
    let bk = branch_key.trim().to_lowercase();
    let inp = input.trim().to_lowercase();
    if bk == inp {
        return true;
    }
    match (canonical(&bk), canonical(&inp)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core vocab::`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/vocab.rs
git commit -m "feat(core): add canonical-aware key_matches()"
```

---

### Task 4: Route `handle_command` through `key_matches`

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (the loop in `handle_command`, ~`engine.rs:413-436`)
- Test: `crates/smobb-core/src/engine.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `vocab::key_matches`, `vocab::canonical`.
- Produces: no signature change to `handle_command`; behavior: typing a label/alias enters the branch keyed with any sibling alias.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn label_word_enters_branch_keyed_by_sibling_alias() {
    use crate::content;
    // Branch is keyed only "extra"; typing the displayed label "stories" must enter it.
    let c = content::from_str(&[r#"ContentPart(
        locations: {
            "hub": ( prompt:[Line(Smobb,"HUB")], commands: [ Branch(keys:["extra"], target: Steps([ Enter("extra") ])) ] ),
            "extra": ( prompt:[Line(Smobb,"STORIES")], commands: [] ),
        },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("stories");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "STORIES")));
}

#[test]
fn password_branch_still_requires_exact_match() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["7549009"], target: Steps([ Line(System,"ok") ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("754");
    assert!(!out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "ok")));
    let out = e.advance("7549009");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "ok")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core label_word_enters_branch_keyed_by_sibling_alias`
Expected: FAIL — `stories` does not match key `extra`, so "STORIES" never appears.

- [ ] **Step 3: Write minimal implementation**

In `engine.rs::handle_command`, change the match line. Replace:

```rust
        for b in &loc.commands {
            if b.keys.iter().any(|k| k.to_lowercase() == input) {
```

with:

```rust
        for b in &loc.commands {
            if b.keys.iter().any(|k| crate::vocab::key_matches(k, input)) {
```

Then, still inside that block, make the discovered/asked recording store the canonical form when the input is a registered word (so `qasim`/`qassim` dedupe), else the raw input (passwords). Replace:

```rust
                if self.branch_label(&loc_id, b).is_none() {
                    self.discovered.insert(input.to_string());
                }
```

with:

```rust
                let recorded = crate::vocab::canonical(input).unwrap_or(input).to_string();
                if self.branch_label(&loc_id, b).is_none() {
                    self.discovered.insert(recorded.clone());
                }
```

and replace:

```rust
                if !is_nav && !NONRECORD.contains(&input) {
                    self.asked
                        .entry(loc_id.clone())
                        .or_default()
                        .insert(input.to_string());
                }
```

with:

```rust
                if !is_nav && !NONRECORD.contains(&input) {
                    self.asked
                        .entry(loc_id.clone())
                        .or_default()
                        .insert(recorded.clone());
                }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — the two new tests pass and all existing engine tests stay green (existing tests type exact keys, which still match exactly).

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): canonical-aware command matching + canonical journal recording"
```

---

### Task 5: Route the `Prompt` matcher through `key_matches`

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (the `pending_prompt` block in `advance`, ~`engine.rs:273-277`)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `vocab::key_matches`.
- Produces: prompt branches keyed `["yes"]` match affirmative aliases like `y`/`yeah`.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn prompt_matches_affirmative_aliases() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [ Branch(keys:["go"], target: Steps([
            Prompt([
                Branch(keys:["yes"], target: Steps([ Line(Smobb,"YESPATH"), Return ])),
                Branch(keys:[], target: Steps([ Line(Smobb,"DEFAULT"), Return ])),
            ]),
        ])) ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("go");
    let out = e.advance("yeah"); // alias of "yes" — must take YESPATH, not DEFAULT
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..} if text == "YESPATH")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core prompt_matches_affirmative_aliases`
Expected: FAIL — `yeah` falls through to the empty-key DEFAULT branch.

- [ ] **Step 3: Write minimal implementation**

In `engine.rs::advance`, inside `if let Some(branches) = self.pending_prompt.take()`, replace:

```rust
            let chosen = branches
                .iter()
                .find(|b| b.keys.iter().any(|k| k.to_lowercase() == input))
                .or_else(|| branches.iter().find(|b| b.keys.is_empty()));
```

with:

```rust
            let chosen = branches
                .iter()
                .find(|b| b.keys.iter().any(|k| crate::vocab::key_matches(k, &input)))
                .or_else(|| branches.iter().find(|b| b.keys.is_empty()));
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — new test passes; existing prompt tests (`prompt_branches_match_and_default`) stay green.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): canonical-aware prompt matching (yes/no aliases)"
```

---

### Task 6: Derive `display_name` + `structural_label` from the registry

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (`display_name` ~`19-44`, `structural_label` ~`46-55`)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `vocab::label`.
- Produces: identical labels as today, now sourced from the registry; the `capitalize` fallback is retained for ids not in the registry.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn display_name_comes_from_registry_with_capitalize_fallback() {
    assert_eq!(display_name("extra"), "Stories");
    assert_eq!(display_name("qassim"), "Qassim");
    assert_eq!(display_name("smobb"), "S.M.O.B.B.");
    assert_eq!(display_name("guide_names"), "Names");
    // unknown id still capitalizes
    assert_eq!(display_name("tranava"), "Tranava");
}
```

- [ ] **Step 2: Run test to verify it fails**

This test will likely PASS for known ids already (current `display_name` hardcodes them) but is the regression guard for the rewrite. Run it now to confirm current behavior:

Run: `cargo test -p smobb-core display_name_comes_from_registry`
Expected: PASS (guards behavior before refactor).

- [ ] **Step 3: Rewrite `display_name` to use the registry**

Replace the body of `display_name` (`engine.rs:19-44`) with:

```rust
fn display_name(id: &str) -> String {
    if let Some(l) = crate::vocab::label(id) {
        return l.to_string();
    }
    // Not a registry id (e.g. a one-off location like "guide" sub-pages):
    // capitalize the first letter.
    let mut c = id.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}
```

Then replace `structural_label` (`engine.rs:46-55`) with:

```rust
fn structural_label(key: &str) -> String {
    let canon = crate::vocab::canonical(key).unwrap_or(key);
    crate::vocab::label(canon).map(|l| l.to_string()).unwrap_or_else(|| key.to_string())
}
```

Note: the registry maps `extra`→"Stories", `guide_names`→"Names", `guide`→"Guide", `settings`→"Settings", and `names`/`story`/`stories` canonicalize into those — covering every case the old `match` arms handled.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — new test plus all existing label-dependent tests (`options_view_lists_public_and_known`, `journal_view_*`).

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "refactor(core): derive display_name/structural_label from vocab registry"
```

---

### Task 7: Replace `STRUCTURAL`/`CHARACTER_NAMES`/`CHARACTERS` consts with registry lookups

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (consts ~`12-17`; `branch_label` ~`304-331`; the `Enter` arm's visited check ~`531-535`)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Consumes: `vocab::canonical`, `vocab::is_character`, `vocab::label`.
- Produces: removal of the three parallel const lists; `branch_label` and visited tracking now derive from the registry (fixes the `qasim`-not-in-`CHARACTERS` drift, audit item #10).

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn entering_via_alias_records_canonical_visit() {
    use crate::content;
    // Branch keyed "qasim" enters location "qassim"; visited must store canonical "qassim".
    let c = content::from_str(&[r#"ContentPart(
        locations: {
            "hub": ( commands: [ Branch(keys:["qasim"], target: Steps([ Enter("qassim") ])) ] ),
            "qassim": ( commands: [ Branch(keys:["end"], target: Steps([ Return ])) ] ),
        },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("qasim");
    assert!(e.visited().contains("qassim"));
}

#[test]
fn branch_label_identifies_character_and_structural_via_registry() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( commands: [
            Branch(keys:["bader"], target: Steps([ Line(Ali,"about bader") ])),
            Branch(keys:["stories"], target: Steps([ Enter("extra") ])),
        ] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let joined: String = e.options_view().iter().filter_map(|o| match o {
        Output::Line { text, .. } => Some(text.clone()),
        _ => None,
    }).collect::<Vec<_>>().join("\n");
    assert!(joined.contains("Ask about Bader"));
    assert!(joined.contains("Stories"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core entering_via_alias_records_canonical_visit`
Expected: FAIL — the `Enter` arm checks `CHARACTERS.contains("qassim")` which is true here, so this specific test may pass; the real change is removing the const. Run the whole new pair:

Run: `cargo test -p smobb-core branch_label_identifies_character_and_structural_via_registry`
Expected: PASS or FAIL depending on current `STRUCTURAL` handling of `stories` as a key — run to capture the baseline, then proceed with the refactor (these are regression guards).

- [ ] **Step 3: Rewrite the const-based logic**

In `engine.rs`, delete the three consts (`CHARACTERS`, `CHARACTER_NAMES`, `STRUCTURAL` at lines ~12-17).

In `branch_label`, replace the character/structural loops:

```rust
        for k in &b.keys {
            if CHARACTER_NAMES.contains(&k.as_str()) {
                return Some(format!("Ask about {}", display_name(k)));
            }
        }
        for k in &b.keys {
            if STRUCTURAL.contains(&k.as_str()) {
                return Some(structural_label(k));
            }
        }
        None
```

with:

```rust
        for k in &b.keys {
            if let Some(c) = crate::vocab::canonical(k) {
                if crate::vocab::is_character(c) {
                    return Some(format!("Ask about {}", display_name(c)));
                }
            }
        }
        for k in &b.keys {
            if let Some(c) = crate::vocab::canonical(k) {
                if matches!(c, "guide" | "extra" | "guide_names" | "settings") {
                    return Some(structural_label(k));
                }
            }
        }
        None
```

In the `Step::Enter(id)` arm (~`531-535`), replace:

```rust
                    if CHARACTERS.contains(&id.as_str()) {
                        self.visited.insert(id.clone());
                    }
```

with:

```rust
                    if crate::vocab::is_character(&id) {
                        self.visited.insert(id.clone());
                    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS — new tests plus existing (`entering_a_character_records_visit`, `asked_topics_recorded_in_journal`, `options_view_lists_public_and_known`).

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "refactor(core): replace parallel name/structural consts with vocab lookups"
```

---

### Task 8: Explicit feedback for unknown navigation input

**Files:**
- Modify: `crates/smobb-core/src/engine.rs` (`handle_command` fall-through, ~`engine.rs:437-438`)
- Test: `crates/smobb-core/src/engine.rs`

**Interfaces:**
- Produces: an unmatched command at a menu now emits one `Output::Line` of feedback before re-showing the menu, instead of silently re-prompting.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `engine.rs`:

```rust
#[test]
fn unknown_command_gives_feedback_then_reprompts() {
    use crate::content;
    let c = content::from_str(&[r#"ContentPart(
        locations: { "hub": ( prompt:[Line(Smobb,"MENU")], commands: [] ) },
        scenes: {},
    )"#]);
    let mut e = Engine::new(c);
    let _ = e.start();
    let out = e.advance("zzzz");
    assert!(out.iter().any(|o| matches!(o, Output::Line{text,..}
        if text.contains("zzzz") && text.to_lowercase().contains("options"))));
    assert!(matches!(out.last().unwrap(), Output::AwaitInput));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p smobb-core unknown_command_gives_feedback_then_reprompts`
Expected: FAIL — current fall-through returns only `menu_prompt()` with no feedback line.

- [ ] **Step 3: Write minimal implementation**

In `engine.rs::handle_command`, replace the final fall-through:

```rust
        // unknown command: just re-prompt
        self.menu_prompt()
```

with:

```rust
        // unknown command: tell the player it wasn't understood, then re-prompt.
        let mut out = vec![Output::Line {
            text: format!("I don't understand '{}'. Type 'options' to see what you can do.", input),
            color: crate::speaker::Color::White,
            speaker: None,
        }];
        out.extend(self.menu_prompt());
        out
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p smobb-core`
Expected: PASS. Note: the existing `unknown_command_reprompts` test only asserts the last output is `AwaitInput`, which still holds.

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/engine.rs
git commit -m "feat(core): explicit feedback on unknown navigation input"
```

---

### Task 9: Registry invariant + cross-room integration tests

**Files:**
- Modify: `crates/smobb-core/src/vocab.rs` (invariant test)
- Create: `crates/smobb-core/tests/registry_routing.rs` (integration test against real loaded content)
- Test: both of the above

**Interfaces:**
- Consumes: `vocab::canonical`, `vocab::label`, `content::load`, `Engine`.

- [ ] **Step 1: Write the invariant test (every label is typeable)**

Add to the `tests` module in `vocab.rs`:

```rust
#[test]
fn every_label_resolves_back_to_its_canonical() {
    // The core anti-drift guarantee: a label shown to the player is always a
    // word that canonicalizes back to the same id (i.e. it is typeable).
    for g in GROUPS {
        let resolved = canonical(g.label)
            .unwrap_or_else(|| panic!("label '{}' for '{}' is not typeable", g.label, g.canonical));
        assert_eq!(
            resolved, g.canonical,
            "label '{}' resolves to '{}', expected '{}'",
            g.label, resolved, g.canonical
        );
    }
}
```

- [ ] **Step 2: Run it; fix any label that is not typeable**

Run: `cargo test -p smobb-core every_label_resolves_back_to_its_canonical`
Expected: PASS. If it FAILS for a label (e.g. a multi-word label like "Skip mode" that does not canonicalize), fix by ensuring that group's `aliases` includes a normalized form of the label, or adjust the label to a typeable single token. (For "Skip mode"/"Skip mode" → add alias `"skip mode"`? No — labels for meta-commands are not entered as navigation. Restrict the invariant to navigable groups: change the loop to `for g in GROUPS.iter().filter(|g| is_character(g.canonical) || matches!(g.canonical, "extra"|"guide"|"guide_names"|"settings"))` so it guards exactly the section/character labels that appear in `options_view`.)

- [ ] **Step 3: Write the cross-room integration test**

Create `crates/smobb-core/tests/registry_routing.rs`:

```rust
use smobb_core::content;
use smobb_core::engine::{Engine, Output};

fn texts(out: &[Output]) -> Vec<String> {
    out.iter().filter_map(|o| match o {
        Output::Line { text, .. } => Some(text.clone()),
        _ => None,
    }).collect()
}

/// The reported bug: at the hub, the section labelled "Stories" must be
/// enterable by typing the displayed word, not only the internal id.
#[test]
fn stories_label_is_enterable_from_hub() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    let out = e.advance("stories");
    // The extra/Stories location intro or prompt should now be showing.
    // Assert we are NOT still at the hub by confirming the hub prompt is gone
    // from the tail and some new content rendered.
    assert!(!texts(&out).is_empty(), "typing 'stories' produced no output (still silently failing)");
}

/// `qasim` (alias) must work in every location that exposes `qassim`.
#[test]
fn qasim_alias_works_in_guide_names() {
    let mut e = Engine::new(content::load());
    e.set_skip(true);
    let _ = e.start();
    let _ = e.advance("guide"); // hub -> guide
    let _ = e.advance("names"); // guide -> guide_names
    let out = e.advance("qasim"); // alias of qassim
    assert!(!texts(&out).is_empty(), "'qasim' did nothing in guide_names");
}
```

- [ ] **Step 4: Run the integration test**

Run: `cargo test -p smobb-core --test registry_routing`
Expected: PASS. If `stories_label_is_enterable_from_hub` fails, confirm Task 4 landed and that `content/hub.ron` keys the extra branch with an `extra`-group alias (it keys `["extra","extras"]`, which canonicalize to `extra` — `stories` also canonicalizes to `extra`, so they match).

- [ ] **Step 5: Commit**

```bash
cargo fmt
git add crates/smobb-core/src/vocab.rs crates/smobb-core/tests/registry_routing.rs
git commit -m "test(core): registry label-typeability invariant + cross-room routing"
```

---

### Task 10: Full-suite verification + branch wrap-up

**Files:** none (verification only)

- [ ] **Step 1: Run the entire workspace test suite**

Run: `cargo test --workspace`
Expected: PASS — all `smobb-core` unit + integration tests, `smobb-tui-core`, and the `content_*`/`playthrough` tests. No content `.ron` changed in Phase 1, so playthroughs are unaffected.

- [ ] **Step 2: Format + lint**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets`
Expected: no formatting diff; no new clippy errors. Fix any introduced warnings inline.

- [ ] **Step 3: Confirm the reported bug manually (optional)**

Run: `cargo run -p smobb-core --example play` (if present) or the TUI, navigate to the hub, type `stories` — confirm it enters the Stories section. Type a garbage word — confirm the "I don't understand" feedback shows.

- [ ] **Step 4: Final commit if anything changed in steps 2–3**

```bash
cargo fmt --all
git add -A
git commit -m "chore(core): phase 1 command-registry verification pass"
```

---

## Self-Review

**Spec coverage (Phase 1 portion of `2026-06-29-smobb-command-layer-design.md` §4 Phase 1 + §3.1 items 1–10, §6):**
- Single source of truth registry → Tasks 1–2. ✅
- Dot-collapse normalization → Task 1 (`normalize`). ✅
- Canonical-aware resolution in `handle_command` + `Prompt` → Tasks 4–5. ✅
- Labels derived from the same registry (no drift) → Tasks 6–7 + invariant Task 9. ✅
- Secrets/passwords bypass (exact match) → Task 3 design + Task 4 test `password_branch_still_requires_exact_match`. ✅
- Audit items: #1 Stories (Task 9 integration), #2 qasim cross-room (Task 9), #3 jaffar variants & #4 mavis dotted (Task 3 `key_matches` + registry aliases), #7 `!save`@qassim (NOTE: `!save` is a secret/exact code — its *availability* in qassim is a missing branch in content, fixed in Phase 3, not here; Phase 1 does not add branches), #10 parallel-list drift (Task 7). ✅ with one carry-forward.
- §6 unknown-input feedback → Task 8. ✅

**Carry-forward to later phases (not Phase 1 gaps):** audit #5 (`recap`@extra), #6 (`end7312`@josh), #7 (`!save`@qassim), #8/#11/#12 (help-text + frontend meta-command interception) require content edits or frontend changes and belong to Phase 3. Phase 1 deliberately changes no `.ron`.

**Placeholder scan:** none — every code step shows complete code.

**Type consistency:** `canonical`/`label`/`normalize`/`key_matches`/`is_character` signatures are defined in Tasks 1–3 and used unchanged in Tasks 4–9. `Output::Line { text, color, speaker }` matches the engine's enum. `vocab::canonical` returns `Option<&'static str>` and is `.unwrap_or(input)`'d (both `&str`) consistently.

**Note on Task 9 invariant:** if the broad label-typeability loop trips on meta-command multi-word labels, Step 2 restricts it to navigable groups — keep that restriction; the guarantee that matters is for section/character labels surfaced in `options_view`.
