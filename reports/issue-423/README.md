# #423 — hybrid astronomical sky

Implements a bounded production-wgpu authored Milky Way and physically sized host
and optional catalog stars. Parent is main `f5e39b2f5901557788bf347a2f9366c7762d3511`;
#422 is merged. No PR is merged by this workflow.

[Maintained source/contract/test mapping](../../docs/universe-sky.md). The streaming
worker verifies optional `sky` sidecars (star IDs, physical parameters, explicit
split positions/epochs, duplicate/count/byte bounds). Runtime deduplicates and
selects apparent-brightness order; the fixed maximum is 128 stars including the
published host. Quality/visibility never grants discovery or writes chart/saves.
The canonical stream observer now uses the published galactic anchor, correcting
the previous origin-cell request. Renderer uses two bounded draws, depth zero,
physical angular disks and subpixel point-spread footprints. Historical fixtures
retain their prior background. Owning guides and root routing changed together.

## Verification

Debian 13 x86_64, Rust/Cargo 1.99.1, Python 3.12, Mesa 25.0.7 llvmpipe LLVM 19.1.7
Vulkan (CPU adapter). Source `/workspace/.salimon-tools/env.sh`; set
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0`, normal assertions enabled.
The existing workspace tool area needed ALSA development packages; these were
extracted there without changing repository build dependencies.

- `cargo test --workspace --locked`: initial run passed 1,064 tests with 38
  opt-in ignored; one **new test-fixture setup failed** because its private-pack
  revision differed from the fixture catalog. Corrected the fixture to use the
  actual catalog revision, preserving production validation. Final universe and
  world suites plus workspace doctests ran separately; see retained final logs.
  All other initial workspace suites passed, including 493 runtime tests.
- `cargo test -p salimon-universe --locked`: corrected full universe suite passes.
- `cargo test -p salimon-client --locked app::sky`: final runtime adapter passes.
- `cargo test -p salimon-world --locked` and workspace doctests: final logs below.
- Workspace build, all-target Clippy with warnings denied, rustfmt, whitespace
  and `python3 scripts/check_docs.py --base origin/main` pass. The first Clippy
  run found two test-only iterator style warnings; corrected and reran.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **134 passed**.
- With `WGPU_BACKEND=vulkan`, the workspace Mesa `VK_DRIVER_FILES`, and
  `SALIMON_SKY_DIR=artifacts/issue-423`,
  `cargo test -p salimon-renderer --locked sky_production_reference -- --ignored --nocapture`:
  **passed**. Actual production shaders export identical origin/1e12-m translated
  pixels, a visible resolved star and black near-depth occlusion; malformed unit
  directions and excess stars reject. Original synchronous capture durations were
  592.486/389.599/380.569 ms, including pipeline warmup/readback, **not frame-time
  or hardware FPS measurements**. Fixed storage is 4 KiB stars + 96 B uniform.
- `start-display.sh scripts/salimon-test run scenarios/evidence/universe-journey.json
  --binary target/debug/salimon-client --artifacts artifacts/issue-423-native
  --screenshot-command '["native"]'`: **25/25 passed**, 110.721 s. Production
  startup, safe Vesper ground, walk/boarding and restore remain functional.

Original pixels: [sky](sky.png), [translated](translated.png),
[occluded](occluded.png), [canonical startup](canonical-start.png).
[Media SHA-256 inventory](media-sha256.txt), [native summary](native-summary.json).
The retained initial failure logs remain alongside corrected final runs.

## Explicit limits and follow-ups

The band is authored art, not a scientific census. Optional star-sidecar
publication is explicit; existing Gaia packs without complete physical/position
inputs do not acquire guessed stars. Color/exposure are approximate. The current
playable body uses its existing fixed host-light axes; full orbital/rotating frame
adoption is #427. Optional proper motion starts at each supplied reference epoch
plus simulation elapsed time, with no current-epoch or UTC/TT conversion claim.
Atmospheric scattering/weather remain #614/#616. Human appearance review of the
originals stays [#561](https://github.com/mr-exception/salimon/issues/561), actual
hardware/M1/Metal budgets [#563](https://github.com/mr-exception/salimon/issues/563),
physical platform qualification #557, full source coverage #524/#526 and public
redistribution review #659. None is claimed passed by these cloud results.
