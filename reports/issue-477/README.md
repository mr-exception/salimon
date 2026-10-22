# #477 — Current Metal corpus and normal-clock presentation review

## Outcome and review boundary

Recaptured the full current Metal candidate corpus and recorded normal-clock
walking, coasting/steering, EVA, doorway exit/re-entry and cargo pickup/carried
movement/clear-floor drop on the reference desktop's two displays. Actual OS
input delivery and checkpoint assertions pass. **Owner image/mask/tolerance and
full-rate smoothness/latency approval remain pending** in
[#561](https://github.com/mr-exception/salimon/issues/561). No candidate was
promoted and no baseline, mask, tolerance or software-Vulkan approval was changed.

The pending Metal descriptor now proposes the **observed scale2** reference
context instead of its stale scale1 value. It remains `pending`; this is a
review-context correction, not image approval. The explicit
[review proposal](review-proposal.json) binds all current image hashes and the
unchanged empty masks, channel8/255 and changed fraction0.001 to pending owner
review. Reviewer/time/evidence remain null until an actual decision is recorded.

## Exact source, machine and images

Collected 2026-10-22/23 on iMac21,1 / Apple M1 /16 GiB /macOS26.2 (25C56), AC,
Metal. The built-in display is nominal59.995Hz/2×; external is nominal74.973Hz/1×.
[Actual display inventory](evidence/display-modes.json). Physical120/144Hz is
unavailable and is not claimed. Adapter driver text remains the API's `" / "`.

Production source is #455 commit `3111366f4e9a5fd1d3559b68aef0e3e636641331`;
[binary identity](evidence/live/primary/provenance.json) matches the
[candidate corpus](candidates/corpus.json). The later cargo checkpoint exporter
is `#[cfg(test)]` only and changes no production input, save schema, renderer,
physics or interpolation behavior. It reuses the existing six-piece FragmentPile
fixture, production tick/aim commands and two verified save slots. Exact
roundtrip, absence of E2E policy after restore and target-ID pickup are checked
before export. [Fixture metadata and slots](evidence/cargo-fixture/).

[128 original PNGs](candidates/) cover all nine fixtures and 38 distinct byte
contents. Direct color-surface images are physical1280×800/scale2, with fixed
16ms steps and desktop-input isolation. They **disable interpolation**. Review
sheets cover every distinct content: [1](review-sheets/sheet-1.jpg),
[2](review-sheets/sheet-2.jpg), [3](review-sheets/sheet-3.jpg),
[4](review-sheets/sheet-4.jpg); use original PNGs for any approval. Agent sheet
inspection sees the intended cabin/canopy/glass, Core, tool/deposit, pile/floor,
ramps and physical screen contents. The prominent HUD/toolbar remains an owner
art/readability decision. None of this substitutes for human image review.

All128 PNG bytes are identical before/after #455's exhausted-row correction;
the final corpus nevertheless records that branch's clean source and actual
binary hash. **86 frozen-repeat comparisons pass**, zero changed pixels.
[Results](evidence/temporal/). Reviewed appearance comparison exits3,
[human approval pending](evidence/reviewed-pending/appearance.json), as required.
Persistent silhouette/material/layout defects are rejected against a current
candidate while repeating the same wrong image passes temporal comparison.
[Defect self-test](evidence/injected/results.json). This tests the comparator;
it does not approve its reference image.

## Normal-clock OS routes

These processes launch normally from copied validated checkpoints with isolated
`SALIMON_SAVE_DIR`; no `--e2e`, stdin stepping, fixed simulation clock or named
presentation override is active. Received native event traces, actual window
bounds, fresh checkpoint generation/current UTC, snapshots and clean exits are
retained. The helper verifies the foreground target PID before global input.
OS-injected controls remain distinct from physical human input.

| Route | Built-in 60Hz/2× | External 75Hz/1× |
| --- | --- | --- |
| Walk, positive mouse and inverse | [recording/state](evidence/live/primary/landed/) | [recording/state](evidence/live/external/landed/) |
| Coasting ship, steering, mouse inverse | [recording/state](evidence/live/primary/moving/) | [recording/state](evidence/live/external/moving/) |
| EVA motion and mouse inverse | [recording/state](evidence/live/primary/eva/) | [recording/state](evidence/live/external/eva/) |
| Open door, exit through doorway, return inside | [recording/state](evidence/live/doorway-primary/landed/) | [recording/state](evidence/live/doorway-external/landed/) |
| Pickup, walk carrying same ID, clear-floor drop | [recording/state](evidence/live/cargo-primary/cargo/) | [recording/state](evidence/live/cargo-external/cargo/) |

All ten final routes pass their recorded checks: positive and inverse mouse in
the first six; real movement/steering; door fully open and actual outside/inside
states; pickup/drop with exact fragment IDs and extracted mass preserved; wall
clock advance, exact paused-state freeze, recording creation and clean exit.
The cargo snapshots also show actual carried-player displacement. The external
window is wholly on the recorded external bounds and the runtime reports a
960×540 drawable after the observed scale transition; primary content is
1920×1080. OS window recordings include native decoration/shadow, so their pixel
dimensions differ from the direct-GPU baseline candidates.

Videos are encoded review copies, preserving full spatial resolution. Their
[original MOV hashes, encoding commands and stream metadata](evidence/videos.json)
are retained; original recordings remain in ignored artifacts. These bounded
clips expose the routes for review, not optical scanout, a75Hz video sample rate,
physical-input latency, representative performance or full-rate human signoff.
Native pause/capture transitions are visible, not edited out. #453's separate
lifecycle and physical-sleep limits remain in its report/#557; no sleep was
requested here.

## Retained failures

- The first external attempt used an older helper binary without `move` support.
  It failed explicitly before the external route. [Failure](evidence/live/failed-window-helper/).
  Rebuilding the already-reviewed AX movement helper from
  [window_input.swift](window_input.swift) fixes the harness selection; the
  final six functional display probes and both additional external routes pass.
- The first cargo route walked into the Core's vicinity; both native F drops
  were blocked and the same identity remained carried.
  [Primary](evidence/live/blocked-cargo-primary/cargo/),
  [external](evidence/live/blocked-cargo-external/cargo/). Their explicit
  `native_drop=false` remains a failure even though the initial report-local
  harness returned0. The final harness exits nonzero for any false check.
  Walking back and sideways onto clear floor then permits the native drop on
  both displays. No clearance, collision or drop policy was changed.
- Historical #314 external-input failures remain in its report. The newly
  received positive/inverse events establish delivery at this recorded context,
  not retroactive success of those older attempts.

## Reproduction and checks

Compile the bounded helpers using existing local permissions; they fail closed
when posting/Accessibility access is unavailable:

```sh
swiftc reports/issue-477/native_input.swift -o artifacts/input-477
swiftc reports/issue-477/window_input.swift -o artifacts/window-input-477
python3 reports/issue-477/live_review.py --binary /absolute/path/to/salimon-client --helper artifacts/input-477 --move-helper artifacts/window-input-477 --output artifacts/new-live-primary
python3 reports/issue-477/doorway_review.py --binary /absolute/path/to/salimon-client --helper artifacts/input-477 --move-helper artifacts/window-input-477 --output artifacts/new-doorway-primary --cases landed
SALIMON_NATIVE_CARGO_DIR="$PWD/artifacts/new-cargo-fixture" cargo test --release -p salimon-client --locked native_cargo_checkpoint_477 -- --ignored --nocapture
python3 reports/issue-477/cargo_review.py --binary /absolute/path/to/salimon-client --helper artifacts/input-477 --move-helper artifacts/window-input-477 --source-checkpoint artifacts/new-cargo-fixture/cargo/checkpoint-0.sav --output artifacts/new-cargo-primary --cases cargo
```

Add `--external` for the attached reference external display; its explicit
position is machine-specific and must be verified against the display inventory.
The live/doorway scripts use #376's validated initial fixtures, copied into fresh
output saves. None of the harnesses operates on the player's active save.
Capture/compare/defect commands use the existing visual runner and the
[new self-test](inject_defects.py); keep output directories new and candidates
separate from baseline paths.

[Checks](evidence/checks.json) records workspace tests/build/Clippy/fmt, the opt-in
exporter, Python syntax, documentation/base-impact and whitespace results.
[SHA-256 inventory](evidence/sha256.json) covers retained data. The runtime
architecture/invariants now distinguish normal-clock evidence from human
signoff, and the validation/visual guides link this boundary. #561 owns explicit
remaining human approval and any resulting baseline-review/remediation PR.
Source #477 closes only through delivery review/merge; acceptance is not inferred.
