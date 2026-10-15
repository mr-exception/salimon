# #237 — larger unequipped cursor

The unequipped white dot is now 9 pixels across instead of 3; its dark outline
is 15 pixels across instead of 5. Each original pixel becomes a centered 3×3
block, giving exactly 3× diameter and 9× pixel area, with no image filtering.
The equipped cross, shared 17×17 texture size, midpoint, image revision policy,
equipment authority and aim/raycast inputs are unchanged. The 15-pixel outline
has a transparent one-pixel margin on every side.

The existing reticle uses physical pixels, independent of OS/UI scale: it stays
9/15 pixels at 1×, 1.5× and 2×, at any ordinary drawable size. Renderer Center
placement keeps NDC (0,0) and ignores scene depth. No additional per-frame
allocation, upload, draw or gameplay work is introduced.

## Checks

Linux x86_64; Rust 1.99.0. Commands from repository root:

- `cargo test -p salimon-client --locked reticle::tests`: passed, 3 tests.
  Covers authoritative equip/stow/pickup state and precision-view hiding,
  bilateral symmetry, exact white/outline bounds and pixel areas of both shapes.
  Equipped cross remains 13/15 pixels with 25 white / 81 outlined pixels.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed.

## Visual verification

On any supported graphical build, compare no selection and empty slots 2–5
against slot 1. Expect the large outlined dot when empty, original cross with
the tool equipped. Pick up a fragment (dot), drop (dot), reselect slot 1 (cross).
Look at bright ground and dark space and resize to 800×600, 1280×800 and
1920×1080; repeat at 1×/1.5×/2× OS scaling. Dot stays at the aim center, outline
remains sharp and no edge clips. Toggle F2: both reticles hide in precision view.
CPU raster/state checks passed; live graphical screenshots were not captured in
this environment. Apple M1 signoff is not required by this issue.
