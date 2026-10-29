# #614 — planetary atmospheric scattering

Parent: non-draft [PR #693](https://github.com/mr-exception/salimon/pull/693),
`d10fb6758256c7903a1c715f09e990ea24b7db94`. #419 and documentation #611 are merged;
#423's usable code is in this ancestry, with green GitHub source/publication
checks. This child targets `issue-423-hybrid-sky`; merge the parent first.

The [maintained atmosphere contract](../../docs/planetary-atmosphere.md) maps
world optical parameters, runtime light/spectrum/eclipse inputs and the neutral
renderer DTO. The production post-opaque pass integrates spherical exponential
single scattering, planetary shadows and illuminator extinction, stopping at
actual reverse-Z geometry depth. Surface/descent/orbit use one continuous model.
Uniform 128 B, no mesh/asset allocation, one optional fullscreen draw, fixed
16 view × 6 solar-path samples. Atmospheric pressure/chemistry/height are explicit;
world climate and gas/drag physics remain authoritative and unchanged. Vesper
stays airless. All affected owning guides and root routing are updated here.

## Actual verification

Cloud Debian 13 x86_64, Rust/Cargo 1.99.1, Mesa 25.0.7 llvmpipe LLVM 19.1.7 Vulkan,
Python 3.12. Source the existing workspace tools environment; profiles use
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0` with assertions enabled.

- `cargo test --workspace --locked atmospher`: **13 passed**, zero failures;
  the production evidence test remains opt-in and was run explicitly below.
  Includes original U18 atmosphere/drag integration regressions, new pressure/
  chemistry/altitude/failure tests and runtime vacuum/illuminator mapping tests.
- `cargo build --workspace --locked`, all-target Clippy with `-D warnings`,
  rustfmt, whitespace and `python3 scripts/check_docs.py --base issue-423-hybrid-sky`:
  pass. `python3 -m unittest discover -s scripts -p 'test_*.py'`: **134 passed**.
- With `WGPU_BACKEND=vulkan`, workspace `VK_DRIVER_FILES`, and
  `SALIMON_ATMOSPHERE_DIR=artifacts/issue-614`,
  `cargo test -p salimon-renderer --locked atmosphere_production_reference -- --ignored --nocapture`:
  **passed**, eight original production images. Day scattering is wavelength
  dependent; red illuminator changes the sky toward red; night/total eclipse/
  vacuum controls are black; foreground near depth suppresses scattering;
  upper-sky attenuation decreases with altitude. Invalid eclipse input rejects.
  Final synchronous submit/readback samples are in the log (roughly 47–94 ms
  after first shader warmup), **not hardware frame-rate or M1 performance**.
- `scripts/salimon-test run scenarios/evidence/planetary-vacuum.json --binary
  target/debug/salimon-client --artifacts artifacts/issue-614-native-final
  --screenshot-command '["native"]'` under a fresh Xvfb/Mesa server:
  **6/6 passed**, 20.062 s. The ordinary Vesper view looks upward with normal
  production camera/terrain. The dark visible sky and host star remain correctly
  occluded by the published landing basin's steep surrounding terrain; gray
  walls are real geometry, not atmosphere. Temporary diagnostic draw omissions
  used to identify this were removed; terrain/markers/physics are unchanged.

The initial GPU test incorrectly assumed whole-image brightness must decrease
with altitude; the changing horizon/ground solid angle disproves that assumption.
The corrected test compares a fixed upper-sky region and passes without changing
the model or acceptance tolerances. The first native launch failed on an expired
X server socket; a fresh server passed. Original failures are retained.

Originals: [day](day.png), [red illuminator](red-star.png), [night](night.png),
[eclipse](eclipse.png), [descent](descent.png), [orbit](orbit.png),
[near-depth occlusion](near-occlusion.png), [vacuum](vacuum.png),
[native Vesper zenith](vesper-zenith.png). [Hashes](media-sha256.txt),
[native summary](native-summary.json).

## Limits retained explicitly

Single scattering, approximate chemistry/color coefficients and mean RGB surface
extinction are visual models. No measured spectroscopy, multiple scattering,
new canonical atmospheric planet, certified ephemeris/eclipse producer, gas
physics change or hazard/hydrology is claimed. Current playable Vesper is vacuum;
the atmospheric world is a declared production-renderer fixture. Generic climate/
light inputs support later physical body adoption (#427); weather coupling #616
and sound #627 are separate. Actual subjective art review remains
[#561](https://github.com/mr-exception/salimon/issues/561), hardware/M1/Metal
qualification [#563](https://github.com/mr-exception/salimon/issues/563) and physical
platform checks #557. Their unrun criteria remain unverified.
