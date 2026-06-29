# About S.M.O.B.B.
S.M.O.B.B. is a text-based story adventure that surrounds A.I. and people, and requires you to use passwords and such to know more about what happened and to progress with the story further. 

The game is not complex at all, and is a five-hour experience at most.

The releases are right here! https://github.com/Kwaddo/smobb/releases/tag/release

## Build & run (Rust)

### Terminal UI (recommended)

```
cargo run -p smobb-tui
```

Launches the interactive terminal UI for playing S.M.O.B.B.

**Keybindings:**
- **Tab** — switch sidebar (Options / Journal)
- **F1** — hide / show sidebar
- **Hold Space** (or **Shift**) — fast-forward: rolls through lines quickly while held, normal pace on release. Space works everywhere; Shift also works on terminals with Kitty keyboard-protocol support. (Web: the on-screen Skip button.)
- **F2** — toggle skip-mode (fully instant text, persists until toggled off). (Web: the on-screen Inst button.)
- **←/→** — move scroll focus between the story (left) and the sidebar (right); the focused pane's border brightens. **Tab** also focuses the sidebar.
- **↑/↓, PgUp/PgDn** — scroll the focused pane (transcript or the Journal/Options sidebar)
- **`leave` / `exit`** — type at any prompt to back out of a dialogue or leave a location (the journal also notes everything you've already asked about)
- **Enter** — submit command
- **Esc** — quit

**Saves:** progress (the journal — clues, passwords, asked topics, decoded hex, people — plus your settings) persists automatically. The terminal app writes `smobb-save.json` next to where you launch it; the native window app writes it to the per-user data dir (`~/Library/Application Support/smobb/` on macOS, `%APPDATA%\smobb\` on Windows, `~/.local/share/smobb/` on Linux); the web app uses the browser's `localStorage`. You restart at the hub with your notebook intact. Delete the save to start over.

### Native window app (standalone, no terminal)

```
cargo run -p smobb-desktop
```

Runs the same UI in its own GPU window (via [`ratatui-wgpu`](https://crates.io/crates/ratatui-wgpu) + winit) — no terminal involved. Cross-platform (macOS / Windows / Linux). On Windows, release builds open no console window.

Build a double-clickable macOS `.app`:

```
cargo install cargo-bundle           # once
cd crates/smobb-desktop
cargo bundle --release               # builds the binary, wraps it, embeds the icon
# -> target/release/bundle/osx/S.M.O.B.B..app   (and a .dmg)
```

(Run it from the crate directory — `cargo bundle --release -p smobb-desktop` from the workspace root builds the wrong package and skips the icon.)

### Headless play

```
cargo run -p smobb-core --example play
```

Runs the pure Rust engine in headless mode (non-TTY). The engine (`crates/smobb-core`) is a state machine shared by every frontend.

### Web / mobile (PWA)

The same terminal UI runs in the browser via [ratzilla](https://crates.io/crates/ratzilla) (the TUI compiled to WebAssembly). Build/serve with [trunk](https://trunkrs.dev):

```
cargo install trunk            # once
rustup target add wasm32-unknown-unknown   # once
cd crates/smobb-web && trunk serve         # dev server at http://127.0.0.1:8080
trunk build --release          # static bundle in crates/smobb-web/dist/
```

On a phone, open the served page and "Add to Home Screen" — the web app manifest + service worker make it installable and playable offline. A touch control bar (≡ sidebar, Tab, hold-Skip, scroll, Enter) plus the soft keyboard drive it without a physical keyboard.

### Architecture

- `crates/smobb-core` — pure engine (story as embedded RON data, no I/O).
- `crates/smobb-tui-core` — portable ratatui App + rendering (no terminal backend).
- `crates/smobb-tui` — terminal app (crossterm).
- `crates/smobb-desktop` — standalone native window app (ratatui-wgpu + winit), no terminal.
- `crates/smobb-web` — browser/PWA frontend (ratzilla/WASM).

### Releases & deploy (GitHub Actions)

Three workflows in `.github/workflows/`:

- **`ci.yml`** — on every push/PR: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, and the wasm build.
- **`release.yml`** — push a version tag and it builds the desktop app for Windows, macOS (Intel + Apple Silicon), and Linux, then uploads the archives to a GitHub Release:

  ```
  git tag v2.0.0 && git push origin v2.0.0
  ```

- **`pages.yml`** — on push to `main`, builds the web app and deploys it to GitHub Pages at `https://<user>.github.io/smobb/`. One-time setup: repo **Settings → Pages → Source: "GitHub Actions"**.

The original Java version is preserved under `legacy/` as the historical artifact.
