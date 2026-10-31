# Issue #628 — planetary environment integration matrix

Parent: `main` at `8165ada3` (full parent SHA retained in matrix evidence).
No active PR existed at selection. #611 is merged; #613/#614/#616/#617/#621/
#622/#624/#626/#627/#436 are closed with their code and reports on this parent.

The maintained `scripts/planetary_environment.py` command requires nine native
routes and four independently repeated source exports feeding actual production
wgpu pipelines. It retains original captures, raw-state results, command/log
hashes and median/p95/max cloud CPU/GPU/draw observations. Missing GPU samples
remain null. The night camera samples actual canonical terrain and clears it by
40 m, opposite the current fixed local light; no simulation clock, canonical
publication, physics or ordinary camera changes.

See the [maintained reproduction guide](../../scripts/README.md#planetary-environment-matrix-628).
Positive atmospheric and biological sources are explicitly fictional fixtures on
representative geometry, not life/air on Vesper or observed Moon publications.
Independent render probes isolate optics and physical specimens; they do not
claim one navigable canonical world containing every positive environment.

## Coverage and boundaries

| Requirement | Current production paths |
| --- | --- |
| Vesper vacuum day/night, quiet sterile world | canonical terrain/vacuum/night/native natural-site routes; fixed local light, no ephemeris day-cycle claim |
| Geology orbit to ground, seam, revisit/LOD | universe-terrain route with four exact frozen repeats and orbit revisit; existing physical cache/seam regression tests |
| Coherent storm | exact shared-TT world/runtime weather export; production cloud/rain/dust/orbit/reload/quality captures; controls explicitly labeled |
| Rare living terrain and appearance | actual fictional biological-harvest/mining world, six generated specimen families/materials and natural giant/colony source captures; no newly authored valley publication |
| Harvest/removal/reload | original natural-part grip/cancel/transfer/drop/reopen; embedded tool depletion/cap/interrupt/reopen; current physical source bindings |
| Authored/natural landmarks | existing authored geological patch export and natural giant/colony source exports; real physical geometry, no invented activity or automatic discovery |
| Bearing-only mark and revisit | suit observations and planetary chart native routes; personal history, uncertainty and reload/manual heading contracts |
| Suit/Scout spatial sound | native ramp/interior/settings path plus actual callback PCM for fictional local weather/open/closed transmission, physical sources and vacuum silence |

No new hydrology, fauna, autopilot, scanner mode, canonical living planet or
terrain migration is introduced. The source exports retain exact publication,
seed, shared epoch, coordinates, physical IDs and renderer segments; native
results retain camera/settings, interactions and state through restart.

## Verification

Results are recorded below after final runs. Commands use cloud Linux,
workspace Rust/Mesa/ALSA tooling, Xvfb 1280×800 and two llvmpipe threads.
Initial build failed at missing ALSA development discovery; system apt was not
writable. Downloading and extracting Debian ALSA development/runtime packages
into the workspace sysroot repaired setup. Those attempts are not passes. The first graphical attempt overlapped workspace compilation and its vacuum capture hit the existing command timeout. It was stopped, and its failed packet retained. Inspection also showed the first night view looked too far down to show the horizon; its final camera uses radial up and a tangent target. Final acceptance reruns use the corrected binary with compilation idle. One subsequent launch packet failed because the helper-created X server had exited and left a stale socket; a dedicated foreground Xvfb session repaired that setup. All nine launch failures remain in the failed packet, not counted as successful routes.

## Separate release qualification

[#561](https://github.com/mr-exception/salimon/issues/561) owns subjective
presentation, actual listening and comfort review of retained originals.
[#557](https://github.com/mr-exception/salimon/issues/557) owns physical input/OS;
[#563](https://github.com/mr-exception/salimon/issues/563) owns actual hardware/M1/
Metal budgets. Cloud shared CPU/software-GPU measurements do not establish
hardware FPS, physical acoustic output, subjective approval or cross-host native
fidelity. These qualifications remain unverified.

## Completed source checks

- `cargo build --workspace --locked`: passed after workspace ALSA repair.
- `cargo test --workspace --locked`: **1,244 passed, 0 failed, 51 existing opt-in ignored**.
  Includes weather reconstruction/concurrent-host/seam/vacuum, ecological fixed
  identity/rarity, real source collision/LOD/reconstruction, harvest/save atomicity,
  unknown-bearing/close-inspection and chart-history failure contracts.
- Corrected night camera: `cargo test -p salimon-client --locked canonical_reference_cameras`:
  **1 passed**. The full workspace suite predates only the final night view's
  tangent target/up correction; this focused regression and final native route
  exercise that change. No domain or shader was changed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`: passed at corrected source.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **141 passed**.
  The existing intentional fake-runner timeout/failed-report output belongs to
  negative tests; the suite itself exits zero. New negative checks reject
  incomplete routes/steps, missing/forged capture provenance, dimensions/raw
  telemetry, vacuum leakage, reversed ramp transmission and wrong PCM duration.
- `python3 scripts/check_docs.py --base origin/main`: **0 failures**.

Native metric rows describe the actual bounded observation windows, which can
include startup, camera transitions and synchronous capture costs. They are not
steady-state isolated benchmarks or new budget approvals. Terrain cache bytes
and available GPU allocator observations remain with each original state.

## Final evidence packet

**All nine native routes passed: 277/277 steps, 29 original GPU captures.**
The resumed session reused five completed routes from the identical binary and
reran the four unfinished routes. No incomplete result was reused. The interrupted
background processes had exited; the original incomplete mining startup is not
a pass. `--native-summary` revalidates every route's full contract, ordered steps,
original captures and telemetry before allowing component verification.

Four source exports (weather, specimens, natural sites, authored geological patch)
repeat byte-for-byte across fresh processes. Their four production renderer tests
pass, including exact weather reload and quality/day-boundary controls, physical
specimen LOD retention and actual landmark relief. Production audio callback
export passes; independent PCM review verifies 48 kHz stereo, exact 2 s duration,
vacuum silence and lower closed-Scout energy. Source fixtures and captured images
are review candidates, not promoted presentation baselines.

- [Final matrix and commands (gzip)](evidence/matrix.json.gz)
- [Current native review and median/p95/max metrics](native-review.json)
- [Original-to-report mapping and hashes](evidence-manifest.json)
- [Vesper night](native/run-tvjfvbku/step-006-airless-night.png)
- [Weather](evidence/weather/clouds.png), [exact reload](evidence/weather/reload.png)
- [Natural colony](evidence/natural/natural-DenseColony.png)
- [Actual partial biological source and capped output](native/run-3z41kki0/step-026-cap-release-and-partial-source.png)
- [Vacuum PCM](evidence/audio/vacuum-quiet.wav), [open Scout](evidence/audio/scout-ramp-open.wav), [closed Scout](evidence/audio/scout-ramp-closed.wav)

All original native result/scenario JSONs, renderer source DTOs and logs are
compressed beside the originals. The manifest maps original absolute workspace
paths to portable report paths without rewriting original result bytes. Repetitive
stdout/protocol streams stay in disposable artifacts; their digests remain in the
failed or final packet when recorded. Final source hashes and binary SHA bind
the evidence; report text is written afterward.
