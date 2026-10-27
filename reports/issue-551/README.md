# #551 — integrated Scout and construction cloud verification

Base: `ee37ca4c40ca99fbf3d38f39faed1e3095b9798f` (current main).
Branch: `codex/issue-551-construction-e2e`; PR targets `main`.

## Change and acceptance coverage

The maintained `scout-construction` suite selects 32 explicit production-native
capture routes, failing on missing scenarios rather than silently shrinking
coverage. Three new routes fund an approved hull cell from an explicitly seeded physical
20 kg iron fragment and advance ordinary work to exactly 0/50/99%, cancel through the
normal mouse route, and require one physical 20/10/0.2 kg refund with no installed
colliders. A complementary funded-stool Rust regression checks each boundary across save/reload, unchanged
structure, exact mass accounting and no repeated refund after a second reload.
No game mechanics, asset geometry, numerical contract or approved budgets change.

| Acceptance | Current route / owning contract |
| --- | --- |
| Cockpit/flight/Core/cargo/doors/ramp and carrying | starter-layout, cockpit-nose, inertial-flight, core-restart, cargo-door, boarding-ramp, fragment-transfer; supplemental #489 machinery driver |
| Fabricator/Archive/power | supplemental #489 Fabricator/blackout and #505 all-record/power drivers |
| Habitation/galley/physical fixture use | habitation, habitation-use, cabin-bunk/table/locker, room-finishes, room-lighting-blackout; hygiene retains inspection-only contract |
| Placement/intake/partial/completed and no early colliders | blueprint-placement, construction-materials/build/cancel-0/50/99 plus runtime construction regressions |
| Multi-cell/two-deck traversal, boarding and structural collision | construction-room; room/flight regressions |
| Module downtime/upgrade/cancel/restoration | construction-module, construction-module-upgrade/restore and module regressions |
| Damage/salvage/detachment | construction-damage/detachment and damage regressions |
| Stop/takeoff/refunds | construction-interlocks and construction-interlocks-cockpit, exact cancellation routes and flight regressions |
| Save/offline/atomic replay/mass/cargo | construction-offline/offline-upgrade, new boundary regression and existing checkpoint/offline owner tests |
| Asset budgets/authored geometry | Scout validator checks GLB/interchange/preservation, generated spatial contracts and canonical manifest metrics |

## Verification

Final environment: Rust/Cargo 1.99.1, Python 3.12.14, Linux x86_64,
AMD EPYC 7763 with a 2-CPU/8-GiB cgroup. Native output uses Xvfb :101,
Mesa llvmpipe LLVM 19.1.7, Vulkan CPU device, 1280×800 at scale 1. Archive
also verifies 1920×1080 at scale 1. Other display/DPI matrices were not rerun.
Exact binary hash, platform and changed-source hashes are in
[environment.json](environment.json) and [source-manifest.json](source-manifest.json).
The recorded base plus those source hashes identifies the verified working tree;
report/evidence packaging follows verification. Functional checks overlapped some
compilation; their timing is not hardware-performance evidence.

| Command (repository root) | Actual result |
| --- | --- |
| `cargo build --workspace --locked --offline` | Final build passed |
| `cargo test --workspace --locked --offline` | Final run: 1,033 passed, zero failed, 38 existing opt-in ignored |
| `cargo test -p salimon-client --locked --offline cancellation_boundary_routes` | Passed after the last structural-collider assertions were added to the JSON routes |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Final run: 131 passed |
| `python3 models/assets/ships/salimon-scout/validate.py` | Passed: 6,767 triangles, 134 primitives, 13 materials, 581,380 GLB bytes, 116 texture bytes; preservation/interchange/generated spatial checks pass |
| `python3 scripts/check_docs.py` and `--base origin/main` | Zero findings |
| `git diff --check` | Passed |
| `bash reports/issue-551/run-native.sh` | 32/32 integrated routes, 1,400/1,400 steps; four machinery lanes (56 captures); Archive reader (17 captures); Archive blackout/restoration (4 captures) all passed |

Build setup: `. /workspace/.salimon-tools/env.sh` plus
`PKG_CONFIG_SYSROOT_DIR=/workspace/.salimon-tools/sysroot` for native compilation.
[run-native.sh](run-native.sh) records the exact native command sequence and owns
its fresh Xvfb lifecycle. It requires fresh output directories. The maintained
suite and supplemental commands are documented in [runner usage](../../scripts/README.md#integrated-scout-and-construction-acceptance-551).

### Original evidence and corrected failures

[All 179 original PNGs](checkpoints.md), complete per-step states, protocol and
process logs are retained under `evidence/`. JSON/log traces are gzip-compressed
without content changes. [retain.py](retain.py) verifies every decompressed
SHA-256 against its original; each directory manifest maps original and retained
paths/hashes. Absolute paths inside original traces describe their capture context.
[checks](checks/manifest.json) retains actual command output, including failures.
The [six-second Fabricator clip](fabricator-stepped.mp4) is a derivative of twelve
original work images sampled every 0.5 simulated seconds, encoded at 2 fps with
FFmpeg/libx264 (CRF 18, yuv420p). It is silent, stepped evidence, not real-time or
physical-FPS qualification. Original pixels remain in the PNGs.

- The first offline build failed on uncached sha2. A normal locked network build
  fetched missing crates, then failed on missing ALSA development files. Downloaded
  libasound2-dev/libasound2t64 from the configured Debian source into the existing
  local sysroot; the final locked offline build passes. Those setup failures remain.
- Initial integrated run: 30 routes passed; the two ten-second stool cancellation
  routes failed exact 50/99% assertions at 49.92/98.88%, because live work advances
  on 16 ms ticks. [Original failed states/captures](evidence/initial-integrated/manifest.json)
  remain. The final fixture declares the existing approved 120-second hull cell,
  whose 60/118.8-second boundaries align exactly. It changes no catalog duration,
  scheduler, numerical tolerance or expected percentage. Ordinary intake/work/
  cancellation now verifies exact 20/10/0.2 kg refunds, absent structural and
  installed colliders, and no carried remainder at 0/50/99%. The direct stool
  save/reload regression also passes at exact boundaries.
- The next launch attempt failed all 32 routes before readiness because the old
  X99 process had exited, leaving a stale socket. [Launch failures](evidence/display-failure/manifest.json)
  remain; the final script owns a fresh X101 server and all routes pass there.
- One intermediate 130-test Python run failed because the invalid-PNG case reused
  the timeout case's 100 ms capture budget under load. Split them into independent
  cases: the timeout check still enforces 100 ms; the invalid-PNG check uses the
  normal budget and still rejects the malformed file. The final 131-test run passes.

The owning [runtime fixture guide](../../client/runtime/README.md#integrated-construction-cancellation-fixture-551),
[validation matrix](../../docs/validation.md#integrated-scout-and-construction-acceptance-551)
and runner usage are updated. Runtime architecture/invariants remain applicable:
only opt-in initial conditions and verification change; later actions still use
production commands, material ownership, work, collision and persistence owners.

## Remaining ownership and next task

Human spatial proportions, preview readability, original image/audio appearance
and normal-play perception remain [#561](https://github.com/mr-exception/salimon/issues/561).
Physical keyboard/mouse/display/DPI, real devices and OS sleep/wake remain
[#557](https://github.com/mr-exception/salimon/issues/557). Actual GPU frame-time,
Metal/M1 compatibility and performance, stress thresholds and asset-budget
approval remain [#563](https://github.com/mr-exception/salimon/issues/563).
Fixed-step protocol/software-Vulkan evidence establishes none of those approvals;
missing hardware counters remain unavailable. No image baseline is promoted.

The next requested task [#413](https://github.com/mr-exception/salimon/issues/413)
requires reviewed production sources and approved snapshots under
[#537](https://github.com/mr-exception/salimon/issues/537), with #524/#526 scientific
acceptance. Private fixtures do not satisfy pack-publication acceptance. No
production rights or snapshots are inferred from this implementation request.
Consequently #414 → #415 → #416 remain blocked behind publication; no empty or
premature implementation PRs are created for that continuation.
