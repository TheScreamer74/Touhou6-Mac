<div align="center">

# 東方紅魔郷
## Touhou 6 — the Embodiment of Scarlet Devil, for macOS

**A native Apple Silicon reimplementation of ZUN's 2002 classic,**
**built in Rust on Metal — bring your own copy of the game.**

![Platform](https://img.shields.io/badge/platform-macOS%20(Apple%20Silicon%20%2B%20Intel)-black?logo=apple)
![Language](https://img.shields.io/badge/language-Rust-b7410e?logo=rust)
![Renderer](https://img.shields.io/badge/renderer-wgpu%20%2F%20Metal-blueviolet)
![Status](https://img.shields.io/badge/status-stage%201%20playable-crimson)

*"The border land was wrapped in Scarlet Magic.*
*Girls believe that you solve this mystery..."*

</div>

---

## What is this?

A from-scratch engine that reads the **original, unmodified game files** of
Touhou Koumakyou ~ the Embodiment of Scarlet Devil (v1.02h) and runs the game
natively on macOS. No Wine, no emulation, no patched binaries — the same
model as [devilutionX](https://github.com/diasurgical/devilutionX) or
[OpenRCT2](https://github.com/OpenRCT2/OpenRCT2).

You supply your own legally obtained copy of the game. This repository
contains **zero** copyrighted assets — only code.

## Status

| Milestone | State |
|---|---|
| PBG3 `.DAT` archive extraction | ✅ done |
| ANM sprite/animation format + VM | ✅ done |
| Title screen, interactive menu | ✅ done |
| Audio — BGM + sound effects | ✅ done |
| Stage 1: Reimu, fairies, Rumia, lives & bombs | ✅ playable (approximated patterns) |
| ECL interpreter (exact original stage scripts) | 🔜 next — decomp-driven |
| Stages 2–6 + Extra | ⬜ |
| Replays, Practice+, rewind | ⬜ |
| JP / EN language selector | ⬜ |

Gameplay logic is being aligned opcode-by-opcode with the
[happyhavoc/th06](https://github.com/happyhavoc/th06) matching decompilation,
so behavior converges on the real thing — including the original RNG.

## Already better than 2002

- **Stable 60 Hz** — game logic runs on a fixed timestep, decoupled from
  display refresh. No more speed tied to your GPU. ProMotion-safe.
- **Native resolution scaling & Metal rendering** via wgpu.
- **Crash reports** written to `logs/` (the original just vanished).
- Planned: rebindable keys & gamepads, practice any spellcard, replay
  rewind, instant restart.

## Building

```sh
# Rust toolchain (rustup.rs), then:
git clone https://github.com/TheScreamer74/Touhou6-Mac.git
cd Touhou6-Mac
cargo run -p th06 -- --game-dir "/path/to/your/th06/folder"
```

The game folder must contain the v1.02h data files
(`CM.DAT`, `IN.DAT`, `MD.DAT`, `ST.DAT`, `TL.DAT`, `ED.DAT` and the `bgm/`
directory). The Windows `.exe` files are never executed — only the data is
read.

## Controls

| Key | Action |
|---|---|
| Arrows | Move |
| `Z` | Shoot / confirm |
| `X` | Bomb (Fantasy Seal) / back |
| `Shift` | Focus (slow movement) |
| `Esc` | Pause / back |

## Architecture

```
crates/
├── formats   PBG3 archives · ANM sprites · (soon) ECL · STD · MSG
├── engine    wgpu sprite renderer · fixed-timestep loop · input · rodio audio
└── game      ANM VM · title menu · stage logic · scenes
```

## Legal

This is an unofficial fan project, unaffiliated with Team Shanghai Alice.
Touhou Project and the Embodiment of Scarlet Devil are © ZUN / Team Shanghai
Alice. This repository distributes no game assets; it requires a legally
purchased copy of the original game to run. Made under the spirit of the
[Touhou Project fan-work guidelines](https://touhou-project.news/guidelines_en/).

## Credits

- **ZUN** — for the game, obviously. Buy his games.
- [happyhavoc/th06](https://github.com/happyhavoc/th06) — matching
  decompilation; our ground truth for game logic.
- [thtk / thpatch](https://github.com/thpatch/thtk) — file format
  documentation and tooling.
- [PyTouhou](https://pytouhou.linkmauve.fr) — pioneering open
  reimplementation, an invaluable map of the territory.

<div align="center">

*⑨ Even Cirno could build this. (She could not.)*

</div>
