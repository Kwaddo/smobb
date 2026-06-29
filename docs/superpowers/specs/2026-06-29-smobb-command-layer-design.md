# S.M.O.B.B. — Consistent, State-Aware Command Layer

- **Date:** 2026-06-29
- **Status:** Approved (design); pending implementation plan
- **Author:** brainstormed with the maintainer
- **Affects:** `crates/smobb-core` (engine, scene model), `content/*.ron`, both frontends (`smobb-tui-core`, `smobb-web`)

## 1. Problem

Player input is routed by hand-typed `keys` lists on each `Branch`, matched by exact lowercase equality (`engine.rs::handle_command`, and the `Prompt` matcher). There is no shared vocabulary and no normalization beyond `trim().to_lowercase()`. Three failure modes result:

1. **Label ↔ key drift.** Sections are *displayed* under names produced by a separate map (`display_name`, `structural_label`, the `STRUCTURAL`/`CHARACTER_NAMES` consts) that is not forced to agree with the keys a `Branch` actually accepts. You type the word you see, nothing happens. The reported symptom — typing `stories`/`story` at the hub does nothing because the branch only accepts `extra`/`extras` (`content/hub.ron:84`), even though the section is labelled "Stories" everywhere — is one instance of a whole class (12 found, §3).

2. **Redundant retyping of earned codes.** Cryptic codes the player has already learned (e.g. the breach code `<---------->`) must be retyped exactly to progress, even though the engine *already* records that the player learned them (the code lands in the `discovered` set via the `Recap` quoted-token extraction). The "dot and dash" breach sections compound this: a breach is signalled only by an unlabelled Morse-like noise line (`Line(System, "--._-...__----")`) followed by a blank `Prompt` — nothing tells the player a breach is happening or what to do.

3. **Help text that lies.** On-screen guidance promises typed commands that do not fire (`recap`/`rp`, `skipmode`/`sm`, `interval`/`in` have empty or no-op targets and are not intercepted by either frontend), and the engine's `NONRECORD` list references commands that are not wired.

Root cause is identical across all three: routing vocabulary and on-screen labels are authored independently per branch/per file, with no single source of truth and no awareness of game state.

## 2. Goals / Non-goals

**Goals**
- Every label shown to the player is typeable; the displayed name and the accepted key cannot drift.
- Command synonyms behave identically in every location.
- Earned cryptic gates are **signposted** and resolve from game **state**; the literal code still works as an added path, never a replacement.
- Secret/cheat codes stay hidden and exact-typed — their surprise is preserved.
- On-screen help matches actual behavior.

**Non-goals**
- No story/plot/dialogue rewrites beyond what a gate's signpost wording requires.
- No change to the behavior or discoverability of secret codes.
- No fuzzy/typo matching of passwords or secret codes (puzzle/secret integrity).
- No change to the visual-effects system.

## 3. Inventory (audit findings to be resolved)

### 3.1 Routing inconsistencies (12)

| # | Where | Shown / expected | Actually wired | Class |
|---|---|---|---|---|
| 1 | `hub.ron:84` | section labelled "Stories" | keys `extra`/`extras` only; `story`/`stories` dead at hub | label≠key |
| 2 | `guide_names` | `qasim` alias | only `qassim`; `qasim` dead here (works at hub) | cross-room |
| 3 | hub/josh/ali | `ja3ffar`/`ja3ffer` | accepted only in bader/bevil/hasan | synonym gap |
| 4 | `smobb` | `m.a.v.i.s`/`m.a.v.i.s.` | only `mavis`; dotted forms dead here | synonym gap |
| 5 | `extra` | `recap` "works at any menu" (`guide.ron:26`) | not keyed in `extra` | prose≠key |
| 6 | `josh` | `end7312` delivered here (`josh.ron:67`) | not keyed in `josh` | prose≠key |
| 7 | `qassim` | `!save` (universal) | missing in `qassim` only | synonym gap |
| 8 | hub | `play`/`pl` shortcut | prints help only | low |
| 9 | engine | `NONRECORD` lists `options`,`journal`,`j`,`look`,`?` | not wired as hub commands | dead refs |
| 10 | `CHARACTERS` vs `CHARACTER_NAMES` (`engine.rs:12-16`) | `qasim` in names list | absent from `CHARACTERS` (visited tracking) | parallel-list drift |
| 11 | guide/hub help | `recap`/`skipmode`/`interval` typed commands | empty/no-op targets; not intercepted by frontends | help lies |
| 12 | `ui.rs:242` | `[F2] instant` hint | no mention of typed `skipmode`/`sm` equivalents | discoverability |

### 3.2 Frontend parity

Both frontends route all key input through the same `app.rs`, so function-key handling (F1/F2/F3, Tab, Space, Shift, arrows, Enter, Esc, Ctrl-C) is identical — **no TUI↔web drift found**. The gap is that **neither** frontend intercepts the typed meta-commands (`recap`/`rp`, `skipmode`/`sm`) that the content branches leave as empty `Steps([])` and that help text promises. Confirm exact interception points during implementation.

### 3.3 Gate inventory (cryptic codes)

| Code | Kind | Sites | Treatment |
|---|---|---|---|
| `<---------->` | breach event | branches in ali/bader/bevil/hasan/josh; noise lines `ali:26,266 bader:190 bevil:129,168 josh:123,155 hasan:208` | → `Gate` (signpost replaces the blank prompt; noise line kept as flavor) |
| `end7312` | ending trigger | `hub`, `ali`; delivered in `josh` but not keyed there | → `Gate` (`on_deploy:[Goto("ending")]`); also key it in `josh` |
| `k1ll` / `kill` | puzzle command | ali/bader/hasan/josh/qassim | → `Gate` after reviewing exact semantics during planning |
| `7549009`, `7789980` | earned passwords (entry) | many locations | registry + surfaced in options/journal as discovered; literal type stays; `Gate`/"use password" affordance optional |
| `9285005`, `!save`, `!break`, `!help` | secret / cheat | various | **unchanged** — hidden, exact-typed, never auto-offered or labelled |
| Jaffar hex lines | already event-gated by `UnlockDecode` | engine `run()` | **unchanged**; breach-failure hex taunts relocate into gate `on_missing`/`on_decline` |

## 4. Architecture

Three layers, each an independently shippable phase.

### Phase 1 — Canonical command registry + normalization

- **Single source of truth.** A registry mapping `canonical_id → { label, aliases: [..] }`, covering: navigable sections, meta-commands, the affirmative set (`yes/y/yeah/yep/yup/...`) and negative set (`no/n/nah/...`), and character-name variants (`qassim/qasim`, `mavis` + dotted forms, `jaffar` + obfuscations). This registry is the only place vocabulary is declared.
- **Normalization.** Keep `trim().to_lowercase()`; add internal-dot collapse so `m.a.v.i.s.` ≡ `mavis` automatically. Punctuation-insensitivity applies to **navigation/affirmative tokens only**, never to secret or literal-code matching.
- **Resolution.** `handle_command` and the `Prompt` matcher resolve a normalized token to its canonical id via the registry, then match against branch canonicals. `display_name`, `structural_label`, and `options_view` derive their labels from the **same** registry, structurally preventing label↔key drift.
- **Secrets bypass.** Secret/cheat codes are matched by exact equality outside the registry and never appear in any label/options surface.
- **Outcome.** All of §3.1 (1–10) resolve as a consequence: a section's label is, by construction, one of its accepted aliases; an alias declared on a canonical is accepted in every location that exposes that canonical.

### Phase 2 — State-conditional primitive + `Gate` step

In `crates/smobb-core/src/scene.rs` / `effect.rs`:

- `Condition` enum backed by existing engine state:
  - `Knows(String)` → `discovered.contains`
  - `Visited(String)` → `visited.contains`
  - `Asked(String, String)` → `asked[loc].contains(topic)`
  - `DecodeUnlocked` → `decoding_unlocked`
- `Step::Gate(GateSpec { code: String, signpost: String, on_deploy: Vec<Step>, on_missing: Vec<Step>, on_decline: Vec<Step> })`. Engine handling in `run()`:
  1. Emit the `signpost` line (uniform wording, e.g. `"BREACH DETECTED — deploy prevention? (yes)"`).
  2. Suspend on input (reuse the `pending_prompt` mechanism, extended to carry a gate).
  3. On affirmative **or** the literal `code`: if `Knows(code)` → run `on_deploy`; else → run `on_missing`.
  4. On negative → run `on_decline`.
  5. On anything else → re-emit the signpost and re-await (never silent).
- `Step::If(Condition, then: Vec<Step>, else: Vec<Step>)` — a thin conditional for bespoke state-gated scenes; nearly free once `Condition` exists, and keeps future state-gating from re-introducing hand-rolled drift.
- No visible content change in this phase; covered by unit tests only.

### Phase 3 — Content migration + signposting + help-text truth

- Convert every breach prompt to a single `Gate`; keep the existing random dot/dash noise as a flavor `Line` immediately before the gate.
- Convert other earned gates per §3.3 (`end7312` → `Gate` with `on_deploy:[Goto("ending")]` and key it in `josh`; `k1ll`/`kill` after semantics review).
- `options_view` / journal list earned-but-undeployed gates as concrete actions (e.g. "Deploy breach prevention"); secrets remain hidden.
- Make typed `recap`/`rp`/`skipmode`/`sm` actually fire the same toggles as the F-keys (frontend interception), and either wire or remove the `interval`/`NONRECORD` claims so help text is true (§3.1 #11–12).

## 5. Data flow

```
input
  → normalize (trim, lowercase, collapse dots for nav)
  → secret? → exact match → (hidden) branch
  → registry resolve → canonical token
  → branch match; Gate/Condition steps evaluated against engine state
  → outputs (Line/Recap/Effect/AwaitInput/...)
```

## 6. Error handling

- Unknown navigation input: replace the silent re-prompt (`handle_command` fall-through) with `"I don't understand 'X' — type 'options'."`, optionally with a nearest-match suggestion (navigation tokens only; never suggests secrets).
- `Gate` with the code unknown: always emits the `on_missing` line — never a silent no-op.
- Secret mismatch: unchanged (silent), preserving the easter-egg feel.

## 7. Testing

- **Registry property test:** every `label` in the registry resolves back to a typeable canonical id (label is always typeable). Guards the entire label≠key class.
- **Cross-room test:** each alias is accepted in every location exposing its canonical (kills the qasim/jaffar/mavis class).
- **Gate tests:** `Knows`→`on_deploy`; missing→`on_missing`; decline→`on_decline`; literal code still works; secret codes unaffected by the registry/gate paths.
- **Frontend parity:** typed `skipmode`/`recap` produce the same effect as their F-keys, on both frontends.
- **Regression:** existing `smobb-core` unit tests and `content_*`/`playthrough` tests stay green, updated only where a gate changes a scene's output shape.

## 8. Rollout

Phases ship in order and independently. **Phase 1 alone fixes the reported Stories bug and all of §3.1** — valuable even if work stops there. Phase 2 adds machinery behind tests with no visible change. Phase 3 flips content and help text onto the new machinery.

## 9. Open questions / risks

- `k1ll`/`kill` exact semantics need a read before migration (is it a one-way story trigger or a reversible choice?). Flagged for the implementation plan.
- Whether earned **passwords** (`7549009`, `7789980`) get a full `Gate` "use password" affordance or just registry + options/journal visibility. Default: the lighter treatment; revisit if it still reads as friction.
- Confirm the precise frontend interception points for typed meta-commands during Phase 3 (audit inferred neither frontend intercepts; verify before editing).
