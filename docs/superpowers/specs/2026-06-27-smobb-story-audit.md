# S.M.O.B.B. — Story & Content Audit (Phase 0)

**Date:** 2026-06-27
**Scope:** the Java source in `src/com/company/` (12 files, ~2,300 lines).
**Mandate:** identify defects, content gaps, and missing story; auto-fix clear
defects, flag larger arcs for author decision. (No new canon invented here.)

Line numbers refer to the original Java files.

---

## 1. Defects to AUTO-FIX (objective, unambiguous)

These are clear errors. They will be corrected in-voice during Phase 1 content
extraction. None change meaning.

| # | File:Line | Issue | Fix |
|---|-----------|-------|-----|
| 1 | `Hasan.java:225` | Package name leaks into dialogue: `com.company.Bader` | `Bader` |
| 2 | `Hasan.java:89` | Cut-off sentence: `"I'm sorry that ."` | `"I'm sorry that it had to come to this."` (minimal in-voice; see §6) |
| 3 | `Ali.java:63` | typo `runied` | `ruined` |
| 4 | `Ali.java:246` | typo `mutliple` | `multiple` |
| 5 | `Ali.java:155` | typo `completedly` | `completely` |
| 6 | `Ali.java:129` | typo `downard` | `downward` |
| 7 | `Ali.java:59` | grammar `decided take action` | `decided to take action` |
| 8 | `Ali.java:285` | missing verb `I HIGH HOPES FOR YOU` | `I HAVE HIGH HOPES FOR YOU` |
| 9 | `Bader.java:180` | typo `beacuse` | `because` |
| 10 | `Bevil.java:169` | name truncated `tell li` | `tell Ali` |
| 11 | `Bevil.java:159` | grammar `as many help` | `as much help` |
| 12 | `Bevil.java:141` | stray bracket `]That's all` | `That's all` |
| 13 | `Josh.java:33` | typo `mean much to be but` | `mean much to me but` |
| 14 | `Guide.java:82` | typo `[REDATCTED]` | `[REDACTED]` |

> Note: stylized spellings are intentional and KEPT — `Go0_dby3, m0n2t3r`
> (Ali:69), the `⬛⬛⬛⬛⬛` redacted name (Ali:285), all hexadecimal Jaffar lines.

---

## 2. Defects to FLAG (need an author ruling — NOT auto-fixed)

| # | File:Line | Issue | Why flagged |
|---|-----------|-------|-------------|
| A | `Guide.java:93` vs `Ending.java:20` | Qassim's surname: **"Qassim Mohammed Hameed"** vs **"Qassim Hujair"** | Continuity break. Which is canon? Pick one. |
| B | Spelling of the AI: `Jaffar` / `Ja3ffar` / `Ja3ffer` used interchangeably (Hasan, Bader, Bevil, Ali) | Decide a canonical display spelling, or confirm the drift is intentional (humans typing a name they half-know). |
| C | `Bader.java:106` | `"calm down chibdi!"` — `chibdi` is unknown slang/name | Keep, define, or replace? |
| D | `Bader.java:116` | The "unlimited bacon but no games, or unlimited games but no games" gag says "no games" twice | Intentional absurd meme, or typo for "but no games"? |
| E | `Ali.java:68` | `"I will fall to shove you in deeper"` | Awkward; may be intentional. Reword? |
| F | `Ending.java:61-78` | **Both** the save path and the die path write the identical `!SAVE` recap line | Likely intended (ability always unlocks), but the two endings are near-identical — confirm. |

---

## 3. Mechanic bugs (fixed by the Rust port, not text)

1. **Recap self-destructs.** `recap.txt` is opened in **truncate** mode
   (`new FileWriter("recap.txt")`, no append flag) at the top of 7 methods —
   `Hasan:11`, `Bevil:9`, `Josh:10`, `Josh:149`, `Qassim:10`, `Ending:12`,
   `Main:145`. Entering any of these scenes **wipes the entire notebook** before
   (maybe) writing one line. The clue economy erases itself. **This is the single
   biggest experience bug.**
2. **Recap reader is broken.** `Main:37` / `Guide:125` allocate
   `new String[10000000]` and read `truerecap.txt` inside `for (i=0; i<Recap.length; i++)`,
   reassigning the array each iteration and printing one element — guaranteed to
   throw `NoSuchElementException` / behave nonsensically.
3. **Swing popups break the terminal.** `Guide.PlayGuide()` and `Main` `play`
   open `JOptionPane` PNG dialogs — unavailable headless and on web.
4. All scene loops are `do/while(true)` exited only by `end`. Fine; ports to a
   clean state machine.

> The port replaces the notebook with a proper append + dedup store and an
> in-app viewer (TUI: file; web: `localStorage`). This directly serves goal #2.

---

## 4. Clue economy — reachability map (the game IS solvable)

The story is non-linear (the player is a "god" replaying recordings in a loop).
Progress is gated by passwords learned in earlier scenes. Verified chain:

```
START (hub) ─ type: hasan → ali ............ teaches 7549009  (also Bevil[hasan], Josh[BaderJosh])
  7549009 ─ unlocks Tranava-file scenes (Bader/Qassim/Josh/Bevil)
  Qassim[7549009]  (death recording) ........ teaches 9285005
    9285005 ─ unlocks S.M.O.B.B.-file scenes (Smobb/Bader/Bevil/Hasan/Josh)
    Josh[9285005] ............................ teaches K1LL
      K1LL ─ unlocks earpiece scenes (Bader/Ali/Hasan/Qassim/Josh)
      Josh[k1ll] .............................. teaches end7312
        end7312 ─ unlocks the Ending
          Ending (save/die) ................... unlocks !save
            Main[!save] ........................ unlocks !break
              Main[!break] ..................... unlocks !help
                Ali[!help] ─ the M.A.V.I.S./Qassim-husk reveal (true ending beat)

Side chains:
  breach code '<---------->' ← Bevil[breach], Bevil[hasan], Hasan[bevil], Josh[hasan]
  7789980 (Bevil's private farewell) ← Main[mavis]   ⚠ see gap §5.1
```

**No dead ends found in the main chain** — every gating password is taught by a
reachable earlier scene.

---

## 5. Narrative gaps & missing story (ask #3 — FLAGGED for your decision)

These are candidates to fill. Nothing here is invented yet; each is a hole to
either resolve in-canon or confirm as intentional ambiguity.

### 5.1 Discoverability gaps (player can get stuck even though content exists)
- **`mavis` keyword at the hub is unsignposted.** Typing `mavis` at the hub
  reveals password `7789980`, which unlocks Bevil's biggest exposition scene
  (`Bevil[7789980]`, the MORP/brain-project backstory). But nothing tells the
  player to type `mavis`. Without it, a major chunk of backstory is unreachable.
- **Topic keywords are unlisted.** `brain`, `plan`, `delete`, `hack`,
  `tranavaloop`, `bomb`, `breach`, `smobb`, `jaffar` are discovered only by
  guessing. A redesigned recap/guide should hint these.

### 5.2 Dangling character arcs (referenced, never resolved on screen)
- **Josh's fate.** `Josh[7549009]` ends on `"Hm? What's this... S.M.O.B.B.?
  How are you...?"` — he discovers the AI, then the thread simply stops. No
  resolution of what happens to him. **Strong candidate for a new scene.**
- **Bevil's fate.** Bevil "leaves" and sends a farewell (`7789980`), but there
  is no on-screen account of where he ends up. Possibly intentional (he ditched);
  confirm or resolve.
- **Bader's death** is reported (Ali tells Bevil his "house exploded") but never
  shown from Bader's side beyond `Bader[7549009]` (the self-destruct apology).
  May be complete; confirm.

### 5.3 Named-but-unplayable handles (set up, never paid off)
- **`Quint9440`** (`Bader.java:134`) — Ali's hacker username. Mentioned in
  `Bader[hack]`, but is never usable as a command/password anywhere. A
  `quint9440` scene (Ali's hack from his own POV) is a clean candidate.
- **`gnu8305`** (`Guide:76`, `Bader:103`) — Bevil's alias; Bader sends a
  recording "to gnu8305". The recording's contents are never directly shown
  (Bevil[7789980] is the closest payoff). Candidate to surface that recording.

### 5.4 Structural / scaffolding gap
- The non-linear loop has **no chronology aid**. The recap was meant to be that
  scaffold, but it's broken (§3). Fixing recap + adding an optional timeline view
  is the highest-leverage "cleaner experience" change (goal #2). Not new story —
  new presentation of existing story.

### 5.5 The 12 parable "stories" (`Extra`) — unmapped
The 12 abstract parables clearly shadow specific characters (e.g. #4 "rat in the
glass" ↔ the Guide's "rat trapped by its own cage" = Hasan; #6/#11 "machine
yearning / lost in itself" ↔ S.M.O.B.B./M.A.V.I.S.; #12 "you" ↔ the player).
There is no in-game mapping. **Optional:** an unlockable "who is this about?"
reveal after the true ending. Flag — could be left deliberately ambiguous.

---

## 6. The one auto-fill that touches authoring (Hasan:89)

Context (Hasan, talking to Bevil after a prevented breach):
> "...Qassim's gone, but he was able to help me."
> "Thank you, if only you were still here."
> **"I'm sorry that ."**  ← cut off
> "If it doesn't, it's over. For us. The group is split..."

Proposed minimal completion: **"I'm sorry that it had to come to this."**
This is the smallest in-voice repair that fits the surrounding regret. Confirm or
substitute your own line.

---

## 7. Lore notes (for the content files, not defects)

- **S.M.O.B.B.** = **S**teins;Gate, **M**onster, **O**warimonogatari, **B**erserk,
  **B**leach (`Qassim:70-73`) — anime/manga; "O" used "because I needed a vowel".
- **KAMI** (`Bader:74`) — Japanese for "god"; ties to the "god" framing of the
  ending. Jaffar/SMOBB address the player as a god throughout.
- **Speaker → color** (`Bits.java`): Ali=aqua, Bader=salmon (salmon2 = mind-
  controlled), Bevil=gold, Hasan=purple, Josh=light-green (aquamarine = mind-
  controlled), Qassim=silver, S.M.O.B.B.=white, Jaffar=red, M.A.V.I.S.=blue.
  This table ports verbatim into the content layer.
- **Jaffar speaks hexadecimal**; the in-game hint points at an external converter
  (`Bevil:98`). The port should keep the hex verbatim (it's a deliberate puzzle).

---

## 8. Recommendation — what to fix vs. fill

**Do now (Phase 1, already approved):** all of §1 (auto-fix), §6 (Hasan:89).
**Fix in the engine (Phase 2):** all of §3 (recap, reader, Swing → in-terminal).
**Bring back to you for a decision:** §2 (continuity rulings) and §5
(dangling arcs / unused handles / parable mapping). These are the real "missing
story" calls and are yours to make — recommended priority order:
1. §5.1 `mavis` discoverability (cheap, unlocks existing content).
2. §5.2 Josh's fate (clearest dangling arc).
3. §5.3 `Quint9440` scene (best setup-without-payoff).
4. §2.A Qassim surname ruling (one-line continuity fix).

---

## 9. RESOLUTION — all gaps closed (2026-06-28)

Author decisions made; the audit is closed.

| Gap | Decision | Status |
|-----|----------|--------|
| §5.1 `mavis` discoverability | Recap signpost after `Ali[!help]` | ✅ in `content/ali.ron` |
| §5.2 Josh's fate | New `josh_fate` scene | ✅ in `content/josh.ron` |
| §5.2 Bader's death | Already shown — `Bader[7549009]` self-destruct (`SELF DESTRUCTING 3..2..1`) is the explosion Ali reports | ✅ complete as-is |
| §5.2 Bevil's fate | **Deliberately left ambiguous** — the coward who left, fate unknown | ✅ intentional, no content |
| §5.3 `Quint9440` | New scene routing into `ali_master_control` | ✅ in `content/ali.ron` |
| §5.3 `gnu8305` | Payoff is `Bevil[7789980]` (Bader sends → gnu8305 = Bevil) | ✅ already paid off |
| §5.5 parable mapping | Post-true-ending reveal | ✅ in `content/extra.ron` |
| §2.A Qassim surname | No full surname carried into content (first names only) | ✅ moot |
| §2.B Jaffar spelling | Canonical display **"Jaffar"**; `ja3ffar`/`ja3ffer` kept as input aliases | ✅ resolved |
| §2.C–E `chibdi` / bacon gag / "fall to shove" | Kept verbatim as intentional ambiguity | ✅ kept |
| §2.F twin endings | Confirmed intentional (ability always unlocks) | ✅ kept |
