<div align="center">

<img src="icon.svg" width="120" alt="S.M.O.B.B." />

# S.M.O.B.B.

**A text-based story adventure about A.I. and people.**
Crack passwords, decode what happened, and follow the story to its end — a self-contained, ~5-hour experience.

[![latest release](https://img.shields.io/github/v/release/Kwaddo/smobb-releases?style=for-the-badge&label=latest&color=cd0000)](https://github.com/Kwaddo/smobb-releases/releases/latest)
[![downloads](https://img.shields.io/github/downloads/Kwaddo/smobb-releases/total?style=for-the-badge&color=3a3a3a)](https://github.com/Kwaddo/smobb-releases/releases)

### ▶ Play now

[![Play in your browser](https://img.shields.io/badge/Play_in_your_browser-cd0000?style=for-the-badge&logoColor=white)](https://kwaddo.github.io/smobb/)

*Runs in any modern browser / also installable on your phone (open it, then “Add to Home Screen”).*

### ⬇ Download

[![macOS — Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/Kwaddo/smobb-releases/releases/latest/download/smobb-desktop-macos-arm64.zip)
[![macOS — Intel](https://img.shields.io/badge/macOS-Intel-555555?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/Kwaddo/smobb-releases/releases/latest/download/smobb-desktop-macos-x86_64.zip)
[![Windows](https://img.shields.io/badge/Windows-x86__64-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/Kwaddo/smobb-releases/releases/latest/download/smobb-desktop-windows-x86_64.zip)
[![Linux](https://img.shields.io/badge/Linux-x86__64-333333?style=for-the-badge&logo=linux&logoColor=white)](https://github.com/Kwaddo/smobb-releases/releases/latest/download/smobb-desktop-linux-x86_64.tar.gz)

*Always the newest build. Older versions: [all releases](https://github.com/Kwaddo/smobb-releases/releases).*

</div>

---

## What is it?

S.M.O.B.B. is a text adventure. You explore a story that surrounds A.I. and people, gathering clues and passwords to unlock what really happened and push the plot forward. It isn't a hard puzzle game — it's a story you play through, roughly five hours end to end. Your progress (clues, passwords, decoded hex, people you've met, and your settings) saves automatically, so you can put it down and come back.

## Running your download

| Platform | Steps |
|----------|-------|
| **macOS** | Unzip, then open `S.M.O.B.B..app`. First launch: **right-click → Open** (the build is unsigned, so a plain double-click is blocked by Gatekeeper). If it still refuses: `xattr -dr com.apple.quarantine "S.M.O.B.B..app"`. |
| **Windows** | Unzip and run `smobb-desktop.exe`. SmartScreen may warn on an unsigned app — **More info → Run anyway**. |
| **Linux** | `tar -xzf smobb-desktop-linux-x86_64.tar.gz`, then `./smobb-desktop`. |

No installer, no terminal needed — the download is the standalone game in its own window. Prefer zero download? [Play in the browser](https://kwaddo.github.io/smobb/).

## Controls

| Key | Action |
|-----|--------|
| **Enter** | Submit command |
| **Tab** | Switch / focus the sidebar (Options ↔ Journal) |
| **F1** | Hide / show the sidebar |
| **Hold Space** (or **Shift**) | Fast-forward text while held |
| **F2** | Toggle instant-text mode |
| **← / →** | Move focus between the story and the sidebar |
| **↑ / ↓, PgUp / PgDn** | Scroll the focused pane |
| **`leave` / `exit`** | Back out of a dialogue or location |
| **Esc** | Quit |

*On touch / web, the on-screen bar covers the sidebar, Tab, Skip, scroll, and Enter.*

**Saves** are automatic and live per platform: next to the app on desktop (`~/Library/Application Support/smobb/` on macOS, `%APPDATA%\smobb\` on Windows, `~/.local/share/smobb/` on Linux) and in `localStorage` in the browser. Delete the save to start fresh.

---

<div align="center">
<sub>Built in Rust — one engine, four frontends (terminal, native window, and web/PWA).</sub>
</div>
