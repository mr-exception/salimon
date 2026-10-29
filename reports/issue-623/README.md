# #623 — rare geological site identities on physical terrain

Parent: [PR #702](https://github.com/mr-exception/salimon/pull/702), branch
`issue-622-biological-materials`, `94ab718164851586970be002f191626f276d03ff`.
#611/#612/#424 are merged. This child contains only #623 changes; merge oldest first.

[Maintained contract](../../docs/planetary-landmarks.md) records the fixed rare
survey, explicit authored anchors, bounded local/regional/planetary support,
measured relief admission, geological history/substrate classification, immutable
source/site/reference binding and exact render/collision consistency. The terrain
itself supplies every site's geometry; no parallel heightfield, displacement,
collider, reward, auto chart marker or player-centered quota is introduced.
Observed geography is preserved. Canonical Vesper reconstructs its empty
procedural site inventory without a save layout change. The API accepts explicit
authored names only where published terrain has meaningful relief; it does not
invent an owner signature or guarantee one on every body. Caves/arches/overhangs
remain unsupported by this heightfield contract.

## Actual validation

Debian 13 x86_64, Rust/Cargo 1.99.1, Python 3.12.14; Mesa 25.0.7 llvmpipe Vulkan.
Debug assertions enabled; workspace-local tooling. Rendered positive-source
basin is a declared physical fixture, not a newly published scientific Moon map.

- `cargo test --workspace --locked landmark`: **4 passed**, 2 evidence-only ignored.
  Actual source positions/normals, strict binding reconstruction/corruption,
  observed/empty/Vesper geography and 32-publication rarity audit pass.
- `cargo test -p salimon-world --locked landmark -- --nocapture`: **3 passed**;
  [rarity audit](rarity-audit.log) preserves actual survey result.
- `cargo build --workspace --locked`, `cargo clippy --workspace --all-targets
  --locked -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`: passed.
- `cargo test --workspace --locked`: **1200 passed**, **48 existing opt-in ignored**.
- `python scripts/check_docs.py --base issue-622-biological-materials`: zero failures.
- With absolute `SALIMON_LANDMARK_PATCH_FILE`, `cargo test -p salimon-client
  --locked landmark_production_patch_export -- --ignored --nocapture`: passed.
  [Original production mesh/source/binding (gzip)](source-patch.json.gz) comes
  from the actual runtime terrain/material generator, not drawn test vertices.
- With that same source file, absolute `SALIMON_TERRAIN_STYLE_DIR`,
  `WGPU_BACKEND=vulkan` and the workspace `lvp_icd.json`, `cargo test -p
  salimon-renderer --locked landmark_production_reference -- --ignored --nocapture`:
  passed. [Approach](approach.png), [regional](regional.png), [local](local.png)
  preserve the same physical relief, at 1e12-m world anchors. Frozen repeated
  production frames match exactly and all contain visible geometry. These
  isolated patch images do not claim a full-planet view; native coverage below
  exercises the full runtime streamer. [GPU log](gpu.log), [hashes](media-sha256.txt).
- `/workspace/.salimon-tools/start-display.sh python scripts/salimon-test run
  scenarios/evidence/universe-terrain.json --binary target/debug/salimon-client
  --artifacts artifacts/issue-623/native --screenshot-command '["native"]'
  --summary`: **20/20 passed**, actual orbit/surface/edge/revisit GPU captures.
  [Native summary](native-summary.json). Shader cache writes outside the workspace
  were unavailable; Mesa disabled that cache, with no capture/test failure.

No scene/material/physics baseline was weakened. The deterministic surveys name
already existing physical features; they do not claim terrain art generation was
newly authored here. Human appearance review #561, physical platform/input #557,
actual hardware/M1/Metal budgets #563 remain explicitly unverified. Biological
signature site extension is next in #624; player observations/charts #625/#626
remain independent of this immutable private inventory. No merge performed.
