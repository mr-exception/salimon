# Issue #430 — Physical cockpit hologram

Added a low-poly procedural projector on the existing console. Three orthogonal
rings, ship-origin marker, selected contact cubes and elevation stems have actual
3D depth through the production instance/depth pass. Angular layout remains 3D;
radial distance is logarithmic and explicitly labeled. The cockpit hint reports
actual sampled center-distance and age. The existing navigation/flight monitor
remains visible. Geometry is bounded to 64 contacts/227 cuboids, including a
selected contact beyond the first page. No new GPU pipeline or collider authority.

M cycles fresh identified contacts through the ordinary native/automation input
path, only in an admitted canonical cockpit. Unknown/out-of-frame states are not
invented. Selection is transient, changes no scan/flight/source state and rejects
pause/terminal/modal/outside-cockpit input. Power loss suppresses projected
geometry while retaining encounter history. Original screenshots are evidence
at this revision, not promoted baselines or human appearance approval.

See [scanner contract/source mapping](../../docs/universe-scanner.md), renderer
and runtime architecture/invariants. Source owners: renderer
`scanner_hologram.rs`, runtime `session/scanner/presentation.rs`, ordinary native
scene/input/prompt composition. The `scanner` engineering reference camera stays
separate from the approved four-camera Scout lighting corpus.

## Production captures

Cloud Linux x86_64, Rust 1.99.1, Xvfb 1280x800 scale 1, Mesa lavapipe/software
Vulkan with two llvmpipe threads. The existing `canonical-flight` fixture places
the player in a landed canonical cockpit; protocol input does not establish a
physical keyboard or normal-play boarding route. Five days of the existing
quiet offline power adapter drain the actual Core and expire fresh contacts;
this is not orbital/scanner catch-up or physical sleep/wake qualification.

- [Undiscovered housing](undiscovered.png): no prior catalog reveal.
- [Identified local contact](identified.png): actual paid scan, selected Vesper
  center/contact, 3D rings/stem and readable sampled distance/age.
- [Cockpit coexistence](cockpit.png): projector and original flight instruments.
- [Blackout housing](blackout.png): actual exhausted Core, no projected rings/contact.

`verify_images.py` (Pillow) checks a fixed declared projection ROI for absent
initial/blackout projection and visible cyan/selected-amber pixels. It reads
originals and never edits images, expands a baseline mask or approves appearance.
`image-metrics.json`, `capture-context.json` and compressed full native results
retain exact evidence/context/hashes. Geometry/input/source tests supply separate
contract coverage; pixels alone do not prove gameplay correctness.

## Corrected failures and limits

An input test initially tried to enter the cockpit directly from the exterior;
fixed the test's declared inside fixture rather than bypassing production entry.
The first Python corpus check caught accidental addition to the approved lighting
camera inventory; moved the engineering camera into its separate runtime seam,
retaining the approved corpus unchanged. A repeated capture hit a stale X socket;
used a dedicated fresh Xvfb lifetime for the route. The first unknown-state image
was obscured by the pause menu; moved capture after resume without advancing the
simulation. Preliminary results are retained separately and not final evidence.

Human appearance/readability #561, physical devices/platform #557 and actual GPU/
M1/Metal/hardware budgets #563 remain unverified. Software timings are diagnostics,
not a 60 FPS or hardware qualification claim. Scanner global-frame adoption and
nonlinear closed-game propagation remain their existing #427/#432/#433 owners;
this projector only consumes actual same-frame discoveries.

## Validation

- `cargo test -p salimon-renderer --locked scanner`: one projection/bounds/blackout test passed.
- `cargo test -p salimon-client --locked scanner`: 5 passed.
- `cargo test -p salimon-client --locked scout_reference`: one passed after camera separation.
- `cargo test --workspace --locked`: 1,124 passed, zero failed, 38 existing opt-in ignored.
- `cargo build --workspace --locked`, workspace all-target Clippy with warnings denied,
  formatting and `git diff --check`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 134 passed after the
  camera-corpus correction. Expected negative-path fixture messages are not suite failures.
- `python3 scripts/check_docs.py --base codex/issue-431-scanner-persistence`: zero failures.
- Dedicated Xvfb production native route: **28/28 steps passed**, 4 original captures.
- `python3 reports/issue-430/verify_images.py`: passed, metrics beside this report.
