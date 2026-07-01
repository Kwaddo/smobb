# SMOBB Web Clean Framed-Terminal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the SMOBB web build read as a deliberate, clean framed terminal instead of a cheap emulator — fill the viewport (no black void), self-host IBM Plex Mono, add a screen bezel + retuned CRT, restyle the control bar, and support wheel/trackpad/touch scrolling.

**Architecture:** Pure web-side chrome pass on `crates/smobb-web`. The ratatui render (layout, borders, per-speaker colors) is shared code and is NOT touched, so the terminal and desktop frontends are unaffected. All changes are CSS, one HTML file, one JS file, and two font assets. Verification is done by building with `trunk`, serving locally, and measuring/screenshotting the page with the Playwright MCP browser tools.

**Tech Stack:** Rust→WASM via `ratzilla` 0.2 (`DomBackend`), `trunk` bundler, plain CSS + ES module JS, IBM Plex Mono (OFL) woff2.

## Global Constraints

- **Web-only.** Only edit files under `crates/smobb-web/`: `style.css`, `index.html`, `controls.js`, and new font assets. Do NOT modify `smobb-core`, `smobb-tui-core`, `smobb-tui`, or `smobb-desktop`.
- **Colors unchanged.** Do not alter per-speaker text colors; those come from the shared render.
- **Cell metric is fixed by ratzilla:** each character cell MUST render as **10px wide × 19px tall**. Ratzilla computes grid `cols = window.innerWidth / 10`, `rows = window.innerHeight / 19` (`ratzilla-0.2.0/src/utils.rs`). Glyph advance MUST be ≤ 10px (aim ~9.9px) so text never overflows/wraps; line box MUST be exactly 19px so the grid fills the viewport height.
- **No `letter-spacing`** on the grid — it inserts gaps that break box-drawing border continuity. Adjust `font-size` only to hit the advance target.
- **Fonts self-hosted, offline-safe.** IBM Plex Mono Regular + Bold as local woff2. The service worker (`sw.js`) is runtime cache-first and caches same-origin GETs automatically — do NOT modify `sw.js`.
- **Keep working:** effect overlays (`#fx-flash`, `#fx-mood`, `body.fx-glitch`, mood tints), the `body.reduced` / F3 reduced-effects path, `prefers-reduced-motion`, the soft-keyboard bridge, and the Skip hold-to-fast-forward button.
- **No separate OS-style title bar** (would push the grid down and break the 10×19 fill or clip the bottom input row). The bezel + glow carries the "screen" identity; the app's own `S.M.O.B.B.` title line stays the header.

## Development loop (used by every task's verification)

Prerequisites (once): `cargo install trunk`, `rustup target add wasm32-unknown-unknown`.

Start the dev server **once** in the background and reuse it across tasks (it hot-rebuilds on file change):

```bash
cd crates/smobb-web && trunk serve
```

Serves at `http://127.0.0.1:8080`. After editing a file, wait for the rebuild, then verify with the Playwright MCP browser tools:

- `browser_navigate` → `http://127.0.0.1:8080` (or `browser_navigate_back`/reload after a rebuild).
- `browser_evaluate` with the **measurement snippet** below to get objective numbers.
- `browser_take_screenshot` for the aesthetic check.
- `browser_resize` to test multiple window sizes.

**Measurement snippet** (paste as the `function` arg to `browser_evaluate`):

```js
() => {
  const grid = document.getElementById('grid');
  if (!grid) return { error: 'no #grid yet' };
  const r = grid.getBoundingClientRect();
  const span = grid.querySelector('span');
  const doc = document.documentElement;
  return {
    voidPx: Math.round(window.innerHeight - r.bottom), // black gap under grid
    overflowX: doc.scrollWidth - doc.clientWidth,       // >0 means text overflows
    rows: grid.children.length,
    fontFamily: span ? getComputedStyle(span).fontFamily : getComputedStyle(grid).fontFamily,
    cellW: span ? +getComputedStyle(span).width.replace('px','') : null,
  };
}
```

`trunk` targets: `trunk build --release --public-url /smobb/` is the production build used in CI; for local verification a plain `trunk serve` (debug) is fine.

---

### Task 1: Bundle IBM Plex Mono + fix the cell-metric void

Foundational task: without the correct 10×19 cell the grid underfills (void) and text can overflow. The metric depends on the actual font advance, so the font and the metric land together.

**Files:**
- Create: `crates/smobb-web/IBMPlexMono-Regular.woff2`
- Create: `crates/smobb-web/IBMPlexMono-Bold.woff2`
- Modify: `crates/smobb-web/index.html` (add two `copy-file` links)
- Modify: `crates/smobb-web/style.css` (@font-face, font stack, cell metric, zero offsets)

**Interfaces:**
- Produces: a viewport-filling grid rendered in IBM Plex Mono with cells ≈10×19. Later tasks assume `#grid` starts at viewport top-left (no body/grid padding) and fills to near the bottom.

- [ ] **Step 1: Establish the failing baseline**

With the current (unmodified) build running, navigate to `http://127.0.0.1:8080` and run the measurement snippet.
Expected (the bug): `voidPx` is large (roughly `innerHeight * 0.15`, e.g. 100–180 on a tall window) and `fontFamily` is a system mono (e.g. `Menlo`/`monospace`), not IBM Plex Mono. Record the number to confirm the fix later.

- [ ] **Step 2: Download the font assets**

```bash
cd crates/smobb-web
curl -L -o IBMPlexMono-Regular.woff2 \
  "https://cdn.jsdelivr.net/npm/@fontsource/ibm-plex-mono@5/files/ibm-plex-mono-latin-400-normal.woff2"
curl -L -o IBMPlexMono-Bold.woff2 \
  "https://cdn.jsdelivr.net/npm/@fontsource/ibm-plex-mono@5/files/ibm-plex-mono-latin-700-normal.woff2"
# sanity: both files are non-trivial woff2 (tens of KB), not an HTML error page
ls -l IBMPlexMono-*.woff2
file IBMPlexMono-Regular.woff2   # -> "Web Open Font Format (Version 2)"
```

(Box-drawing glyphs `═ ║ ╔ …` are not in the Latin subset and are not in Plex Mono anyway; they resolve via the fallback mono in the stack — verified in Step 6.)

- [ ] **Step 3: Copy the fonts into the bundle**

In `crates/smobb-web/index.html`, add alongside the existing `copy-file` links (after the `icon.svg` copy-file line):

```html
    <link data-trunk rel="copy-file" href="IBMPlexMono-Regular.woff2" />
    <link data-trunk rel="copy-file" href="IBMPlexMono-Bold.woff2" />
```

- [ ] **Step 4: Declare the font and set the font stack**

At the TOP of `crates/smobb-web/style.css`, add:

```css
@font-face {
  font-family: "IBM Plex Mono";
  src: url("IBMPlexMono-Regular.woff2") format("woff2");
  font-weight: 400;
  font-style: normal;
  font-display: swap;
}
@font-face {
  font-family: "IBM Plex Mono";
  src: url("IBMPlexMono-Bold.woff2") format("woff2");
  font-weight: 700;
  font-style: normal;
  font-display: swap;
}
```

Then change the `html, body` font stack (currently `"Cascadia Code", "JetBrains Mono", ...`) to lead with Plex and keep a box-drawing-capable fallback:

```css
  font-family: "IBM Plex Mono", "DejaVu Sans Mono", Menlo, Consolas, monospace;
```

- [ ] **Step 5: Set the 10×19 cell metric and zero the offsets**

Replace the current `#grid` / `#grid pre` rules:

```css
/* ratzilla assumes 10px x 19px cells (cols = innerW/10, rows = innerH/19).
   Match it exactly so the grid fills the viewport and never overflows. */
html, body { margin: 0; padding: 0; }
#grid {
  padding: 0;
  margin: 0;
  font-size: 16.5px;   /* IBM Plex Mono advance ~0.6em -> ~9.9px (<=10) */
  line-height: 19px;   /* exact row height to match rows = innerH/19 */
  white-space: pre;
}
#grid pre { margin: 0; line-height: 19px; }
```

Remove the existing `body { padding-bottom: 56px; }` rule and the `@media (min-width: 800px) { body { padding-bottom: 44px } }` line — the grid must occupy the full window height (the control bar becomes a pure overlay in Task 3). Leave the `#kbd` hidden-input rule as-is.

- [ ] **Step 6: Verify the fix (measure + screenshot)**

Reload `http://127.0.0.1:8080`, run the measurement snippet.
Expected: `voidPx` < 19 (under one row), `overflowX` ≤ 0, `fontFamily` starts with `"IBM Plex Mono"`, `cellW` ≈ 9.9.
Then `browser_take_screenshot` and confirm visually: no black void at the bottom, text is IBM Plex Mono, and the double-line box borders are continuous and aligned (no broken/misplaced border glyphs). If `overflowX` > 0, lower `font-size` by 0.1px increments and re-measure. If borders are misaligned, confirm the fallback family renders them at the same advance (screenshot).

- [ ] **Step 7: Resize check**

Use `browser_resize` to a few sizes (e.g. 800×600, 1440×900, 1920×1200, 500×900). Re-run the snippet at each. Expected at every size: `voidPx` < 19 and `overflowX` ≤ 0 (ratzilla re-inits the grid on resize).

- [ ] **Step 8: Commit**

```bash
cd /Users/delirium/Documents/smobb
git add crates/smobb-web/IBMPlexMono-Regular.woff2 crates/smobb-web/IBMPlexMono-Bold.woff2 \
        crates/smobb-web/index.html crates/smobb-web/style.css
git commit -m "feat(web): self-host IBM Plex Mono and fix ratzilla cell-metric void

Match ratzilla's 10x19 cell assumption (font-size 16.5px, line-height
19px) so the grid fills the viewport with no black void or overflow.
Bundle IBM Plex Mono Regular+Bold as offline woff2."
```

---

### Task 2: Screen dressing — background, bezel, CRT retune

Turn the filled-but-plain grid into a screen: richer dark background, a soft bezel hugging the viewport, and CRT atmosphere strong enough to read as intentional.

**Files:**
- Modify: `crates/smobb-web/index.html` (add one bezel overlay div)
- Modify: `crates/smobb-web/style.css` (background, bezel, retuned scanlines/vignette/glow)

**Interfaces:**
- Consumes: viewport-filling `#grid` from Task 1.
- Produces: `#bezel` decorative overlay (`pointer-events: none`) layered above the grid, below effect overlays.

- [ ] **Step 1: Add the bezel element**

In `crates/smobb-web/index.html`, right after the opening `<body>` / before `#fx-flash`, add:

```html
    <!-- decorative screen bezel (non-interactive) -->
    <div id="bezel" aria-hidden="true"></div>
```

- [ ] **Step 2: Richer background + bezel CSS**

In `style.css`, change the flat `--bg` usage: keep `--bg: #0b0b0b;` (drives `theme-color`) but give `body` a subtle gradient behind the grid:

```css
body {
  background:
    radial-gradient(ellipse at 50% 40%, #14161a 0%, #0b0b0b 70%, #060607 100%);
}

/* soft screen bezel: thin rounded frame + inner glow hugging the viewport */
#bezel {
  position: fixed;
  inset: 6px;
  pointer-events: none;
  z-index: 9200;
  border: 1px solid rgba(120, 140, 160, 0.18);
  border-radius: 10px;
  box-shadow:
    inset 0 0 24px rgba(0, 0, 0, 0.85),
    inset 0 0 2px rgba(160, 200, 220, 0.15),
    0 0 0 1px rgba(0, 0, 0, 0.6);
}
```

- [ ] **Step 3: Retune the CRT layers**

The existing `body::before` (scanlines), `body::after` (vignette), and `#grid span` (glow) currently stop at `bottom: 52px` (leftover control-bar gap). Change all three to `bottom: 0` (control bar is an overlay now) and strengthen slightly:

```css
#grid span { text-shadow: 0 0 5px currentColor; }

body::before {   /* scanlines */
  content: ""; position: fixed; left: 0; right: 0; top: 0; bottom: 0;
  pointer-events: none; z-index: 9000;
  background: repeating-linear-gradient(
    0deg, rgba(0,0,0,0.22) 0 1px, transparent 1px 3px);
}
body::after {    /* vignette */
  content: ""; position: fixed; left: 0; right: 0; top: 0; bottom: 0;
  pointer-events: none; z-index: 8999;
  background: radial-gradient(ellipse at center,
    transparent 50%, rgba(0,0,0,0.6) 100%);
}
```

Delete the now-obsolete `@media (min-width: 800px) { body::before, body::after { bottom: 40px; } }` rule (they are full-height now). Leave the `#fx-flash, #fx-mood` overlays' `bottom` at `0` too (change their `bottom: 52px` / media `bottom: 40px` to `0`).

- [ ] **Step 4: Verify effects still gate**

Reload. `browser_take_screenshot`: confirm bezel frames the screen, background has subtle depth, scanlines/vignette read as deliberate (not washing out text). Then confirm the reduced path still works: run `browser_evaluate` →

```js
() => { document.body.classList.add('reduced'); return getComputedStyle(document.getElementById('fx-flash')).opacity; }
```

Expected: `"0"` (reduced kills the flash). Remove the class afterward:

```js
() => { document.body.classList.remove('reduced'); return true; }
```

- [ ] **Step 5: Commit**

```bash
cd /Users/delirium/Documents/smobb
git add crates/smobb-web/index.html crates/smobb-web/style.css
git commit -m "feat(web): add screen bezel, depth background, retuned CRT

Soft rounded bezel + inner glow, subtle radial background, and
stronger scanline/vignette/glow that fill the full viewport now that
the grid does. Reduced-motion/F3 path preserved."
```

---

### Task 3: Restyle control bar + fix bottom-row overlap

The grid now fills the full height, so the fixed control bar overlays the bottom rows (the input `>` and hint line). Restyle it to match the bezel, minimize it on desktop, and keep the game's bottom line visible on touch.

**Files:**
- Modify: `crates/smobb-web/style.css` (control-bar styling + desktop hide)

**Interfaces:**
- Consumes: viewport-filling grid (Task 1), bezel (Task 2).
- Produces: control bar cohesive with the frame; hidden on wide viewports, present and non-occluding on narrow ones.

- [ ] **Step 1: Restyle + desktop-hide the control bar**

Replace the `#controls` block and its `@media (min-width: 800px)` variant:

```css
#controls {
  position: fixed;
  left: 6px; right: 6px; bottom: 6px;   /* inside the bezel */
  height: 44px;
  display: flex;
  gap: 6px;
  padding: 5px;
  background: rgba(18, 20, 24, 0.92);
  border: 1px solid rgba(120, 140, 160, 0.18);
  border-radius: 8px;
  z-index: 9600;                         /* above bezel */
  padding-bottom: calc(5px + env(safe-area-inset-bottom, 0px));
}
#controls button {
  flex: 1 1 0; min-width: 0;
  background: var(--btn);
  color: var(--fg);
  border: 1px solid var(--bar-border);
  border-radius: 6px;
  font: inherit; font-size: 15px;
  cursor: pointer; user-select: none; -webkit-user-select: none;
  touch-action: manipulation;
}
#controls button:active,
#controls button.active { background: var(--btn-active); }

/* desktop keyboards do not need the touch bar */
@media (min-width: 800px) { #controls { display: none; } }
```

- [ ] **Step 2: Verify desktop (bar hidden, nothing occluded)**

Reload at a wide size (`browser_resize` 1440×900). `browser_take_screenshot`: control bar is gone; the bottom hint line (`[Tab] sidebar …`) and input `>` are fully visible above the viewport bottom.

- [ ] **Step 3: Verify touch/narrow (bar present, input visible)**

`browser_resize` to 500×850. `browser_take_screenshot`: control bar visible inside the bezel; confirm it does not cover the input `>` line. If it does, the app's bottom rows sit under it — reduce `#controls height` (e.g. to 40px) and re-check; the bar floats over the vignette margin, and the last game rows should remain readable above it.

- [ ] **Step 4: Commit**

```bash
cd /Users/delirium/Documents/smobb
git add crates/smobb-web/style.css
git commit -m "feat(web): restyle control bar to match bezel, hide on desktop

Bar now floats inside the bezel on touch and is hidden on wide
viewports where a physical keyboard covers its functions. Sized so it
does not occlude the game's input/hint line."
```

---

### Task 4: Wheel + trackpad + touch-swipe scrolling

The app scrolls the focused pane on `ArrowUp`/`ArrowDown` (large jumps on `PageUp`/`PageDown`); `controls.js` already dispatches synthetic `keydown`s on `document`. Add pointer/wheel → arrow-key bridging. No wasm changes.

**Files:**
- Modify: `crates/smobb-web/controls.js` (wheel + touch-swipe listeners)

**Interfaces:**
- Consumes: the existing `dispatchKey(key)` helper in `controls.js`.
- Produces: wheel/trackpad/touch-swipe scroll the currently focused pane.

- [ ] **Step 1: Add wheel/trackpad scrolling**

In `crates/smobb-web/controls.js`, after the control-bar button loop (and reusing the existing `dispatchKey`), add:

```js
// --- wheel / trackpad scrolling -----------------------------------------
// Bridge scroll gestures to the arrow/page keys the app already handles.
// Scrolls whichever pane has focus (story vs sidebar, chosen via <-/-> or Tab).
let wheelAccum = 0;
document.addEventListener(
  "wheel",
  (e) => {
    e.preventDefault(); // stop the page itself from scrolling
    wheelAccum += e.deltaY;
    const STEP = 24; // px per line of scroll
    if (Math.abs(e.deltaY) >= 240) {
      dispatchKey(e.deltaY > 0 ? "PageDown" : "PageUp");
      wheelAccum = 0;
      return;
    }
    while (Math.abs(wheelAccum) >= STEP) {
      dispatchKey(wheelAccum > 0 ? "ArrowDown" : "ArrowUp");
      wheelAccum -= Math.sign(wheelAccum) * STEP;
    }
  },
  { passive: false }
);
```

- [ ] **Step 2: Add touch-swipe scrolling**

Below the wheel handler, add swipe tracking that does not disturb the existing tap-to-focus-keyboard `pointerdown` handler or the `#controls` buttons:

```js
// --- touch-swipe scrolling ----------------------------------------------
let swipeY = null;
document.addEventListener(
  "pointerdown",
  (e) => {
    if (e.pointerType === "touch" && !e.target.closest("#controls")) {
      swipeY = e.clientY;
    }
  },
  { passive: true }
);
document.addEventListener(
  "pointermove",
  (e) => {
    if (swipeY === null || e.pointerType !== "touch") return;
    const SWIPE = 24; // px per scroll line
    let dy = swipeY - e.clientY; // finger up => scroll down
    while (Math.abs(dy) >= SWIPE) {
      dispatchKey(dy > 0 ? "ArrowDown" : "ArrowUp");
      dy -= Math.sign(dy) * SWIPE;
      swipeY -= Math.sign(swipeY - e.clientY) * SWIPE;
    }
  },
  { passive: true }
);
document.addEventListener("pointerup", () => { swipeY = null; });
document.addEventListener("pointercancel", () => { swipeY = null; });
```

- [ ] **Step 3: Verify wheel scrolling scrolls the focused pane**

Reload. The story pane starts focused. Capture a screenshot, then scroll up with the wheel and re-screenshot:

```js
// simulate a wheel-up burst, then report — run via browser_evaluate
() => {
  document.dispatchEvent(new WheelEvent("wheel", { deltaY: -120, bubbles: true, cancelable: true }));
  return "sent wheel up";
}
```

Better: use the Playwright mouse wheel if available, or `browser_take_screenshot` before/after real trackpad scroll. Expected: the transcript scrolls up (older lines appear) and the page itself does NOT scroll (no scrollbar movement). Press Tab / ArrowRight to focus the sidebar and confirm the wheel then scrolls the Journal/Options pane instead.

- [ ] **Step 4: Verify page does not hijack-scroll**

Run the Task 1 measurement snippet after scrolling. Expected: `voidPx` unchanged (< 19), `overflowX` ≤ 0 — the document did not gain a scroll offset.

- [ ] **Step 5: Commit**

```bash
cd /Users/delirium/Documents/smobb
git add crates/smobb-web/controls.js
git commit -m "feat(web): wheel, trackpad, and touch-swipe scrolling

Bridge scroll gestures to the Arrow/Page keydowns the app already
handles, scrolling the focused pane. Page-level scroll suppressed."
```

---

### Task 5: Production build + release-parity verification

Confirm the real CI build path works and the result is coherent end to end.

**Files:** none (verification only).

- [ ] **Step 1: Production build**

```bash
cd /Users/delirium/Documents/smobb/crates/smobb-web
trunk build --release --public-url /smobb/
```
Expected: builds with no errors; `dist/` contains `IBMPlexMono-Regular.woff2`, `IBMPlexMono-Bold.woff2`, hashed css/js/wasm, `controls.js`, `manifest.json`, `sw.js`, `icon.svg`.

- [ ] **Step 2: Serve the production bundle and full visual pass**

Serve `dist/` under the `/smobb/` path (matching production), e.g.:

```bash
cd /Users/delirium/Documents/smobb/crates/smobb-web
python3 -m http.server 8081 --directory . >/dev/null 2>&1 &
# then open http://127.0.0.1:8081/dist/  (public-url /smobb/ expects a /smobb/ mount;
# for a quick check trunk serve --release also works)
```

Navigate with the Playwright tools and confirm: no void, Plex Mono, bezel + CRT, control bar hidden on desktop / present on narrow, wheel + swipe scroll. Compare against the existing reference `docs/superpowers/smobb-web-smoke.png`; if the team keeps a smoke reference, capture a fresh screenshot and update it.

- [ ] **Step 3: Sanity — shared frontends untouched**

```bash
cd /Users/delirium/Documents/smobb
git diff --name-only main...web-clean-terminal
```
Expected: only files under `crates/smobb-web/` and `docs/superpowers/`. No changes to `smobb-core`, `smobb-tui-core`, `smobb-tui`, `smobb-desktop`.

- [ ] **Step 4: Commit any updated reference screenshot** (if applicable)

```bash
git add docs/superpowers/smobb-web-smoke.png
git commit -m "docs(web): refresh web smoke screenshot after redesign"
```

---

## Self-Review

**Spec coverage:**
- Void fix / 10×19 metric → Task 1. ✓
- Self-hosted IBM Plex Mono Regular+Bold, offline → Task 1 (sw.js unchanged: runtime cache-first, verified). ✓
- Full-bleed framed screen: bezel, background, CRT retune → Task 2. ✓
- Control bar restyle + bottom-overlap fix + desktop minimize → Task 3. ✓
- Wheel/trackpad/touch scrolling of focused pane, no page hijack → Task 4. ✓
- Colors unchanged; shared render untouched → Global Constraints + Task 5 Step 3 check. ✓
- Testing (build, screenshot, resize, scroll, mobile, reduced-motion, CI parity) → per-task verifies + Task 5. ✓
- Title-strip decision (dropped to preserve fill) → Global Constraints, matches spec's "decide during implementation". ✓
- Box-drawing fallback risk → Task 1 Steps 2 + 6 verify alignment. ✓
- Floating-window stretch → out of scope, not planned. ✓

**Placeholder scan:** No TBD/TODO; all CSS/JS shown in full; commands and expected outputs concrete. ✓

**Type/name consistency:** `dispatchKey` reused from existing `controls.js`; key names (`ArrowUp`/`ArrowDown`/`PageUp`/`PageDown`/`Tab`) match `to_key` in `smobb-web/src/main.rs`; `#grid`, `#bezel`, `#controls`, `#fx-flash`, `#fx-mood`, `body.reduced` consistent across tasks. ✓
