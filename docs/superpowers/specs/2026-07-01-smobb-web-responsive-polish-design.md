# SMOBB Web — Responsive Polish

**Date:** 2026-07-01
**Scope:** `crates/smobb-web` (`style.css`, `index.html`, `controls.js`, `manifest.json`, icon assets) plus **one** width-gated change to the shared render `crates/smobb-tui-core/src/ui.rs`. No changes to `smobb-core`, `smobb-tui`, or `smobb-desktop`. Per-speaker colors and the ratatui layout are otherwise untouched.

## Goal

Make the in-browser game (`https://kwaddo.github.io/smobb/`, ratzilla/WASM) feel responsive and clean on **phones, iPad/tablets, desktop, and the installed PWA**. Keep the existing CRT/framed-terminal aesthetic; refine it into one cohesive frame rather than redesigning it.

## Locked decisions

- Target: polish the existing ratzilla in-browser game (not a landing site, not a new DOM frontend).
- Depth: web-side chrome + **width-aware** tweaks to shared `ui.rs` that trigger **only on narrow widths** (terminal/desktop unaffected at normal sizes).
- Devices: phone (iOS Safari / Android Chrome, portrait + landscape), iPad/tablet, desktop (mouse + keyboard, wide/ultrawide), installed PWA (offline, safe-area).
- Rework level: **refine the chrome** — cohesive frame, fix bugs, keep the look. Not a full adaptive font/column overhaul.
- Mobile grid anchoring: **bottom-anchor** (input + newest text always visible; top title clips off, replaced by an HTML title strip on touch).
- Include the width-aware hint change in shared `ui.rs`.
- **No auto-commit** (user preference): deliver files; the user commits.

## Root-cause facts (verified in ratzilla 0.2.0 source)

`ratzilla-0.2.0/src/utils.rs` + `src/backend/utils.rs`:

```rust
// DomBackend rebuilds its buffer from get_sized_buffer():
let size = if is_mobile() { get_screen_size() } else { get_window_size() };
// get_window_size() = (inner_width / 10, inner_height / 20)   // desktop
// get_screen_size() = (screen.width / 10, screen.height / 19) // mobile (PHYSICAL display)
// is_mobile() = navigator.userAgent contains "mobile" || "tablet"
```

Consequences that drive the design:

1. **Cell metric is UA-branched.** Desktop rows = `inner_height/20`; mobile rows = `screen.height/19`. The CSS `line-height` must match ratzilla's *own UA branch* (20px desktop, 19px mobile) or a black void / overflow returns. → The cell-metric CSS class MUST stay UA-based (mirroring `is_mobile()`), independent of touch capability.
2. **Mobile sizes to the physical screen, not the viewport.** `screen.height` > `window.innerHeight` (browser URL bar), so on a real phone the grid is taller than the visible viewport: the bottom rows (input `>`, bottom border) render **below the fold**. (Headless-emulated `screen ≈ viewport` hides this; a real device shows it.)
3. **Window/screen sizing, not element sizing.** DomBackend ignores container size; it cannot be told to render into a smaller box in 0.2 without a fork. The canvas/webgl backends *do* size from the element, but they render to a bitmap (no per-cell `<span>`), which would break the CSS phosphor-glow/scanline look — **out of scope** (keeps the look).
4. DomBackend re-inits on `resize` (`add_on_resize_listener` sets `initialized=false`), so a correct metric re-fills the grid across resizes/orientation automatically.

## Problems being fixed (observed on current build)

| # | Problem | Evidence |
|---|---------|----------|
| P1 | iPad / touch laptop gets **no control bar** and mis-uses affordances | UA `is_mobile()` false on iPadOS 13+ (reports desktop Safari) → `#controls` hidden |
| P2 | **Double framing**: CSS `#bezel` border line + ratatui box-border; title `S.M.O.B.B.` collides with the bezel corner | desktop/iPad screenshots |
| P3 | Bezel border **shimmers into a dashed line** against the scanline gradient on high-DPI | phone/iPad screenshots |
| P4 | On real phones the **input `>` is below the fold** and/or under the control bar | ratzilla `get_screen_size()` (fact #2) |
| P5 | Shared **hint line overflows / truncates** on narrow widths and is useless on touch | phone screenshot (`[Tab] sidebar [<-/-> focus]` cut off) |
| P6 | Control bar carries **redundant `↑ ↓`** (swipe + wheel already scroll); input prompt is a weak affordance | screenshots + `controls.js` |
| P7 | Sidebar at 30% **wraps awkwardly** when columns are few (iPad portrait, landscape phone) | iPad/landscape screenshots |
| P8 | PWA: single SVG icon, no sized PNG/`apple-touch-icon`; safe-area only partially handled | `manifest.json`, `index.html` |

## Design

### A. Split detection (fixes P1)

The current single `is-mobile` UA class does two unrelated jobs. Split them:

- **Cell-metric class — keep UA-based, unchanged.** Keep `document.documentElement.classList.add("is-mobile")` gated on `/mobile|tablet/i.test(navigator.userAgent)`, and keep `html.is-mobile #grid { line-height: 19px }`. This MUST mirror ratzilla's UA `is_mobile()` (fact #1). Do not change its trigger.
- **Affordance class — new, capability-based.** Add `is-touch` when `matchMedia("(pointer: coarse)").matches || navigator.maxTouchPoints > 0 || "ontouchstart" in window`. Gate the control bar and soft-keyboard behavior on `is-touch` (was `is-mobile`).

Result: iPad (UA=desktop → correct /20 metric, no void) **and** touch → shows the control bar. Touch laptops likewise. Pure-mouse desktop: neither class beyond metric; no bar.

### B. One frame + kill the shimmer (fixes P2, P3)

- Remove the `#bezel` **border line** (`border: 1px solid …`) — the element doubling the ratatui border and producing the dashed shimmer.
- `#bezel` becomes a pure monitor treatment: rounded-corner mask over the screen + outer glow + edge vignette (inner box-shadow), `pointer-events: none`, below effect overlays. Apply the corner rounding so the grid's sharp corners read as a CRT.
- The single visible frame is the ratatui box-border (authentic, carries the ` S.M.O.B.B. ` title) on desktop.
- Retune scanline (`body::before`) opacity so it reads cleanly over the border without shimmer; keep `prefers-reduced-motion` / `body.reduced` escape hatches and the F3 toggle working.

### C. Bottom-anchor the grid + reserve bar space + touch title strip (fixes P4)

On `is-touch`:

- `#grid { position: fixed; left: 0; right: 0; bottom: var(--bar-h); }` — bottom-anchor the (screen-sized) grid so the input `>` and newest lines sit just above the control bar. The oversized top (title, top border, oldest lines) overflows above the viewport and is clipped by `body { overflow: hidden }`. The story pane auto-scrolls to newest, so no meaningful content is lost.
- `--bar-h` = control-bar height + `env(safe-area-inset-bottom)`. The bar never overlaps content because the grid bottom stops at `--bar-h`.
- Because the ratatui title is now clipped off-top on touch, add a **slim HTML title strip** at the top, shown only on `is-touch`: `● ● ●  S.M.O.B.B.` styled to match the bezel. No doubling (the ratatui title is off-view on touch). `pointer-events: none` except where decorative.

Desktop (non-touch): grid stays window-sized (fits the viewport exactly, no void), top-anchored, ratatui title visible, no HTML strip, no bar — unchanged from today.

Uniform touch rule keeps iPad correct too: its window-sized grid is bottom-anchored above the bar; only the top title row clips, filled by the HTML strip.

### D. Control-bar refinement (fixes P6)

- Buttons: `≡ (F1 sidebar) · Tab · Skip (hold) · Inst (F2) · FX (F3) · ⏎ (Enter)`. **Remove `↑ ↓`** (swipe + wheel scroll already exist in `controls.js`).
- Restyle to match the bezel: same border/radius/background family, phosphor-tinted active state, translucent with a bottom scrim gradient. Larger touch targets (min ~40px height).
- Bottom padding `calc(base + env(safe-area-inset-bottom))`; the bar sits inside the safe area in standalone PWA.
- `#b-skip` hold-to-fast-forward and the `beforeinput`→keydown soft-keyboard bridge behavior are preserved (re-gated to `is-touch`).

### E. Width-aware hint — only shared-render change (fixes P5)

In `ui.rs::render`, gate the bottom hint on `input_area.width` (equivalently `f.area().width`):

- **Wide** (≥ ~90 cols): current full hint — desktop/terminal unchanged.
- **Medium** (~55–90 cols): a shortened hint (core keys only, e.g. `[Tab] sidebar  [Esc] quit`).
- **Narrow** (< ~55 cols): **no hint** — render just `> {input}`.

Purely width-driven (not frontend-specific): a narrow real terminal benefits identically; normal terminal/desktop sizes are unaffected. Thresholds are tunable during implementation; add a unit test in `ui.rs` asserting the hint is absent at a narrow width and present at a wide width.

### F. Sidebar on narrow (fixes P7)

- Keep phone-portrait auto-hide, but re-key the `controls.js` heuristic from a raw `max-width: 700px` px query to the actual usable column count (or `is-touch` + a width test consistent with ratzilla's column math), dispatching `F1` once after `TrunkApplicationStarted` when columns are too few for a comfortable 70/30 split.
- iPad portrait / landscape (enough columns): keep the sidebar visible; no shared-layout change. The 30/70 split and its wrapping are accepted at these widths (out of scope to re-flow the shared layout for this pass).

### G. PWA polish (fixes P8)

- Add sized PNG icons (e.g. 192, 512, plus a maskable 512) generated from `icon.svg`; reference them in `manifest.json` and add `apple-touch-icon` links in `index.html`. Keep the SVG as `any`.
- Keep the network-first-for-stable-entries service worker (`sw.js`) as-is; verify the new icon assets are cached appropriately (fingerprinted → cache-first).
- Safe-area: `viewport-fit=cover` already set; ensure bezel, title strip, and control bar all respect `env(safe-area-inset-*)` so nothing clips under the notch/home indicator in standalone mode.

## Out of scope

- Switching ratzilla backends (canvas/webgl) or forking it for element-based sizing.
- Responsive font-size / column clamping and orientation-specific relayouts (the "full adaptive" option, declined).
- Any change to per-speaker colors, story content, save format, or the ratatui layout beyond the width-gated hint.
- A landing/marketing site or a new DOM play frontend.

## Verification

Reusable headless-chromium screenshot harness (in scratchpad) across six viewports — `desktop-1440`, `desktop-narrow-1024`, `ipad-portrait-820`, `phone-390`, `phone-landscape-844`, `phone-small-360` — captured before and after each change.

Automated / screenshot checks:
1. `cd crates/smobb-web && trunk build --release --public-url /smobb/` builds clean; `cargo fmt --check`, `clippy -D warnings`, `cargo test` pass (the new `ui.rs` hint test included).
2. Desktop: single frame (no doubled border, no dashed shimmer), title readable, no void, no horizontal overflow, box-drawing borders continuous.
3. iPad portrait: control bar present (P1), correct metric (no void), sidebar readable.
4. Phone portrait/landscape/small: control bar (6 buttons, no `↑↓`), title strip on top, input `>` visible above the bar, no hint overflow.
5. Resize/orientation: grid re-fills, no void, no overflow.
6. `prefers-reduced-motion` and F3 (`body.reduced`) still suppress motion.

**Real-device manual checklist** (headless cannot reproduce `screen > viewport`, fact #2):
- On a real phone: input `>` and newest lines visible above the bar; nothing clipped below the fold; soft keyboard raises on tap; typed text appears at the prompt; return submits; swipe scrolls the focused pane; Skip fast-forwards; orientation flip re-fills cleanly.
- Installed PWA: offline load works; frame/bar clear of the notch and home indicator.

## Files touched

- `crates/smobb-web/controls.js` — `is-touch` capability class; gate controls/soft-keyboard on it; drop `↑↓` handlers; re-key sidebar auto-hide; keep wheel/swipe/skip/SW.
- `crates/smobb-web/style.css` — split `is-mobile`/`is-touch` rules; remove bezel border (glow/vignette/rounded mask); bottom-anchor grid + `--bar-h` on touch; title-strip styling; control-bar restyle + safe-area; scanline retune.
- `crates/smobb-web/index.html` — title-strip markup; drop `↑↓` buttons; `apple-touch-icon` links; icon `copy-file` links.
- `crates/smobb-web/manifest.json` + new PNG icon assets — sized/maskable icons.
- `crates/smobb-tui-core/src/ui.rs` — width-aware hint + unit test (only shared change).
