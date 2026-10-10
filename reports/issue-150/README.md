# Issue #150 — authored deposit surface emission

## Outcome

Mining no longer assigns ordinal rows/columns beside deposits. Native creation
prefers the mining ray's first visible authored triangle, then exposed nearby
facets ordered by contact distance. Fragment convex support reserves full 2 kg
growth plus 5 mm clearance; radial terrain support, the player capsule and
existing fragment envelopes reject unsafe creation. Crowding pauses creation
without consuming deposit mass, fragment IDs or accumulating pending output.

World owns splitting, stable identity, material, mass and session state through
`extract_with_spawn`. Runtime supplies geometry/clearance and installs motion
only in successful new-identity callbacks. Physics provides outward/radial lift,
lateral escape for top faces, and bounded deterministic velocity scatter. Growth,
streaming/requery and missing/restored motion never trigger another impulse.
Pickup/drop and the existing #148 convex contacts remain the normal lifecycle.

## Validation

Environment: Linux x86_64; Rust/Cargo 1.99.0; Python 3.12.14.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: 300 tests passed (including all production scenario contracts).
- `cargo build --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 35 passed.
- `git diff --check`: passed.

Focused tests cover all 12 authored deposit variants on three radial axes,
initial/full masses, deterministic facet origins and full-growth clearance,
player exclusion, occupied-facet pauses/retries, unchanged growing/restored
identities/motion, outward motion and settling. Portable production scenarios
exercise mining, carrying, ship transfer, depletion and streamed restoration.
The new baseline asserts three pieces and conserved 5.76 kg after settling.

## Native sequence and limitation

The paired [baseline](../../scenarios/mining-emission.json) and
[evidence](../../scenarios/evidence/mining-emission.json) use production controls
with origin (1 frame), outward ejection (11 frames), repeated emission (180
frames) and settling (240 additional frames after release) checkpoints. They
are included in default baseline discovery and the resource-collection group,
so Linux CI's required evidence suite captures the sequence.

Attempted locally:

```sh
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test run scenarios/evidence/mining-emission.json --binary target/debug/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

**Blocked before renderer readiness:** this workspace denies X11 Unix sockets;
Xvfb cannot establish its ordinary display listener. A TCP-only Xvfb started,
but the game connection was also unavailable under the workspace networking
restrictions. No native screenshot/video was captured or reviewed here. The
visual acceptance criterion remains pending a working native/CI display; CPU
contracts do not establish rendering fidelity. No macOS/Windows GPU or reference
hardware performance claim is made. Asset regeneration/model tests are not
applicable: authored exports are unchanged.

To review on a working display:

```sh
python scripts/salimon-test run scenarios/evidence/mining-emission.json --binary target/debug/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

Promote reviewed origin/ejection/repetition/settling PNGs and their authoritative
states beside this report. The issue stays open for review/merge and completion
of native visual evidence.

## CI timeout correction

Run [183](https://github.com/mr-exception/salimon/actions/runs/38067902185)
passed Rust/Python/model/build gates on all three OSes. Linux native baseline
passed the new emission route and both pile fixtures, but five existing mining
scenarios exhausted the runner's default **shared 5-second action + inspection
budget** during 600-frame physics batches (9.6 seconds of simulated time).
Failures were request deadlines, not failed mass, interaction or pose assertions.

Mining, carrying, transfer, resource-loop and streaming baseline/evidence pairs
now explicitly allow 30 seconds for batches of at least 300 frames. Ordinary
actions retain 5 seconds, scenario deadlines are unchanged, and no frame count,
physics behavior or gameplay assertion was removed. The evidence pairing and
runner contract tests validate these scenario changes. Native CI will rerun on
the updated PR; local X11 restrictions still apply.

Run [184](https://github.com/mr-exception/salimon/actions/runs/38068699496)
then exposed the independent server-side 5-second reply/queue deadline: long
batches returned `command timed out` despite the longer runner deadline.
The server now gives valid protocol-1 batches of 300–600 frames the same bounded
30 seconds for queue expiry and reply waiting. Ordinary and invalid commands
retain 5 seconds. A focused regression covers the range endpoints, ordinary
requests, malformed JSON, wrong protocol and invalid frame counts/types.
Formatting, workspace Clippy and the focused deadline test passed; a fresh
full native CI run validates the coordinated deadlines.
