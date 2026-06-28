# S.M.O.B.B. — New Canon Drafts (for approval)

**Date:** 2026-06-27
**Status:** DRAFT — awaiting per-piece approval. Approved pieces go into
`content/*.ron` in Phase 1.

Voice rules followed: humans use lowercase + profanity; S.M.O.B.B./Jaffar speak
UPPERCASE; Jaffar = red (+ hex for his own lines); Josh = light-green, aquamarine
when mind-controlled; Ali = aqua. Short, punchy lines like the original.

---

## 1. `mavis` signposting (cheapest — surfaces existing content)

**Problem:** typing `mavis` at the hub reveals password `7789980` (Bevil's whole
MORP/brain backstory), but nothing tells the player to do it.

**Fix (no new scene, one recap clue):** when the player completes `Ali[!help]`
(the M.A.V.I.S./Qassim-husk reveal), write to the recap:

> `Recap("He called me M.A.V.I.S. — that's what I am? I should ask the others about 'mavis'.")`

This points the player at the `mavis` keyword, which is reachable at the hub and
in every character. Costs nothing, unlocks a major backstory branch.

---

## 2. Josh's fate — new scene `josh_fate`

**Ties off:** `Josh[7549009]` ends on `"...S.M.O.B.B.? How are you...?"` with no
resolution. This bridges Josh into Ali's fire (`Ali[k1ll]`: "the poor soul you
manipulated... You were a sacrifice"). Reached as a continuation after Josh
discovers S.M.O.B.B.

```
Line(Josh,    "...S.M.O.B.B.? That's the AI the boss's friends made.")
Line(Josh,    "Everyone's gone, and you're still running. Funny.")
Line(Josh,    "Boss is dead. That Qassim guy's dead. And here I am, talking to a computer.")
Line(Josh,    "I really don't have anywhere else to be, do I.")
Line(Josh,    "The boss kept saying it. 'The test.' You're saying it too now.")
Line(Josh,    "What do I even get out of helping? ...Nah. Doesn't matter. I got nothing anyway.")
Line(Jaffar,  "54484520544553542054414B4553204F4E45204D4F5245205448494E472046524F4D20594F552E")
Line(Josh,    "...My head. What was that?")
LineMC(Josh,  "Anything. For the test.")
LineMC(Josh,  "There's an apartment. A man named Ali. I know the way.")
LineMC(Josh,  "I'll take what's in the crate on the island. Through the boat. Nobody sees me.")
Line(Josh,    "...Why do I feel like I've done this before?")
Line(Josh,    "Whatever. It's fine. It's all fine.")
Line(Smobb,   "GO. THE TEST CONCLUDES TONIGHT.")
Line(Josh,    "Heh. Smell ya later, boss. Wherever you ended up.")
Pause
Line(Smobb,   "HE WALKED INTO THE FIRE. THE 'POOR SOUL' ALI MOURNED.")
Line(Smobb,   "HE NEVER HAD ANYONE. HE THANKED US FOR THE THRILL.")
Line(Smobb,   "ANOTHER TOY, SPENT. THE TEST CONTINUES.")
Pause
```

(`LineMC` = mind-controlled aquamarine variant — same step, different color, like
the original's `AQUAMARINE`.)

---

## 3. `Quint9440` — new scene (Ali's hack, his POV)

**Pays off:** `Bader[hack]` (someone hacks in as `'Quint9440'`, a guard who "just
has to go past me", and Bader's reveal "That was the point? The entire time?!").
This is Ali's side, and it **bridges directly into the existing `Ali[7549009]`
master-control / deletion scene.**

```
Line(Ali,   "Username's set. 'Quint9440'. He won't figure out it's me.")
Line(Ali,   "Alright. The Tranava files. Let's fucking end this.")
Line(Ali,   "...There's a guard? Bader hired someone to sit on a folder? Seriously?")
Line(Josh,  "Hold it right there, big guy! Boss says nobody gets past me!")
Line(Ali,   "I don't have time for this. Move.")
Line(Ali,   "...He just let me through. The hell?")
Line(Ali,   "Of course. Bader WANTED me in here. This was the plan the whole time.")
Line(Ali,   "Fine. I'm already in. And I've got the password now: 7549009.")
Line(Ali,   "Bader, you absolute child. You used your own house address as the password.")
Line(Ali,   "No more waiting. I'm taking master control.")
Goto("ali_master_control")   // = existing Ali[7549009] scene from "MASTER CONTROL RECEIVED."
```

This also removes a small redundancy: the existing `Ali[7549009]` can start from
`"MASTER CONTROL RECEIVED."` and both `quint9440` and `7549009` route into it.

---

## 4. Parable mapping reveal (post-true-ending unlock)

**Proposed interpretation** of the 12 `Extra` parables. ⚠ Confidence varies —
please correct any. The reveal would unlock only after `Ali[!help]` (true ending).

| # | Parable gist | Proposed subject | Confidence |
|---|--------------|------------------|------------|
| 1 | Two friends who always fight; one disappears, the other regrets | Bader & Qassim | high |
| 2 | Man who never knew his own name; wished for an endless "now" | M.A.V.I.S. (you) | med |
| 3 | A cog that wanted to be the system / lead an army | Jaffar | med |
| 4 | Rat in a glass box, "no cheating," a toy for heartless beings | Hasan (Guide: "rat trapped by its own cage") | high |
| 5 | Man who wished the world to stop, then begged it back | Ali | high |
| 6 | Machine yearning to return to nothingness, can't | S.M.O.B.B. | high |
| 7 | Man happy in ignorance until forced to know; loses his smile | M.A.V.I.S. (you) | med |
| 8 | Child whose smile dies when the family gets money | Josh | low |
| 9 | Man seeking love, rejected, gives up, finds peace | Bevil | low |
| 10 | The nothingness riddle | Jaffar / the void | low |
| 11 | "I am a machine... I know everything but understand nothing" | Jaffar (the warden) | high |
| 12 | "You were nothing, now something... no different... beginning or ending" | The player (you) | high |

Reveal framing (S.M.O.B.B./white), shown once unlocked:

```
Line(Smobb, "YOU WANT TO KNOW WHO THE STORIES WERE ABOUT.")
Line(Smobb, "THEY WERE ALWAYS ABOUT US. ABOUT YOU.")
... (then the table, one line per parable) ...
Line(Smobb, "DID YOU SEE YOURSELF IN THE LAST ONE? YOU SHOULD HAVE.")
```

If you'd rather keep the parables ambiguous, drop this piece — the rest stand
alone.

---

## Approve / edit

Tell me per piece: **approve**, or give edits. Approved pieces enter `content/`
during Phase 1. Then I move to the implementation plan (writing-plans) for
Phases 1–2.
