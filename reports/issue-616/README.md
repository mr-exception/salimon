# #616 — coherent cloud, storm, dust and precipitation presentation

Parent: non-draft [PR #694](https://github.com/mr-exception/salimon/pull/694),
`69e177ef575d08a2df8e7fbb3e2e5c32e3192231`. This child targets
`issue-614-planetary-atmosphere`; #615 and #611 are merged. #694 has green source
and macOS/Windows publication-golden checks. Merge the stack oldest first.

[Source/ownership/model contract](../../docs/planetary-weather-visuals.md).
World creates a fixed body-frame 64×32 field from one exact shared-TT climate
query epoch, with cloud/storm/actual precipitation flux, explicit dust admission,
wind/temperature and the exact local audio/lighting source. Runtime maps the
field, chemical visual threshold and local cloud light attenuation. Renderer
uses two 32-bit float textures (64 KiB), a 224 B uniform and one 8/16/24-sample
optical-volume draw. Ground/opaque depth stops foreground weather. Quality and
reload never change canonical weather; Vesper remains vacuum with no field.
No fluid accumulation, hazards, fauna or new canonical world is introduced.

## Verification and original evidence

Cloud Debian 13 x86_64; Rust/Cargo 1.99.1, Python 3.12, Mesa 25.0.7 llvmpipe
LLVM 19.1.7 Vulkan. Source workspace tools env, dev/test debug=0, assertions on.

- `cargo test --workspace --locked weather`: **12 passed**, zero failures.
  Includes original climate repeatability/seam/vacuum tests and new frame
  reconstruction, exact local-source matching, dust admission and vacuum tests.
- Workspace build, all-target Clippy `-D warnings`, fmt, whitespace and docs
  `--base issue-614-planetary-atmosphere`: pass. Tooling: **134 passed**.
  An initial Clippy test-literal grouping warning was corrected.
- `SALIMON_WEATHER_FRAME_FILE=artifacts/issue-616-frame.json cargo test -p
  salimon-client --locked weather_integrated_frame_export -- --ignored --nocapture`:
  **passed**. Actual 2,048 climate queries took **19.227 ms** in this debug build;
  this explicit fixture/capture cost is not an ordinary-frame or hardware claim.
  Current Vesper short-circuits in vacuum; consumers of nonvacuum worlds must
  schedule/cache bounded field refreshes outside latency-sensitive paths.
- With that same exported file, `SALIMON_WEATHER_DIR=artifacts/issue-616-integrated`,
  and the workspace Vulkan driver,
  `cargo test -p salimon-renderer --locked weather_production_reference -- --ignored --nocapture`:
  **passed**. Actual world/runtime frame definition
  `5d1b7c2048e4ab5b429552c65aba7bd8df4e672efad3b2dc0357c76f5f99b10d`,
  TT day 10957/nanos 43200000000000. Reload pixels are exact; day-wrap continuity,
  sample quality, nonzero cloud/rain, distinct dust and empty clear controls pass;
  invalid atmosphere/grid/basis reject. Controlled constant fields also pass.
  Synchronous submit/readback samples are 69–198 ms, including readback/warmup,
  **not actual hardware FPS or M1/Metal qualification**.
- Native `planetary-vacuum.json` on the rebuilt production client under Xvfb/Mesa:
  **6/6 passed**, retaining the original airless Vesper view and physical basin.

Original integrated frames: [clouds](clouds.png), [exact reload](reload.png),
[low quality](low-quality.png), [dust](dust.png), [precipitation](rain.png),
[orbit](orbit.png), [clear](clear.png), [day end](day-end.png),
[day start](day-start.png), [Vesper](vesper-zenith.png).
[Media hashes](media-sha256.txt), [native summary](native-summary.json),
[exact exported source frame (gzip)](world-runtime-frame.json.gz),
[decompressed SHA-256](frame-sha256.txt).

The first integrated render used a nearly clear polar region and correctly did
not satisfy a deliberately cloudy control's brightness assertion. The final
fixture camera selects the actual maximum-cloud region from the unchanged
source field; no cloud metadata was forced into the dry region. The initial
failure is retained. This demonstrates why source data and camera context matter.

## Limits and retained ownership

The atmospheric fixture is declared fictional engineering data, not a scientific
Moon/planet publication. Optical cloud microstructure and wind-driven periodic
wisps/streaks are stylized; there is no CFD/droplet/volumetric-shadow claim. Local
cloud light attenuation is approximate. Physical pack/body/frame adoption remains
#427; real audio consumption is #627. Original pixels remain review candidates
under [#561](https://github.com/mr-exception/salimon/issues/561); physical platform
#557 and actual hardware/M1/Metal budgets #563 remain explicitly unverified.
All affected world/runtime/renderer guides and root routing changed in this PR.
