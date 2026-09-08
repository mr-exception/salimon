# Issue #470 — recovered transmissions and physical investigations

Implemented the two location-bearing recovery payloads and repeatable voluntary
investigations. Complete Core-funded reconstruction admits distinct distress and
remote-warning leads. Partial work supplies no coordinates. Senders, chronology,
“trap,” and the proposed passage near Absenat remain claims; neither packet nor
physical inspection proves that the traveler entered a wormhole. There are no
additional messages, required quests, NPC encounters or Absenat/Earth fixes.

## Production changes

- `session/narrative.rs` owns stable version-1 Vesper site IDs/ground anchors,
  minimal inspectable relay geometry and separated physical observations/message
  claims. The near site is 65 m from the landing-chart origin; the remote origin
  is about 14 km away. They exist independently of recovery and run seed.
- Paid Recovery completion publishes exactly one uncertainty-bearing source per
  site. Reopening Archive admits previously completed legacy packets. Existing
  recipe, Core reserve, no-refund work, interruptions and irreversible gaps remain.
- P/M/Enter can explicitly annotate an **UNVERIFIED** same-frame investigation
  route. It snapshots the exact lead/source and never creates scanner knowledge,
  autopilot or a confirmed destination. Generic unobserved celestial leads retain
  the earlier UI restriction. Recovered routes survive saves and contact expiry.
- Aimed, unobstructed surface E inspection within 3 m opens a modal reader and
  retains the physical clue with its Archive packet. Pause/terminal/world gates,
  held-input clearing and revisit behavior are exercised through production code.
- Save schema 31 records versioned first-inspection timestamps, omits the reader,
  validates exact packet/paid-work/lead binding, rejects altered coordinates or
  future timestamps, and initializes absent legacy sources. Inspection changes
  enter discrete checkpoint detection; current missing source rejects atomically.

Canonical behavior and owner routing are in
[opening/investigations](../../docs/opening-and-investigations.md), runtime
[architecture](../../client/runtime/architecture.md) /
[invariants](../../client/runtime/invariants.md), universe
[chart invariants](../../client/universe/invariants.md) and the
[save contract](../../docs/save-contract.md#narrative-inspections--schema-31-470).

## Cloud evidence

[Native evidence/state](native-evidence.json) binds nine original 1280×800,
scale-1 production GPU captures to Vulkan/llvmpipe Mesa 25.0.7, seed 0, 16 ms
protocol input. The opening and both exterior site routes wait for
`terrain.presentation_ready=true` before capture. Archive/chart captures exercise
physical UI/overlays; their terrain streaming state is recorded without claiming
exterior terrain acceptance from those views.

- [Distress site](narrative-distress-physical-relay.png) and
  [physical clue](narrative-distress-inspected-clue.png).
- [Separate warning origin](narrative-warning-physical-relay.png) and
  [physical clue](narrative-warning-inspected-clue.png).
- [Recovered distress](narrative-recovered-distress-payload.png) and
  [warning](narrative-recovered-warning-payload.png).
- [Uncertain near route](narrative-chart-uncertain-route-1.png) and
  [remote route](narrative-chart-uncertain-route-2.png).

The production-control regression actually walks from the canonical arrival to
the nearby recovered relay and inspects it. The remote native route declares
initial placement; this driver does not claim to have flown the 14 km approach.
The existing canonical mine/carry/Fabricator/disk-restore regression also passes.
Recovered native fixtures declare 2 kg each iron/silicate, then use actual
Fabricator installation and paid Recovery funding; they do not establish earned
acquisition or balance. Existing flight/world behavior is reused unchanged.

## Validation and retained failures

Parent/base: `main` at `3bcf3bf977108408e4a3ba9d855aef0051370515`.
Cloud: Linux x86_64 / glibc 2.41, 3 logical CPUs, Rust 1.99.1, locked Cargo inputs,
Xvfb and Mesa software Vulkan. [Relevant source hashes](source-hashes.json) and
[repaired test outcomes](repaired-tests-final.json) retain exact coverage.

```sh
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all -- --check
cargo test --workspace --locked
cargo test -p salimon-client --locked --no-run
# Rebuilt runtime test executable: exact reruns of the 24 failed cases, then:
target/debug/deps/salimon_client-a1438275ed68383a session::narrative
cargo test --workspace --exclude salimon-client --exclude salimon-character --locked
python -m unittest discover -s scripts -p 'test_*.py'
python scripts/check_docs.py
python scripts/check_docs.py --base origin/main
git diff --check
```

Build, all-target Clippy, formatting, 156 script tests and docs/link checks pass.
Rust coverage totals **1,293 normal cases passed**: 77 character, 545 initial
runtime passes plus 24 repaired exact runtime cases, and 647 remaining package
cases. The normal commands retain 52 opt-in ignores; the opening PCM opt-in was
also executed successfully. The first workspace command was **not green**:
24 failures exposed legacy-source admission, generic-lead routing and three
schema/empty-source expectations. Those defects/expectations were corrected;
all 24 exact reruns and all five narrative regressions pass. Slow unchanged
flight tests were not repeated after these isolated fixes. PR CI verifies the
complete current published revision; do not relabel the initial command a pass.

Production GPU command (after Rust checks, on an otherwise quiet executor):

```sh
WGPU_BACKEND=vulkan \
VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json \
xvfb-run -a -s '-screen 0 1280x800x24 -fp /workspace/.salimon-tools/sysroot/usr/share/fonts/X11/misc' \
python scripts/verify_narrative.py --binary target/debug/salimon-client --output artifacts/narrative-470-471-final
```

Passed all five declared setups / nine captures. Earlier driver attempts retained
an unknown key alias, a stale Xvfb socket, and a long step timeout while Rust/GPU
work contended. The alias and fixture notification setup were corrected and the
final driver uses bounded ordinary steps with terrain readiness; no numerical,
physics or acceptance threshold was relaxed. The original single-line chart
text fits the 1280-pixel capture and exposes source, uncertainty, bearing/range
and route status; no human visual approval is implied.

## Boundaries

Originals establish this Linux production path, not OS-injected/physical input,
Metal/M1 behavior, hardware FPS or subjective art/audio approval. Those remain
#720–#723/#661. No observed-source or full-Gaia acceptance is implied: the
separate [#524 report](../issue-524/README.md) explicitly retains its full-release
selection/storage blocker. This is one combined PR at the user's request; #470
and #471 close through merge, while #524 remains open.
