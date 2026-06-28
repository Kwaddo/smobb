# S.M.O.B.B. QoL Layer — Design

**Date:** 2026-06-27
**Status:** Approved (design)
**Builds on:** `2026-06-27-smobb-2.0-design.md` (Phases 1–2 complete: engine + content port)

## Goal

Make the game convenient and ideal to follow, addressing three player asks:
1. **Decoded cipher** — Jaffar's hexadecimal lines shown WITH their plain-English meaning ("show both"), no external converter.
2. **Discovered options** — a clear, always-available list of what you can do here + the passwords you've learned (undiscovered secrets stay hidden).
3. **Structured journal** — auto-populated notes (passwords / people / clues / decoded messages) so the non-linear story is easy to follow.

## Key constraint: ZERO content-file changes

All three are pure `smobb-core` engine additions, derived by heuristics that fit the
existing RON data. The ported content (`content/*.ron`) and its fidelity are
untouched. This is also why the same features automatically benefit both the
TUI (Phase 3) and web (Phase 4) frontends — they live in the shared engine.

## Placement & sequencing

- **Phase 2.5 (this spec):** engine QoL features (1–3 below) + update the headless
  `examples/play.rs` to demonstrate them. Fully testable headless.
- **Phase 3 (TUI):** renders these in a real interface (story pane with inline
  translations + a persistent options/journal sidebar). The exact layout is
  mocked when Phase 3 is designed.
- The web frontend (Phase 4) reuses the same engine outputs.

## Feature 1 — Hex decoder ("show both")

- New pure util: `pub fn decode_hex(s: &str) -> Option<String>` (module `crates/smobb-core/src/decode.rs`).
  - Returns `Some(plain)` iff `s` is non-empty, even-length, all chars in `[0-9A-Fa-f]`,
    and the decoded bytes are valid UTF-8 that is all printable/whitespace.
  - Returns `None` otherwise (so normal prose is never mangled).
- New output variant: `Output::Translation(String)`.
- Engine run loop: when about to emit `Line(Jaffar, text)`, if `decode_hex(&text)` is
  `Some(plain)`, emit the `Line` as today AND immediately follow it with
  `Output::Translation(plain)`. Gated on **speaker == Jaffar** so only the cipher is decoded.
- Each `(hex, plain)` pair is recorded (deduped) for the journal's **[Decoded]** section.
- Frontend rendering: the headless driver prints the translation dimmed + indented,
  e.g. `  ⤷ "you're trying to figure me out."` (ANSI faint). TUI/web render it
  dimmed/indented under the raw hex.

## Feature 2 — Discovered options (`options` / `?`, any menu)

### Discovery tracking (no content edits)
Engine adds `discovered: BTreeSet<String>`, filled two ways:
1. **From recaps:** when a `Step::Recap(t)` fires, scan `t` for single-quoted tokens
   (`'…'`) and add each to `discovered`. The ported recaps consistently quote the
   secret they teach (`'7549009'`, `'9285005'`, `'K1LL'`, `'end7312'`,
   `'<---------->'`, `'!SAVE'`, the converter URL recap has no quotes → nothing added).
2. **From use:** when the player successfully enters a key the classifier deems
   *secret* (see below), add that key to `discovered`.

### Option classification (by branch, no per-branch metadata)
For each command branch of the current location, classify:
- **Navigation** — target's steps contain `Enter(x)` → public; label = `display_name(x)`
  (e.g. hub's `ali` → "Ali").
- **Exit** — key is `end`/`quit` → public; label "Leave" (sub-location) or "Quit" (hub).
- **Topic** — key ∈ CHARACTER_NAMES (`ali,bader,bevil,hasan,josh,qassim,qasim,smobb,mavis`)
  and not Navigation → public; label "Ask about `<Name>`".
- **Structural** — key ∈ {`guide,extra,story,stories,names,settings`} → public; friendly label.
- **Otherwise** → *secret* (hidden from the options list until discovered).

### `options_view(&self) -> Vec<Output>`
Returns Lines:
- A "You can:" group listing the public labels above (deduped, stable order).
- A "Known passwords (try these):" group listing the `discovered` set (or "none yet").
Undiscovered secrets never appear — discovery is preserved.

## Feature 3 — Structured journal (`journal` / `j`, any menu)

Engine state additions: `visited: BTreeSet<String>` (character locations entered, via
`Step::Enter`), and reuse `recap` (Clues) + `discovered` (Passwords) + `decoded` (Decoded).

`journal_view(&self) -> Vec<Output>` returns four labeled sections (each "(none yet)" when empty):
- **[Clues]** — the recap clue strings (existing recap entries).
- **[Passwords]** — the `discovered` set.
- **[People]** — `visited` character locations, via `display_name`.
- **[Decoded]** — recorded `(hex → plain)` Jaffar translations.

This supersedes the flat recap viewer; `recap`/`rp` becomes an alias that shows [Clues]
(kept for backward compatibility with existing content/driver).

## Engine API summary (additions)

```rust
// decode.rs
pub fn decode_hex(s: &str) -> Option<String>;

// engine.rs
pub enum Output { /* …existing… */, Translation(String) }
impl Engine {
    pub fn options_view(&self) -> Vec<Output>;
    pub fn journal_view(&self) -> Vec<Output>;
    // internal: discovered/visited/decoded sets + hooks on Recap/Enter/Jaffar-line
    // internal: fn display_name(id: &str) -> String;  fn classify(...) for options
}
```
- `CHARACTER_NAMES` and a `display_name` map (id → "Ali", "Bader", …, plus locations
  hub/guide/extra) live in one place in the engine.

## Frontend (driver) changes — Phase 2.5 demo

`examples/play.rs`: intercept `options`/`?` → `options_view()`, `journal`/`j` → `journal_view()`
(globally, like `recap`/`skipmode` today); render `Output::Translation` dimmed + indented.
TUI/web do the same later.

## Testing (TDD, `cargo test -p smobb-core`)

- `decode_hex`: valid hex → text; odd length / non-hex / non-printable → None; a real
  Jaffar string decodes to the expected sentence.
- Translation emission: a `Line(Jaffar, "<hex>")` yields a following `Output::Translation`;
  a non-Jaffar hex-looking line does NOT.
- Discovery: a `Recap("… '7549009' …")` adds `7549009` to `discovered`; using a secret
  key adds it; using a public key does not.
- `options_view`: lists nav/exit/topic/structural labels; hides secrets; shows known
  passwords; a known password appears after its recap fires.
- `journal_view`: four sections populate correctly; `visited` grows on `Enter`.
- Integration: a playthrough that learns a password sees it in both `options_view`
  and `journal_view`, and a traversed Jaffar line appears decoded in [Decoded].

## Out of scope (YAGNI)

- No always-decode mode or decode toggle (user chose "show both" — always on).
- No full-reveal option list (user chose discovered-only).
- No content-file schema changes.
- TUI/web layout (their own phases).
