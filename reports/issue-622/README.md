# #622 — biological materials and shared-time weather response

Parent: `main`, `3faf0f764` (full SHA recorded below). #611/#615/#616/#619/#620
are merged; GitHub has no active PR stack at start. This is the first of three
sequential delivery branches; no merge or baseline promotion is authorized.

[Maintained implementation contract](../../docs/planetary-organisms.md#biological-materials-and-weather-response-622)
records source ownership, 96-byte instance layout and limits. Stable species traits
supply six roughness/material families, opaque thin-tissue approximation,
restrained iridescence and sparse terminal-organ emission. Shared TT climate
samples supply wetness, frost, dust and fine wind-directed appendage movement.
Root caps/stems stay fixed; equal radial contraction contains motion within
physical bounds. LOD retains every part. No material/resources/save identity or
canonical life decision changes; Vesper stays sterile. Weather accumulation,
full subsurface/transparent shading, growth and soft-body simulation are excluded.

## Actual verification

Debian 13 x86_64, Cargo/Rust 1.99.1, Python 3.12.14; workspace-local toolchain and
Mesa 25.0.7 llvmpipe Vulkan. Debug assertions enabled. Actual GPU originals are
512×512 fitted specimen views, not approved art baselines or published planets.

- `cargo build --workspace --locked`: passed at final source.
- `cargo test --workspace --locked`: **1196 passed**, **46 existing opt-in ignored**.
  After final shared-clock/source-location mapping, `cargo test -p salimon-client
  --locked app::`: **50 passed**, 2 opt-in ignored. Includes fixed root/endpoints,
  actual weather response warm/cold/dry/vacuum, deterministic repeated time,
  all six material-family bounds, sterile and physical source integration tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`: passed at final source.
- `python -m unittest discover -s scripts -p 'test_*.py'`: **136 passed**.
- `python scripts/check_docs.py --base origin/main`: zero failures.
- With absolute `SALIMON_ORGANISMS_FILE` pointing to the artifact JSON:
  `cargo test -p salimon-client --locked organism_specimen_export -- --ignored
  --nocapture`: passed. [Source DTOs (gzip)](source-specimens.json.gz) retain
  actual world-generated fictional species. Six families × day/red/night,
  twelve shape variants and declared wet/frost/dust fixture responses: **33 images**.
  Warm/cold/dry weather overrides are explicitly marked fixtures; they exercise
  the real adapter, not claims of measured planetary weather.
- With the same file, `SALIMON_ORGANISM_DIR`, `WGPU_BACKEND=vulkan` and
  `VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json`:
  `cargo test -p salimon-renderer --locked organism_production_reference --
  --ignored --nocapture`: passed. Every original has visible geometry, distinct
  bytes, same part count and distant-LOD retention. [GPU log](gpu.log) records
  adapter/timing; synchronous readback is not hardware FPS.
- Production `planetary-vacuum.json` under Xvfb/Mesa: **6/6 passed**, GPU screenshot
  confirms ordinary sterile/airless canonical startup. Final shared-time build
  rerun is retained in native summary below.

Initial setup attempts failed on missing PATH/ALSA development link, unavailable
system package installation, Xvfb socket permissions and an invalid runner flag.
The workspace-local toolchain/ALSA package, declared network permission for the
local X socket and correct runner arguments resolved them. An initial exporter
used a relative path resolved inside its crate and failed; absolute paths fixed
it. First Clippy found a modulo style warning, fixed with `is_multiple_of`; a
clock error-message type inference compile failure was corrected before final
build/checks. None is counted as a passing attempt.

Inspected originals: [day](material-30-day.png), [night without emission](material-30-night.png),
[localized emission without bloom](material-95-night.png), [wet](wet.png),
[frost](frost.png), [dust](dust.png). [Hashes](media-sha256.txt).
#561 retains subjective presentation/art review; #557 physical platform/input;
#563 actual hardware/M1/Metal budgets. These remain unverified; no cloud timing
or software image approves those criteria. No source rights or balance decision
is introduced.
3faf0f76d537d4a6bb3dce5c2fc6de575fad206e
