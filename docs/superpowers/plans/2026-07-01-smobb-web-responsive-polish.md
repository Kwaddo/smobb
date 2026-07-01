# SMOBB Web Responsive Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the ratzilla/WASM in-browser build of S.M.O.B.B. responsive and clean on phone, iPad/tablet, desktop, and installed PWA, keeping the CRT/framed-terminal look.

**Architecture:** Almost all work is web-side chrome (`style.css`, `index.html`, `controls.js`, `manifest.json`, icons) plus one width-gated change to the shared render `crates/smobb-tui-core/src/ui.rs`. The ratzilla grid is sized by the library from the window (desktop) or physical screen (mobile); we cannot resize the grid element, so mobile uses a bottom-anchored, bar-reserving layout rather than fighting ratzilla. CSS classes are split so the UA-based cell metric stays correct while touch affordances key off capability.

**Tech Stack:** Rust (ratatui/ratzilla, WASM via `trunk`), vanilla JS/CSS, playwright-core headless chromium for screenshot verification.

**Design spec:** `docs/superpowers/specs/2026-07-01-smobb-web-responsive-polish-design.md`

## Global Constraints

- **No auto-commit (user preference).** Where a step says "Commit", **stage** the changes (`git add`) and pause for the user to commit. Do NOT run `git commit` unless the user has explicitly approved committing in this session.
- Web crate is `crates/smobb-web`. Shared render is `crates/smobb-tui-core/src/ui.rs`. Do NOT touch `smobb-core`, `smobb-tui`, or `smobb-desktop`.
- `ratzilla = "0.2"`. Cell metric is UA-branched: desktop rows = `inner_height/20`, mobile rows = `screen.height/19`, cols = `width/10`. CSS `line-height` MUST match ratzilla's UA branch (20px desktop, 19px mobile) — the class driving the metric must stay UA-based (`is_mobile()` = UA contains "mobile"/"tablet").
- Keep the CRT look: per-cell `#grid span { text-shadow }` glow, scanlines, vignette, effect overlays (`#fx-flash`, `#fx-mood`, `fx-glitch`, mood tints) must keep working. Keep `prefers-reduced-motion` and `body.reduced` (F3) escape hatches intact.
- Local verification builds are **root-relative** (`trunk build`, no `--public-url`) so they serve at `http://localhost:8099/`. The deploy build uses `trunk build --release --public-url /smobb/` (do not commit its output).
- Font `font-size: 16.5px` and the desktop `line-height: 20px` / mobile `19px` cell metric are load-bearing — do not change them.

---

### Task 1: Verification harness + baseline screenshots

Sets up the headless-chromium screenshot tool every later task uses, and captures the current build as the "before" baseline.

**Files:**
- Create: `crates/smobb-web/dev-shot.mjs` (dev-only, gitignored)
- Modify: `.gitignore`
- Create (generated): `crates/smobb-web/.shots/before-*.png`

**Interfaces:**
- Produces: `node crates/smobb-web/dev-shot.mjs <prefix>` — builds nothing, loads `http://localhost:8099/`, writes `crates/smobb-web/.shots/<prefix>-<viewport>.png` for 6 viewports: `desktop-1440`, `desktop-narrow-1024`, `ipad-portrait-820`, `phone-390`, `phone-landscape-844`, `phone-small-360`. Blocks service workers so rebuilds are never served stale.

- [ ] **Step 1: Add gitignore entries**

Append to `.gitignore`:

```
# smobb-web dev verification (not shipped)
crates/smobb-web/dev-shot.mjs
crates/smobb-web/.shots/
crates/smobb-web/node_modules/
```

- [ ] **Step 2: Install playwright-core in the web crate**

Run:
```bash
cd crates/smobb-web && npm init -y >/dev/null 2>&1; npm install playwright-core >/dev/null 2>&1 && echo installed
```
Expected: `installed`. (The headless chromium binary is already present under `~/Library/Caches/ms-playwright/chromium_headless_shell-*`.)

- [ ] **Step 3: Write the screenshot harness**

Create `crates/smobb-web/dev-shot.mjs`:

```js
import { chromium } from 'playwright-core';
import fs from 'fs';
import os from 'os';
import path from 'path';

const prefix = process.argv[2] || 'shot';
const URL = 'http://localhost:8099/';
const OUT = path.join(process.cwd(), '.shots');
fs.mkdirSync(OUT, { recursive: true });

// locate the already-installed headless chromium shell
const base = path.join(os.homedir(), 'Library/Caches/ms-playwright');
const dir = fs.readdirSync(base).filter(d => d.startsWith('chromium_headless_shell-')).sort().pop();
const execPath = path.join(base, dir, 'chrome-headless-shell-mac-arm64/chrome-headless-shell');

const PHONE_UA = 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1';
const IPAD_UA = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15';

const viewports = [
  { name: 'desktop-1440', w: 1440, h: 900 },
  { name: 'desktop-narrow-1024', w: 1024, h: 720 },
  { name: 'ipad-portrait-820', w: 820, h: 1180, ua: IPAD_UA, touch: true },
  { name: 'phone-390', w: 390, h: 844, ua: PHONE_UA, touch: true, mobile: true },
  { name: 'phone-landscape-844', w: 844, h: 390, ua: PHONE_UA, touch: true, mobile: true },
  { name: 'phone-small-360', w: 360, h: 640, ua: PHONE_UA, touch: true, mobile: true },
];

const browser = await chromium.launch({ executablePath: execPath });
for (const v of viewports) {
  const ctx = await browser.newContext({
    viewport: { width: v.w, height: v.h },
    userAgent: v.ua,
    hasTouch: !!v.touch,
    isMobile: !!v.mobile,
    deviceScaleFactor: 2,
    serviceWorkers: 'block',
  });
  const page = await ctx.newPage();
  await page.goto(URL, { waitUntil: 'load' });
  await page.waitForTimeout(2500); // wasm boot + first frame
  await page.screenshot({ path: `${OUT}/${prefix}-${v.name}.png` });
  console.log('shot', v.name);
  await ctx.close();
}
await browser.close();
```

- [ ] **Step 4: Build current web (root-relative) and serve it**

Run:
```bash
cd crates/smobb-web && trunk build && (pkill -f "http.server 8099" 2>/dev/null; true) && (python3 -m http.server 8099 --directory dist >/tmp/smobb-serve.log 2>&1 &) && sleep 1 && echo served
```
Expected: `served`. Leave this server running for all later tasks.

- [ ] **Step 5: Capture the baseline**

Run:
```bash
cd crates/smobb-web && node dev-shot.mjs before
```
Expected: `shot desktop-1440` … `shot phone-small-360` (6 lines). Confirm 6 PNGs exist in `crates/smobb-web/.shots/`.

- [ ] **Step 6: Commit (stage + pause per no-commit rule)**

```bash
git add .gitignore
# dev-shot.mjs and .shots/ are gitignored by design
```
Stage `.gitignore`; report to the user that the harness is ready. Do not `git commit` unless approved.

---

### Task 2: Width-aware hint in shared render (`ui.rs`)

The only shared-render change. Extracts the bottom hint into a width-gated helper so narrow widths (phones, narrow terminals) drop it instead of overflowing. Full TDD in Rust.

**Files:**
- Modify: `crates/smobb-tui-core/src/ui.rs` (hint construction in `render`, ~lines 276-297; add helper near `banner_lines`)
- Test: `crates/smobb-tui-core/src/ui.rs` (`#[cfg(test)] mod tests`)

**Interfaces:**
- Produces: `pub fn hint_for_width(width: u16, reduced_effects: bool) -> Option<String>` — full hint at `width >= 90`, short hint at `55..90`, `None` below 55. `render` calls it with `input_area.width`.

- [ ] **Step 1: Write the failing tests**

Add to the `tests` module in `crates/smobb-tui-core/src/ui.rs`:

```rust
#[test]
fn hint_full_at_wide_width() {
    let h = super::hint_for_width(140, false).expect("wide has hint");
    assert!(h.contains("[F2] instant"));
    assert!(h.contains("[Esc] quit"));
    assert!(h.contains("[F3] effects"));
}

#[test]
fn hint_reduced_flag_flips_f3_label() {
    let h = super::hint_for_width(140, true).unwrap();
    assert!(h.contains("[F3] effects:OFF"));
}

#[test]
fn hint_short_at_medium_width() {
    let h = super::hint_for_width(80, false).expect("medium has hint");
    assert!(!h.contains("[F2] instant"), "medium drops secondary keys");
    assert!(h.contains("[Esc] quit"));
}

#[test]
fn hint_absent_at_narrow_width() {
    assert!(super::hint_for_width(37, false).is_none());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p smobb-tui-core hint_ -- --nocapture`
Expected: FAIL — `cannot find function hint_for_width in module super`.

- [ ] **Step 3: Add the helper**

Insert into `crates/smobb-tui-core/src/ui.rs` above `pub fn render` (after `banner_lines`):

```rust
/// Bottom-row hint scaled to available width. `None` when too narrow to be
/// useful (phones / narrow terminals), so the input row shows only the prompt.
pub fn hint_for_width(width: u16, reduced_effects: bool) -> Option<String> {
    let fx = if reduced_effects {
        "[F3] effects:OFF"
    } else {
        "[F3] effects"
    };
    if width >= 90 {
        Some(format!(
            "[Tab] sidebar  [<-/-> focus]  [hold Space] fast  [F2] instant  {}  ['leave' exits]  [Esc] quit",
            fx
        ))
    } else if width >= 55 {
        Some("[Tab] sidebar  [hold Space] fast  [Esc] quit".to_string())
    } else {
        None
    }
}
```

- [ ] **Step 4: Use the helper in `render`**

In `crates/smobb-tui-core/src/ui.rs`, replace the input/hint block at the end of `render` (the `let fx_label = …` through the final `f.render_widget(input, input_area);`) with:

```rust
    // input line + width-aware hint
    let mut spans = vec![RSpan::styled(
        format!("> {}", app.input()),
        Style::default().fg(RColor::Rgb(220, 220, 220)),
    )];
    if let Some(hint) = hint_for_width(input_area.width, app.reduced_effects()) {
        spans.push(RSpan::styled(
            format!("    {}", hint),
            Style::default()
                .fg(RColor::Rgb(110, 110, 110))
                .add_modifier(Modifier::DIM),
        ));
    }
    let input = Paragraph::new(Line::from(spans));
    f.render_widget(input, input_area);
```

- [ ] **Step 5: Run tests + fmt + clippy**

Run:
```bash
cargo test -p smobb-tui-core && cargo fmt -p smobb-tui-core -- --check && cargo clippy -p smobb-tui-core -- -D warnings
```
Expected: all tests pass (including the 4 new + existing `renders_title_story_and_sidebar` etc.), fmt clean, clippy clean.

- [ ] **Step 6: Commit (stage + pause)**

```bash
git add crates/smobb-tui-core/src/ui.rs
```
Report; do not commit unless approved.

---

### Task 3: Split detection — capability-based `is-touch`

Fixes the iPad/touch bug: control bar and soft-keyboard key off capability, cell metric stays UA-based.

**Files:**
- Modify: `crates/smobb-web/controls.js`
- Modify: `crates/smobb-web/style.css`

**Interfaces:**
- Produces: `html.is-touch` toggled by capability; `#controls` and the soft-keyboard bridge gate on `is-touch`. `html.is-mobile` (UA) stays and drives only `#grid` line-height.

- [ ] **Step 1: Add the capability class in `controls.js`**

In `crates/smobb-web/controls.js`, directly below the existing `is-mobile` UA block, add:

```js
// Touch AFFORDANCES (control bar, soft keyboard) key off capability, NOT UA.
// iPadOS reports a desktop UA, so is-mobile (UA) would hide the bar; is-touch
// (capability) shows it. is-mobile stays UA-only for the cell metric, which
// must mirror ratzilla's own UA is_mobile() row divisor.
const IS_TOUCH =
  (window.matchMedia && window.matchMedia("(pointer: coarse)").matches) ||
  navigator.maxTouchPoints > 0 ||
  "ontouchstart" in window;
if (IS_TOUCH) {
  document.documentElement.classList.add("is-touch");
}
```

- [ ] **Step 2: Gate the soft-keyboard focus on `is-touch`**

The soft-keyboard `pointerdown` handler already checks `e.pointerType === "touch"`, so it is capability-correct and needs no change. Leave it. (No edit in this step — verify by reading `controls.js` that the `kbd.focus()` handler keys on `pointerType === "touch"`.)

- [ ] **Step 3: Switch the control-bar CSS gate to `is-touch`**

In `crates/smobb-web/style.css`, change:

```css
html.is-mobile #controls { display: flex; }
```
to:
```css
html.is-touch #controls { display: flex; }
```

Leave `html.is-mobile #grid, html.is-mobile #grid pre { line-height: 19px; }` unchanged.

- [ ] **Step 4: Rebuild, screenshot, verify**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs t3
```
Read `.shots/t3-ipad-portrait-820.png`, `.shots/t3-desktop-1440.png`, `.shots/t3-phone-390.png`. Expected:
- iPad: control bar now VISIBLE (was absent in `before-ipad-portrait-820.png`), no black void (metric still /20).
- Desktop: NO control bar.
- Phone: control bar still visible.

- [ ] **Step 5: Commit (stage + pause)**

```bash
git add crates/smobb-web/controls.js crates/smobb-web/style.css
```
Report; do not commit unless approved.

---

### Task 4: One frame — remove bezel border, kill the shimmer

De-duplicates the frame (CSS bezel border vs ratatui box-border) and removes the dashed shimmer from the bezel edge interacting with scanlines.

**Files:**
- Modify: `crates/smobb-web/style.css`

**Interfaces:**
- Produces: `#bezel` as a borderless monitor treatment (glow + vignette + rounded-corner mask). No new selectors consumed elsewhere.

- [ ] **Step 1: Replace the `#bezel` rule**

In `crates/smobb-web/style.css`, replace the entire `#bezel { … }` block with:

```css
/* soft monitor treatment: NO border line (the ratatui box-border is the only
   frame). Rounded-corner mask + outer glow + edge vignette read as a screen. */
#bezel {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9200;
  border-radius: 12px;
  box-shadow:
    inset 0 0 90px 20px rgba(0, 0, 0, 0.55),
    inset 0 0 3px rgba(160, 200, 220, 0.10);
}
/* round the actual screen corners so the grid reads as a CRT, not a rectangle */
html, body { border-radius: 12px; }
#grid { border-radius: 12px; }
```

- [ ] **Step 2: Retune the scanlines so they no longer shimmer on the (now absent) border**

In `crates/smobb-web/style.css`, in the `body::before` scanline rule, soften the gradient — change the `background` to:

```css
  background: repeating-linear-gradient(
    0deg,
    rgba(0, 0, 0, 0.16) 0 1px,
    transparent 1px 3px
  );
```

- [ ] **Step 3: Rebuild, screenshot, verify**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs t4
```
Read `.shots/t4-desktop-1440.png` and `.shots/t4-phone-390.png`. Expected:
- Single frame only (the ratatui box-border); no second rounded border line hugging the same edge.
- No dashed/dotted shimmer along the screen edges (compare to `before-phone-390.png`).
- Title ` S.M.O.B.B. ` on the top border no longer collides with a bezel corner.
- Effect overlays unaffected (screen still legible, vignette present).

- [ ] **Step 4: Commit (stage + pause)**

```bash
git add crates/smobb-web/style.css
```
Report; do not commit unless approved.

---

### Task 5: Bottom-anchor the grid on touch + reserve bar space + title strip; drop ↑↓

Fixes the real-device off-screen input: on touch the (screen-sized) grid is bottom-anchored above a reserved bar height, and a slim HTML title strip fills the clipped top.

**Files:**
- Modify: `crates/smobb-web/index.html`
- Modify: `crates/smobb-web/style.css`

**Interfaces:**
- Consumes: `html.is-touch` (Task 3).
- Produces: CSS var `--bar-h`; `#titlebar` element shown only on touch; `↑`/`↓` buttons removed from `#controls`.

- [ ] **Step 1: Add the title strip and remove ↑↓ in `index.html`**

In `crates/smobb-web/index.html`, add immediately after the opening `<body>` (before `#bezel`):

```html
    <!-- touch-only title strip: fills the top the ratatui title clips into
         when the grid is bottom-anchored (see style.css). -->
    <div id="titlebar" aria-hidden="true">
      <span class="dots">&#9679;&#9679;&#9679;</span>
      <span class="name">S.M.O.B.B.</span>
    </div>
```

In the same file, delete these two buttons from `<nav id="controls">`:

```html
      <button type="button" data-key="ArrowUp" title="Scroll up">&#8593;</button>
      <button type="button" data-key="ArrowDown" title="Scroll down">&#8595;</button>
```

(Wheel + touch-swipe scrolling in `controls.js` already cover scrolling; the generic `data-key` loop handles the remaining buttons, so no JS change is needed for the removal.)

- [ ] **Step 2: Add the layout variables and bottom-anchor rule in `style.css`**

In `crates/smobb-web/style.css`, add to `:root`:

```css
  --bar-h: 52px; /* control-bar height incl. padding; grid reserves this on touch */
```

Add after the `#grid` rules:

```css
/* On touch, ratzilla sizes the grid to the PHYSICAL screen (taller than the
   visible viewport), so the input row would fall below the fold. Bottom-anchor
   the grid above the reserved control bar so the prompt + newest lines stay
   visible; the oversized top (old lines, top border) clips upward. */
html.is-touch #grid {
  position: fixed;
  left: 0;
  right: 0;
  bottom: calc(var(--bar-h) + env(safe-area-inset-bottom, 0px));
}
```

- [ ] **Step 3: Style the title strip in `style.css`**

Add to `crates/smobb-web/style.css`:

```css
/* title strip: only on touch, where the ratatui title clips off-top */
#titlebar { display: none; }
html.is-touch #titlebar {
  display: flex;
  align-items: center;
  gap: 10px;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: calc(28px + env(safe-area-inset-top, 0px));
  padding: env(safe-area-inset-top, 0px) 12px 0;
  z-index: 9300; /* above grid, below effect overlays (9500+) */
  background: linear-gradient(180deg, rgba(10, 12, 15, 0.96), rgba(10, 12, 15, 0));
  font-size: 13px;
  color: rgba(180, 200, 210, 0.75);
  pointer-events: none;
}
#titlebar .dots { color: rgba(120, 140, 160, 0.55); letter-spacing: 2px; }
#titlebar .name { letter-spacing: 3px; font-weight: 700; }
```

- [ ] **Step 4: Rebuild, screenshot, verify**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs t5
```
Read `.shots/t5-phone-390.png`, `.shots/t5-phone-landscape-844.png`, `.shots/t5-ipad-portrait-820.png`, `.shots/t5-desktop-1440.png`. Expected:
- Phone/iPad: `● ● ● S.M.O.B.B.` strip across the top; control bar has 6 buttons (no ↑↓); input `>` row sits just above the bar with clear space.
- Desktop: NO title strip (ratatui title still on the top border), grid unchanged, no bar.

- [ ] **Step 5: Commit (stage + pause)**

```bash
git add crates/smobb-web/index.html crates/smobb-web/style.css
```
Report; do not commit unless approved.

---

### Task 6: Control-bar restyle + safe-area

Makes the bar cohesive with the bezel, translucent with a scrim, bigger targets, inside the PWA safe area.

**Files:**
- Modify: `crates/smobb-web/style.css`

**Interfaces:**
- Consumes: `--bar-h` (Task 5). Keep `#controls` total height (incl. padding) ≈ `--bar-h` so the reserve math stays correct.

- [ ] **Step 1: Replace the `#controls` and button rules**

In `crates/smobb-web/style.css`, replace the `#controls { … }` block and the `#controls button { … }` / `:active` rules with:

```css
/* touch control bar: cohesive with the bezel, translucent, inside safe area */
#controls {
  position: fixed;
  left: 8px;
  right: 8px;
  bottom: calc(6px + env(safe-area-inset-bottom, 0px));
  height: 44px;
  display: none;
  gap: 8px;
  padding: 4px;
  background: linear-gradient(180deg, rgba(16, 19, 24, 0.72), rgba(10, 12, 16, 0.92));
  border: 1px solid rgba(120, 140, 160, 0.16);
  border-radius: 12px;
  z-index: 9600;
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
}
html.is-touch #controls { display: flex; }
#controls button {
  flex: 1 1 0;
  min-width: 0;
  background: rgba(34, 38, 44, 0.9);
  color: var(--fg);
  border: 1px solid rgba(120, 140, 160, 0.14);
  border-radius: 8px;
  font: inherit;
  font-size: 15px;
  cursor: pointer;
  user-select: none;
  -webkit-user-select: none;
  touch-action: manipulation;
  text-shadow: 0 0 4px rgba(160, 200, 220, 0.25);
}
#controls button:active,
#controls button.active {
  background: rgba(70, 90, 105, 0.95);
  border-color: rgba(160, 200, 220, 0.4);
}
```

Leave the `body.reduced #controls button[data-key="F3"]` rule intact (update its color to `rgba(70,90,105,0.95)` if it referenced the old `--btn-active`).

- [ ] **Step 2: Rebuild, screenshot, verify**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs t6
```
Read `.shots/t6-phone-390.png` and `.shots/t6-phone-small-360.png`. Expected:
- Bar reads as one cohesive piece with the frame; 6 evenly-sized buttons; no overflow on the 360px width; input row still clear above the bar.

- [ ] **Step 3: Commit (stage + pause)**

```bash
git add crates/smobb-web/style.css
```
Report; do not commit unless approved.

---

### Task 7: Re-key sidebar auto-hide to column count

Replaces the raw `max-width: 700px` px query with a column-based heuristic so the phone auto-hides the sidebar consistently with ratzilla's actual columns, while iPad keeps it.

**Files:**
- Modify: `crates/smobb-web/controls.js`

**Interfaces:**
- Consumes: nothing new.
- Produces: F1 auto-dispatch after `TrunkApplicationStarted` when estimated columns < 64.

- [ ] **Step 1: Replace the responsive auto-hide block**

In `crates/smobb-web/controls.js`, replace:

```js
// --- responsive: start with the sidebar hidden on phones -----------------
if (window.matchMedia("(max-width: 700px)").matches) {
  // ratzilla mounts asynchronously; nudge after it's listening.
  window.addEventListener("TrunkApplicationStarted", () =>
    setTimeout(() => dispatchKey("F1"), 100)
  );
}
```
with:

```js
// --- responsive: hide the sidebar when there aren't enough columns -------
// ratzilla lays out ~10px cells, so columns ~= width/10. Below ~64 cols the
// 70/30 split gets cramped (phones), so start with the sidebar hidden. Uses
// the same width source ratzilla sizes from (screen on touch, window else).
const estCols = ((IS_TOUCH ? window.screen.width : window.innerWidth) || 0) / 10;
if (estCols < 64) {
  window.addEventListener("TrunkApplicationStarted", () =>
    setTimeout(() => dispatchKey("F1"), 100)
  );
}
```

(`IS_TOUCH` is defined in Task 3.)

- [ ] **Step 2: Rebuild, screenshot, verify**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs t7
```
Read `.shots/t7-phone-390.png` (sidebar hidden, story full width) and `.shots/t7-ipad-portrait-820.png` (sidebar still shown). Expected as described.

- [ ] **Step 3: Commit (stage + pause)**

```bash
git add crates/smobb-web/controls.js
```
Report; do not commit unless approved.

---

### Task 8: PWA icons + manifest + apple-touch-icon

Adds sized/maskable PNG icons (rasterized from `icon.svg` via the already-installed headless chromium) and wires them into the manifest and HTML.

**Files:**
- Create: `crates/smobb-web/dev-gen-icons.mjs` (dev-only, gitignored)
- Create: `crates/smobb-web/icon-192.png`, `icon-512.png`, `icon-512-maskable.png`
- Modify: `crates/smobb-web/manifest.json`
- Modify: `crates/smobb-web/index.html`

**Interfaces:**
- Produces: three PNG icons referenced by `manifest.json` and `index.html`.

- [ ] **Step 1: Gitignore the generator**

Append to `.gitignore`:
```
crates/smobb-web/dev-gen-icons.mjs
```
(The PNG icons themselves are shipped assets — do NOT gitignore them.)

- [ ] **Step 2: Write the icon generator**

Create `crates/smobb-web/dev-gen-icons.mjs`:

```js
import { chromium } from 'playwright-core';
import fs from 'fs';
import os from 'os';
import path from 'path';

const base = path.join(os.homedir(), 'Library/Caches/ms-playwright');
const dir = fs.readdirSync(base).filter(d => d.startsWith('chromium_headless_shell-')).sort().pop();
const execPath = path.join(base, dir, 'chrome-headless-shell-mac-arm64/chrome-headless-shell');

const svg = fs.readFileSync('icon.svg', 'utf8');
const targets = [
  { file: 'icon-192.png', size: 192, pad: 0 },
  { file: 'icon-512.png', size: 512, pad: 0 },
  { file: 'icon-512-maskable.png', size: 512, pad: 64 }, // safe-zone padding for maskable
];

const browser = await chromium.launch({ executablePath: execPath });
for (const t of targets) {
  const ctx = await browser.newContext({ deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const inner = t.size - t.pad * 2;
  await page.setViewportSize({ width: t.size, height: t.size });
  await page.setContent(
    `<html><body style="margin:0;background:#0b0b0b;width:${t.size}px;height:${t.size}px;
      display:flex;align-items:center;justify-content:center;">
      <div style="width:${inner}px;height:${inner}px;">${svg}</div></body></html>`
  );
  await page.waitForTimeout(150);
  await page.screenshot({ path: t.file, clip: { x: 0, y: 0, width: t.size, height: t.size } });
  console.log('icon', t.file);
  await ctx.close();
}
await browser.close();
```

- [ ] **Step 3: Generate the icons**

Run:
```bash
cd crates/smobb-web && node dev-gen-icons.mjs
```
Expected: `icon icon-192.png` / `icon icon-512.png` / `icon icon-512-maskable.png`. Confirm the three PNGs exist and are non-empty.

- [ ] **Step 4: Update `manifest.json`**

Replace the `"icons"` array in `crates/smobb-web/manifest.json` with:

```json
  "icons": [
    { "src": "icon.svg", "sizes": "any", "type": "image/svg+xml", "purpose": "any" },
    { "src": "icon-192.png", "sizes": "192x192", "type": "image/png", "purpose": "any" },
    { "src": "icon-512.png", "sizes": "512x512", "type": "image/png", "purpose": "any" },
    { "src": "icon-512-maskable.png", "sizes": "512x512", "type": "image/png", "purpose": "maskable" }
  ]
```

- [ ] **Step 5: Copy icons into the bundle + link apple-touch-icon in `index.html`**

In `crates/smobb-web/index.html`, add next to the other `copy-file` links:

```html
    <link data-trunk rel="copy-file" href="icon-192.png" />
    <link data-trunk rel="copy-file" href="icon-512.png" />
    <link data-trunk rel="copy-file" href="icon-512-maskable.png" />
```

And replace the existing `apple-touch-icon` line with a sized PNG (iOS ignores SVG for the home-screen icon):

```html
    <link rel="apple-touch-icon" href="icon-192.png" />
```

- [ ] **Step 6: Rebuild + verify manifest and assets**

Run:
```bash
cd crates/smobb-web && trunk build && ls dist/icon-192.png dist/icon-512.png dist/icon-512-maskable.png && python3 -c "import json;json.load(open('manifest.json'));print('manifest ok')"
```
Expected: the three icon files listed under `dist/`, and `manifest ok`.

- [ ] **Step 7: Commit (stage + pause)**

```bash
git add crates/smobb-web/manifest.json crates/smobb-web/index.html crates/smobb-web/icon-192.png crates/smobb-web/icon-512.png crates/smobb-web/icon-512-maskable.png
```
Report; do not commit unless approved.

---

### Task 9: Full verification sweep + release build

Final gate: full CI-equivalent checks, after/before screenshot comparison across all viewports, and the real-device manual checklist for the user.

**Files:**
- None modified (verification only). May produce `crates/smobb-web/.shots/after-*.png`.

- [ ] **Step 1: Run the workspace CI checks**

Run:
```bash
cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
Expected: all clean/pass. If any fail, fix in the owning task's files and re-run.

- [ ] **Step 2: Release web build (deploy config, do not commit output)**

Run:
```bash
cd crates/smobb-web && trunk build --release --public-url /smobb/
```
Expected: clean build into `dist/`. (Deploy is handled by `pages.yml` on push; the local `dist/` is gitignored.)

- [ ] **Step 3: Rebuild root-relative + capture the "after" set**

Run:
```bash
cd crates/smobb-web && trunk build && node dev-shot.mjs after
```
Read each `.shots/after-<viewport>.png` against `.shots/before-<viewport>.png`. Confirm:
- Desktop: one frame, no shimmer, title readable, no void, no horizontal overflow.
- iPad portrait: control bar present, sidebar readable, no void.
- Phone (390 / small-360 / landscape): title strip, 6-button bar, input `>` clear above the bar, no hint overflow.

- [ ] **Step 4: Confirm reduced-motion / F3 still work**

Add a one-off check: in `dev-shot.mjs`-style manual run, load with `newContext({ reducedMotion: 'reduce' })` (or press F3 via `page.keyboard.press('F3')`) and confirm the grid still renders and motion is suppressed. Document the result in the task report.

- [ ] **Step 5: Produce the real-device manual checklist for the user**

Report this checklist for the user to run on a real phone + installed PWA (headless cannot reproduce `screen > viewport`):
- Input `>` and newest lines visible above the bar; nothing clipped below the fold.
- Tap raises the soft keyboard; typed text appears at the prompt; Return submits.
- Swipe scrolls the focused pane; Skip fast-forwards; Tab/≡ switch/toggle the sidebar.
- Rotate portrait↔landscape: grid re-fills, no void/overflow.
- Install to Home Screen: offline load works; frame, title strip, and bar clear the notch and home indicator.

- [ ] **Step 6: Stop the dev server + final stage**

Run:
```bash
pkill -f "http.server 8099" 2>/dev/null; true
```
Stage any remaining shipped changes for the user; report completion with the before/after screenshots and the manual checklist. Do not commit unless approved.

---

## Self-Review

**Spec coverage:**
- Spec §A (split detection) → Task 3. ✓
- Spec §B (one frame + shimmer) → Task 4. ✓
- Spec §C (bottom-anchor + reserve + touch title strip) → Task 5. ✓
- Spec §D (control-bar refine, drop ↑↓, safe-area) → Tasks 5 (drop ↑↓) + 6 (restyle/safe-area). ✓
- Spec §E (width-aware hint in ui.rs) → Task 2. ✓
- Spec §F (sidebar auto-hide re-key) → Task 7. ✓
- Spec §G (PWA icons/manifest/safe-area) → Task 8 (icons/manifest) + Tasks 5/6 (safe-area insets). ✓
- Spec §H (screenshot harness + real-device checklist) → Task 1 (harness/baseline) + Task 9 (sweep + checklist). ✓

**Placeholder scan:** No TBD/TODO; every code step shows the actual code. Thresholds (90/55 cols, 64 cols, `--bar-h: 52px`) are concrete with rationale. ✓

**Type/name consistency:** `hint_for_width(width, reduced_effects) -> Option<String>` defined in Task 2 and called there. `IS_TOUCH` defined Task 3, reused Task 7. `is-touch` class set in Task 3, consumed by Tasks 5/6. `--bar-h` defined Task 5, consumed Task 6. `#titlebar` created Task 5 HTML + styled Task 5 CSS. ✓
