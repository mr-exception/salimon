# Issue 626 — personal planetary chart

Parent: [PR #707](https://github.com/mr-exception/salimon/pull/707),
`issue-625-suit-observations`, exact head
`74c7cb539a0b631d9c39b680b3214310c7ba82b1`. Its published-head CI passed;
#611/#469/#406 and required #625 code are present in this ancestry. No merge performed.

## Delivery and ownership

The powered physical Scout projector reviews personal suit marks, position error
volumes and bearing cones. Backslash opens review; M cycles; R/native text edits
notes; Enter saves or selects a manual target; Delete hides a personal mark while
retaining history; Escape discards a draft before the host pause route. Targets
require measured locations or retained physical-inspection receipts. Unknown
range never turns into a guessed target; angular uncertainty of 90 degrees or
more says UNCONSTRAINED and draws no misleading tangent cone. Relative navigation
uses a quantized current ship reference and adds its error to observation error.

Universe owns immutable observation consumers, annotation/inspection history,
tombstones and target receipts. Runtime owns physical ray/part/terrain source
admission, current-publication validation, controls, text and projector mapping.
Schema 28 pairs the chart with the suit book, initializes legacy personal entries
and rejects missing current sources, duplicate/dangling IDs, malformed text/time,
forged target locations and incompatible inspected sources before publication.
No catalog POIs, Earth coordinates, scanner modes or autopilot are introduced.

[Maintained contract, limits and source/test map](../../docs/planetary-chart.md).
Owning runtime/universe/persistence guides, usage, save, routing, runner and
validation documentation are updated in this PR.

## Actual verification

Debian Linux x86_64; Rust/Cargo 1.99.1, Python 3.12.14. Commands use
`source /workspace/.salimon-tools/env.sh`. Native evidence is production wgpu,
Xvfb, Mesa 25.0.7 llvmpipe Vulkan at 1280x800 with LP_NUM_THREADS=2; fixed 16 ms
protocol steps, not physical keyboard/device or M1 performance evidence.

- `cargo test --workspace --locked --offline`: final **1225 passed**, zero
  failures, 49 existing opt-in ignored ([full log](workspace-tests.log.gz)).
  Includes three chart domain tests, four live-session/geometry/terrain tests and
  the native-dispatch Escape/focus regression. Positive close inspection uses
  an actual published terrain landmark fixture; its physical source is unchanged.
- `cargo build --workspace --locked --offline`: final pass.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`:
  final pass. `cargo fmt --all -- --check`, `git diff --check`: pass.
- `python3 scripts/check_docs.py --base issue-625-suit-observations`: zero failures.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **136 passed**;
  deliberate timeout/failure fixture output is retained within passing tests.
- `LP_NUM_THREADS=2 MESA_SHADER_CACHE_DIR=/workspace/salimon/artifacts/mesa-cache /workspace/.salimon-tools/start-display.sh scripts/salimon-test run scenarios/evidence/planetary-chart.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary --artifacts artifacts/issue-626-complete`:
  final **42/42 passed**, 98.792 s. Covers editing/Backspace, measured target,
  unknown-range rejection, Escape/discard without pause, deletion, exact reload
  and durable personal history. Capture waits for three completed rendered samples.
  [Result](result.json.gz), [lossless protocol](protocol.jsonl.gz),
  [binary/capture hashes](hashes.json).

Final originals were inspected: [spot error volume](step-007-spot-uncertainty.png),
[named manual target](step-018-named-manual-target.png),
[bearing cone and rejection](step-031-bearing-cone.png),
[deleted marks](step-042-deleted-personal-marks.png). Context stays visible beside
transient feedback, within the viewport and above the equipment reservation.

## Initial corrections and honest limits

Initial compile checks caught optional-inspection-to-history conversion mistakes;
fixed before passing verification. The runner initially rejected the unregistered
fixture and text operation before starting the game; both are now registered.
An initial 27-step route passed state assertions but captured an older overlay
and feedback that obscured context. The source now combines chart context with
feedback, and evidence explicitly waits for rendered samples. Later 35-step
runs passed; final evidence was regenerated after relative-ship error and native
Escape corrections. Initial failures/stale captures are not promoted as evidence.

Vesper's existing frozen terrain/site inventory is preserved, not populated for
a demonstration. Positive physical-source inspection is covered with real
published terrain data through the receipt owner; no visited-Vesper-site capture,
new world-selection integration or scientific identification is claimed. The
explicit native fixture creates marks through walking/suit commands, then sets
cockpit review initial conditions; it does not claim that teleport is boarding.
Actual physical input/platform (#557), subjective presentation (#561) and actual
M1/Metal budgets (#563) remain separately unverified release qualification.
