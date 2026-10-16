# Production model image regression

Run from repository root, on a dedicated desktop with a fixed 1280×800 viewport,
same driver/backend, build, seed and pixel density. Fixtures reuse maintained
production scenarios, seed and fixed-step actions. Screenshots use the explicit
wgpu surface copy (not a desktop screenshot or Blender rendering). Repeats freeze
simulation while continuing presentation. The moving fixture advances camera
pose between checkpoints; comparisons use matching poses, never consecutive
moving images. Other views cover lower glazing/glass edges, exterior, monitor,
Core, tool/deposit/fragment, contact piles and floor/door/ramp.

```
python -m pip install -r scripts/visual-requirements.txt
python scripts/model_visual_tests.py capture --binary target/debug/salimon-client
python scripts/model_visual_tests.py compare --temporal-only
python scripts/model_visual_tests.py compare
```

Linux software Vulkan: prefix capture with
`xvfb-run -a -s '-screen 0 1280x800x24'` and set `WGPU_BACKEND=vulkan`.
Nightly verification runs software-Vulkan captures and repeat-stability checks.
These commands also remain available locally; PR CI only builds source. Missing approved baselines
fail ordinary comparison, never silently initialize or overwrite a baseline.
Artifacts contain PNG candidates, per-step production inspection/protocol logs,
highlight/difference PNGs, changed/compared pixel counts and peak channel delta.

Default tolerance: RGB channel delta >8 counts as changed, at most 0.1% of
unmasked pixels. This allows minor raster noise, not major color/silhouette
changes. Absolute pixel masks `[left,top,right,bottom]` exclude intentional
animation (or a changing HUD) only after review. Empty comparison regions fail.
Flicker checks compare each frozen repeat with its first frame; deterministic
intentional motion belongs between poses. Tolerances are backend-specific
starting points and must be calibrated with real captures rather than inflated
to conceal failures. Size changes fail explicitly.

## Approval policy and style replacement

`baselines/option-a/<fixture>/<checkpoint>.png` is reserved for **human-reviewed**
Option A images. No approved production image baseline is claimed by this
implementation. Review candidate images for coherent low-poly silhouettes,
transparency, scale and competing surfaces before copying selected images there
in a separate reviewed commit, recording driver/adapter, resolution and revision.
Do not automatically promote captured output. Hardware/native M1 execution and visual
approval remain tracked in #229; software Vulkan is not equivalent coverage.

An Option B trial uses a separate baseline directory via `--baselines`, with
unchanged fixture setup/actions/gameplay assertions. Comparing an A capture to a
B candidate must expose the deliberate visual replacement as a reviewable diff.
The offline injected-change test demonstrates that gate without inventing an
approved aesthetic or redesigning a production asset.
