# SMOBB Web — Clean Framed-Terminal Redesign

**Date:** 2026-07-01
**Scope:** `crates/smobb-web` only (`style.css`, `index.html`, `controls.js`, one font asset). No changes to `smobb-core`, `smobb-tui-core`, `smobb-tui`, or `smobb-desktop`. The shared ratatui render (layout, borders, per-speaker colors) is untouched, so the terminal and desktop frontends are unaffected.

## Problem

The deployed web build (`https://kwaddo.github.io/smobb/`) reads like a "cheap emulator":

1. **Black void.** The terminal grid ends ~16% short of the viewport bottom, leaving a large dead black area.
2. Grid floats top-left with no framing or centering.
3. Flat `#0b0b0b` background; scanline/vignette/glow too faint to read as intentional.
4. Per-machine system monospace = inconsistent, undesigned look.
5. Generic gray touch-button strip.
6. Mouse wheel / trackpad / touch-swipe do not scroll; only the on-screen ↑/↓ buttons and arrow keys do.

## Root cause of the void (verified in ratzilla 0.2 source)

`ratzilla-0.2.0/src/utils.rs` and `src/backend/utils.rs`:

```rust
// desktop
get_window_size() = (inner_width / 10, inner_height / 19)
```

Ratzilla's `DomBackend` computes the grid as **cols = window_width / 10px** and **rows = window_height / 19px** — it hard-assumes every character cell is **10px wide × 19px tall**. The current `style.css` uses the browser-default 16px font with `line-height: 1.0`, so real cells are ~9.6px × ~16px. Ratzilla allocates `inner_height/19` rows, but each row only paints ~16px, so the painted grid ends ~16% above the viewport bottom → the void.

Two consequences that constrain the design:

- **The browser window *is* the terminal screen.** Sizing derives from `window.inner_width/height`, not from any container element. Ratzilla cannot be told (in 0.2) to render into a smaller centered box without patching/forking it. Therefore the redesign uses a **full-bleed framed screen**, not a floating window with margins.
- Ratzilla sizes to the **full** window height, ignoring `body` padding. Once the void is fixed, the bottom rows will slide under the fixed control bar unless we account for it (see §D).

Ratzilla re-inits the grid on `resize` (`add_on_resize_listener` sets `initialized = false`), so a correct cell metric keeps the grid filled across window sizes automatically.

## Design decisions (locked with user)

- **Layout:** full-bleed framed screen. Floating-window-with-margins stretch is **out of scope** (would require patching ratzilla's window-based sizing for modest gain).
- **Colors:** unchanged. Per-speaker coding stays exactly as the shared render produces it. All work is web-side chrome.
- **Font:** **IBM Plex Mono**, self-hosted (woff2), bundled via trunk so it works offline in the PWA.
- **Scrolling:** wheel + trackpad + touch-swipe, wired in `controls.js`.

## Solution

### A. Kill the void — match ratzilla's cell metric

Make each rendered cell exactly **10px × 19px**:

- `#grid pre { line-height: 19px; }` (exact, not `1.0`) so each row is 19px — matches `inner_height / 19`.
- Choose a `font-size` for IBM Plex Mono so the glyph advance is **≤ 10px** (Plex Mono advance ≈ 0.6em → ~16–16.7px font-size lands near 10px). Target at or just under 10px so total text width ≤ `cols × 10` and never overflows/wraps. A few px of slack on the right edge is acceptable (absorbed by the frame).
- **Do not** use `letter-spacing` to reach 10px — it inserts gaps between box-drawing border characters and breaks the box outlines. Adjust `font-size` only.
- Remove/rework the current `#grid { padding: 8px }` top-left offset so the grid aligns to the framed screen (see §C); any inset must not push content past `cols × 10 / rows × 19`.

**Verification is mandatory here:** after building, screenshot and confirm (a) the grid fills to the viewport bottom with no black void, (b) no horizontal overflow or line wrapping, (c) box-drawing borders remain continuous and aligned (see box-drawing risk below).

### B. Self-hosted IBM Plex Mono

- Add `IBMPlexMono-Regular.woff2` (and Bold if the render uses bold weight — the sidebar headers and hint styling do) to `crates/smobb-web/`.
- Copy into `dist/` via trunk: `<link data-trunk rel="copy-file" href="IBMPlexMono-Regular.woff2" />` (and Bold), plus a `@font-face` in `style.css`.
- `font-family: "IBM Plex Mono", <box-drawing-capable fallback>, monospace;` on `html, body`.
- License: IBM Plex is OFL — self-hosting is fine.

**Box-drawing risk (verify during implementation):** the UI uses double-line box borders (`═ ║ ╔ …`) plus single-line and junction glyphs. Confirm IBM Plex Mono covers the Box Drawing Unicode block. If it does not, the browser falls back per-glyph to the next family in the stack; pick a fallback whose box-drawing advance matches (e.g. `"DejaVu Sans Mono", Menlo, Consolas, monospace`) and screenshot-verify the borders stay aligned. Do not attempt to fix this by changing the shared render's border set (out of scope).

### C. Dress the screen (CSS chrome)

Full-bleed, so the "frame" dresses the viewport edges rather than boxing a smaller element:

- **Bezel:** fixed-position overlay hugging the viewport with a subtle border, slightly rounded corners, and a soft inner glow/inset shadow. Non-interactive (`pointer-events: none`), above the grid but below effect overlays.
- **Title strip:** slim fixed bar at the very top — `● ● ●  S.M.O.B.B. //` styled to match the bezel. Reserve its height so the first grid rows are not obscured (the render's own `S.M.O.B.B.` title line can sit just under it, or the strip overlays the top margin slack — decide during implementation via screenshot).
- **Background:** replace flat `#0b0b0b` with a subtle dark radial/linear gradient (kept dark; `--bg` stays the theme-color anchor) so the edges/vignette read as a screen, not a flat panel.
- **CRT treatment:** retune the existing `body::before` scanlines, `body::after` vignette, and `#grid span` per-cell glow so they read as deliberate atmosphere rather than faint noise — slightly stronger but still legible. Keep the `prefers-reduced-motion` / `body.reduced` escape hatches intact (F3 effects toggle must still work).
- Keep existing effect overlays (`#fx-flash`, `#fx-mood`, `fx-glitch`, mood tints) working; the bezel must not cover them.

### D. Control bar — restyle + fix overlap

- Restyle `#controls` to sit cohesively inside the bezel (matching border/radius/background, phosphor-tinted active state).
- **Desktop:** auto-hide or heavily minimize — a physical keyboard needs none of it (it is already dimmed via the `min-width: 800px` media query; take it further, e.g. hidden until hover or removed on wide viewports).
- **Bottom-row overlap:** because ratzilla fills the full window height, the last painted rows can fall under the control bar. Ratzilla renders `floor(inner_height/19)` rows, leaving `inner_height mod 19` px of slack at the bottom (typically a few px). On touch, keep the control bar but ensure the render's bottom line (input `>` + hint bar) is not obscured — if the bar covers it, shrink the bar and/or use the bottom slack; verify by screenshot on a phone-sized viewport. On desktop with the bar hidden, no overlap.

### E. Proper scrolling (`controls.js`, no wasm changes)

The app already scrolls the focused pane on `ArrowUp`/`ArrowDown` and `PageUp`/`PageDown`; `controls.js` already dispatches synthetic `keydown`s on `document` (the events ratzilla listens for). Extend it:

- **Wheel/trackpad:** add a `wheel` listener on `document`; map scroll delta to a burst of `ArrowUp`/`ArrowDown` `keydown`s (a few per notch; use `PageUp`/`PageDown` for large deltas). `preventDefault` to stop the page itself trying to scroll.
- **Touch swipe:** track `pointerdown`→`pointermove` vertical delta (touch pointer type) and emit `ArrowUp`/`ArrowDown` per threshold, without interfering with the existing tap-to-focus-soft-keyboard behavior or the `#controls` and `#b-skip` handlers.
- Scrolling acts on the **currently focused pane** (story vs sidebar, chosen via ←/→ or Tab), consistent with keyboard scrolling. (Pointer-position-aware pane targeting is not attempted — cell hit-testing is unreliable; note this limitation.)

## Out of scope

- Floating centered window with dark side margins (needs ratzilla sizing patch/fork).
- Any change to the shared ratatui render: layout, border style, colors, content.
- New game features, save format changes.

## Testing / verification

1. `cd crates/smobb-web && trunk build --release --public-url /smobb/` builds clean.
2. `trunk serve`, load in a desktop browser; **screenshot** and confirm:
   - No black void; grid fills to viewport bottom.
   - No horizontal overflow / line wrap.
   - Box-drawing borders continuous and aligned (font coverage check).
   - Bezel, title strip, background, CRT treatment read as intentional.
3. Resize the window (narrow, wide, short, tall) — grid re-fills each time, no void, no overflow.
4. Wheel + trackpad scroll the focused pane; Tab/←/→ still switches focus; F1/F2/F3 still work.
5. Phone-sized viewport (or device): soft keyboard raises on tap, touch-swipe scrolls, control bar functional and not covering the input/hint line.
6. `prefers-reduced-motion` and the F3 (`body.reduced`) path still suppress motion.
7. Compare against the existing smoke reference (`docs/superpowers/smobb-web-smoke.png`); update the reference screenshot if kept.

## Files touched

- `crates/smobb-web/style.css` — cell metric, @font-face, font stack, bezel, title strip, background, CRT retune, control-bar restyle.
- `crates/smobb-web/index.html` — font `copy-file` link(s), title-strip/bezel markup if needed.
- `crates/smobb-web/controls.js` — wheel + touch-swipe scrolling; control-bar behavior.
- `crates/smobb-web/IBMPlexMono-Regular.woff2` (+ Bold if needed) — new asset(s).
