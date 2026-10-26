# Issue #643 — renderer and native presentation owners

Extracted GPU construction into `client/renderer/src/initialize.rs`, drawable
resize/configuration into `surface.rs`, and ordered preparation, passes,
capture/submission/present and observations into `frame.rs`. Renderer resources,
public DTOs/errors and pure camera/scene helpers remain in the facade. Runtime
redraw is in `app/presentation.rs`; its `prompts.rs` child owns contextual priority
and anchors. `app/scene.rs::gameplay_ship` owns Scout observation mapping. Native
lifecycle, input, clocks and recovery stay in app. No API, shader, layout, draw
order, numerical rule or domain dependency changes.

[Metrics](metrics.json) compare the same pinned 30-PR history: renderer facade
1,623 → 1,007 lines (63 → 39 branch tokens); app 2,046 → 1,476 lines
(311 → 225 branch tokens). New owner sizes/branches are included so moving code
does not conceal complexity. Counts establish clearer owners, not measured
future merge-conflict reduction. The three #645 candidates are the two existing
broad facades and the new scene adapter's existing permitted ship reference;
there are no new crate cycles or forbidden domain directions.

## Validation

Linux x86_64; Rust 1.99.1; fixed-step protocol input; Xvfb 1280×800 scale 1;
Mesa llvmpipe software Vulkan, LP_NUM_THREADS=2. Exact source parent, binary
hashes and image equality are in [evidence](evidence.json).

- `cargo test --workspace --locked`: **1,031 passed, 38 existing opt-in ignored**, zero failures.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base codex/issue-645-architecture-ci`: zero failures.
- `python3 scripts/architecture_review.py --base codex/issue-645-architecture-ci --output artifacts/architecture/643-review.json`: passed, warning-only candidates assessed above.
- Production `lower-cockpit-windows.json`: **24/24 steps**, all **8 originals byte-identical** to the parent.
- Production `construction-interlocks.json`: **21/21 steps**, all **3 originals byte-identical** to the parent, including Stop/Resume and physical cancellation refunds.

For each parent/current run use:
`DISPLAY=:91 SALIMON_E2E_ISOLATE_INPUT=1 python3 scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json --binary BINARY --artifacts OUTPUT --screenshot-command '["native"]' --summary`.
Repeat with `scenarios/evidence/construction-interlocks.json`. Set the Vulkan ICD
and local library paths to the prepared Mesa installation. Build prerequisites
follow [validation](../../docs/validation.md#native-graphical-gates). The original
compressed results and PNGs are retained in [flight](flight/current-result.json.gz)
and [construction](construction/current-result.json.gz); parent results are beside
them. [Build](643-build.log), [Clippy](643-clippy.log) and [tests](643-final-tests.log)
retain actual checks. [Environment](environment.txt) records toolchain/context.

Initial missing Rust/ALSA metadata and local library/surface setup failures were
repaired with workspace-local tooling/packages. Extraction compile errors
(missing imports/private type path) and an item-after-tests lint were corrected
before the final gates. The [initial compile log](initial-compile.log) is retained;
no failed setup is reported as successful evidence. The early CLI attempt used
an unsupported `--output`; final runs use the documented `--artifacts` option.

## Limits and follow-up

Software Vulkan images establish these deterministic scenes, not Metal, physical
input/display/OS lifecycle, subjective appearance approval or hardware FPS. Human
appearance review remains [#561](https://github.com/mr-exception/salimon/issues/561),
physical platform checks [#557](https://github.com/mr-exception/salimon/issues/557),
and hardware budgets [#563](https://github.com/mr-exception/salimon/issues/563).
No baselines/masks/tolerances were promoted. No performance timing is inferred
from captured route duration. No merge performed.
