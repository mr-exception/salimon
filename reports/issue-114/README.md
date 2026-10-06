# Issue #114 — portable physical-object simulation

## Outcome

Extracted the existing custom fragment solver now, into `client/physics`
(`salimon-physics`), rather than deferring until another loose-object feature.
Its only dependency is `salimon-math`; there are no platform, renderer, character,
ship, catalog or resource-session dependencies.

Physics owns gravity, release/ejection impulses, spherical pair contact,
restitution, friction, hull/deck response and deterministic substeps. Runtime
retains nearby/carried selection, stable fragment order, environmental geometry,
ship frame projection/conversion and validated session writeback. World retains
identity, source, mass, material and orientation. Character supplies the existing
pure authored floor-containment query. Constants, arithmetic order, three pair
passes and the 48-substep cap are retained.

The [ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114)
defines reuse for compatible cargo, equipment and debris and prohibits unrelated
physical rules accumulating in runtime. Added crate maintenance/architecture/
invariant guides and updated runtime, client, coding and validation guides.

## Validation

Environment: Linux x86_64, rustc 1.99.0, existing local Cargo dependency cache.
Commands ran from repository root with
`CARGO_TARGET_DIR=/workspace/scratch/deb16f8b0e72/salimon/target` to reuse compiled
artifacts; `--offline` uses the local cache and `--locked` preserves resolution.

| Command/check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test --workspace --locked --offline` | Passed: 260 tests, 1 existing ignored renderer manual benchmark |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Passed |
| `cargo build --workspace --locked --offline` | Passed |
| `git diff --check` | Passed |
| Relative Markdown path existence in changed/new guides | Passed |

Eight physics tests cover normalization thresholds, impulses/variants, restitution/
friction, floor-bound rejection, radial settling, deterministic coincident-object
stacking, support-frame separation, zero duration and the large-delta substep cap.
Existing runtime deck-settling, stacking and surface-ejection tests remain intact.
Two additional adapter tests verify carried/distant exclusions and rotated/translated
ship-frame writeback while preserving identity/source/material/orientation.

## Limits

Native graphical scenarios were not run: this environment lacks Xvfb and an
installed Vulkan ICD. CPU checks do not establish live GPU or hardware performance.
No appearance/asset changes were made, so no new screenshot is supplied.

This remains a small equal-weight spherical solver, without rigid-body mass
weighting or spin; large deltas can exceed the nominal 1/90 s substep because of
the retained 48-step cap. Multiple ship-local frames require separate calls.
Character locomotion and ship flight remain in their existing domains. No full
physics engine, WASM target build or backend implementation was added.
