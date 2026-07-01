# Vertex oracle (`oracle/vtx`)

Proves the port's background **quad geometry** matches the decomp — the transform
path no other oracle covers. The bg-state oracle checks the camera; the anm-quad
oracle checks per-quad VM state (sprite/scale/rotation/colour). Neither touches
the final world transform + projection built in `background.rs::scene()`.

## How it works

`build.sh` compiles the **real decomp** `AnmManager::ExecuteScript` **and `Draw3`**
(kept, not trimmed) with:
- real, spec-exact D3DX matrix math (`d3dx8math.h` — row-vector, LH),
- a `FakeDev` extended to capture the world matrix `Draw3` hands to
  `SetTransform(D3DTS_WORLD)`.

`oracle_vtx_main.cpp` reads a quad list (emitted by the port so the STD parse is
shared, not the thing under test), runs `Draw3` per quad, then projects the ±128
base quad through a spec-exact `SetupCamera` (view · proj) to screen space.

The port side (`crates/game/examples/bg_vtx_dump.rs` → `Background::dbg_quad_geom`)
emits the same quads' screen corners from `scene()`.

## Run

```
./compare.sh <stage 1-6> <frame>
```

Corners are compared order-independently (Draw3 bakes the Y-flip into the world
matrix, reversing corner order vs the port). Off-screen / near-plane quads
(clip-w < 10, or projecting far outside the 384×448 viewport) are skipped —
there f32-vs-float rounding explodes and screen space is meaningless. On-screen
quads match to **0.000 px** across all 6 stages.

## What it caught

The AnchorTopLeft (op23) shift used a signed `spriteW*scaleX/2`, but the decomp
uses `fabsf(...)` (`Draw3` 876-887). With an op7 X-flip (negative scaleX — the
mirrored stage-1 right bank), the port shifted the wrong way → banks landed 64
units off → the stage-1 "lean". Fixed in `scene()` by taking the absolute value.
