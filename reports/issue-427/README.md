# Issue 427 — ordinary canonical play isolation

Parent: `main`, `33befc07`. Cloud Linux implementation; no PR was merged.

## Change and scope

Ordinary rendering now maps published local bodies directly, without constructing
compressed Solar System spheres or leaking the three precision markers. F2 and
explicit automation fixtures retain the historical engineering scene. Canonical
ship telemetry and live-controller forecast selection no longer disappear at the
prototype's 3 Mm cutoff; local interaction/collision limits are unchanged.

Normal checkpoint startup requires schema 24+ canonical state with its existing
exact immutable publication binding. Prototype/engineering slots return a fatal
incompatibility error before publication: both generations and caller state stay
unchanged. A new `SALIMON_SAVE_DIR` starts a separate journey. Historical fixture
reconstruction remains explicitly available to tests, with no normal save import.

This task does not add ephemeris-driven ordinary flight, orbital state adoption,
offline trajectory, full observed-data coverage or new planetary geography. Their
existing implementation owners and recorded limits remain intact.

## Evidence mapping

- Runtime `checkpoints.rs::canonical_gate_tests`: real two-slot native store,
  source-byte preservation, no partial caller replacement, current admission and
  rejection before canonical binding was introduced.
- Runtime `app/scene.rs::canonical_scene_tests`: canonical bodies/no precision
  markers versus explicit six-body/three-marker engineering scene.
- Ship `controller/telemetry.rs::canonical_range_tests`: canonical current-body
  telemetry beyond 3 Mm; existing compressed range tests retain their boundary.
- Existing canonical session regressions exercise arrival, hull/ramp walking,
  mining, physical fragments, cargo/Fabricator, funded flight and exact reload.
- [Canonical guide](../../docs/universe-gameplay.md#ordinary-canonical-play-427),
  owning runtime/ship architecture/invariants and save/usage/routing guides are
  updated in this PR. No catalog, terrain or save payload bytes are reseeded.

## Environment and checks

Debian Linux x86_64; Rust/Cargo 1.99.1, Python 3.12.14. Reused workspace
`/workspace/.salimon-tools/env.sh`; installed missing `libasound2-dev` in the
workspace sysroot and corrected its local pkg-config library path. No repository
dependency changes. Software Vulkan/Xvfb uses `LP_NUM_THREADS=2` and a writable
Mesa shader cache. Physical input, actual M1/Metal budgets and subjective review
remain unverified under #557/#563/#561; cloud results do not establish them.

- `cargo test --workspace --locked --offline`: **1209 passed**, 49 existing opt-in ignored, zero failed. This run preceded the final extra bootstrap regression (added and passed separately).
- `cargo test -p salimon-client --locked --offline canonical_gate_tests`: **3 passed**, including normal fresh bootstrap, verified exit/reopen and fatal prototype rejection.
- `cargo test -p salimon-ship --locked --offline canonical_telemetry_has_no_compressed_neighborhood_cutoff`: **1 passed** after the test-only Clippy correction.
- `cargo build --workspace --locked --offline`: passed.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: passed at the final source.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base origin/main`: zero failures.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **136 passed**. The logged timeout/failed report is intentional negative-fixture output within passing tooling tests.

[Full final Rust log](workspace-tests.log.gz) preserves the test results.


## Retained initial failures

The first compile lacked ALSA development metadata; after installing it, linking
exposed the sysroot library path, which was corrected. The initial workspace run
found four historical checkpoint tests using the newly restricted normal entry
point; they now explicitly use the fixture entry point. That superseded run was
terminated before rerunning. Clippy found a default-then-field-assignment in the
new test; initialization was corrected. Xvfb initially could not bind its socket
under the restricted executor; the network-enabled executor supports it. The
first renderer launch timed out before step 1; the subsequent bounded Mesa run
and its actual results are recorded below. These initial failures are not passes.

## Rendered production evidence

`LP_NUM_THREADS=2 MESA_SHADER_CACHE_DIR=/workspace/salimon/artifacts/mesa-cache /workspace/.salimon-tools/start-display.sh scripts/salimon-test run scenarios/evidence/universe-journey.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary --artifacts artifacts/issue-427-retry`: **25/25 passed**, 110.299 seconds.

Original native captures were inspected: [arrival](step-004-published-start.png),
[rear approach](step-017-approach-rear-door.png), [inside cabin](step-024-entered-ship.png).
[Result](native-result.json), [lossless protocol](native-protocol.jsonl.gz) and
[binary/capture hashes](evidence-hashes.json) retain source and measurement context.
The test uses protocol gameplay input and production wgpu, not physical input.
