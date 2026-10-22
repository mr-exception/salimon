# Nightly verification repair — 2026-10-22

## Outcome and source evidence

- [Nightly 39727064767](https://github.com/mr-exception/salimon/actions/runs/39727064767/job/119653365938): workspace build/tests passed; evidence failed at landing cutoff step 102 (old 19.84 m/s expectation) and cockpit monitor step 36 (waiting for landing after thrust now produced 240 m/s).
- [Nightly 39548321491](https://github.com/mr-exception/salimon/actions/runs/39548321491): gameplay, evidence and temporal checks passed; reviewed appearance failed with `incompatible visual corpus driver`.

## Changes and ownership

- Landing screenshot route matches the maintained portable baseline: one forward tick gives 16 m/s, two give 32 m/s, two reverse ticks restore 16 m/s. The 20 m/s production landing limit is unchanged.
- Cockpit monitor preserves its five named screenshot poses and 25% thrust display, using two ticks (8 m/s). It asserts assist admission immediately, advances a bounded 850 ticks through approach/landing and asserts landed state/zero speed instead of waiting on an impossible transition.
- Runtime `flight_regression` replays both real evidence JSON routes through production session commands; screenshot markers do not imply CPU pixel validation. Python suite tests protect landing baseline/evidence parity.
- Nightly/comparator select a new explicitly pending software-Vulkan v2 corpus. The approved v1 images/metadata, compatibility checks, pixel thresholds and masks remain untouched. Fixture actions changed, so reusing v1 approval would be invalid regardless of driver compatibility.
- [Review #518](https://github.com/mr-exception/salimon/issues/518) owns independent current-context capture/image/mask/tolerance approval. Pending coverage remains pending in reports, not a full nightly pass. Temporal, evidence and packaged-input failures still fail.

## Validation

- `python -m unittest discover -s scripts -p 'test_*.py'`: passed, 105 tests (Python 3.12.14, Pillow 12.3.0). This includes landing baseline/evidence parity, strict visual corpus compatibility/integrity and appearance pending/failure precedence contracts.
- `python scripts/check_docs.py --base origin/main`: passed, zero documentation failures.
- `git diff --check`: passed.
- Rust fmt/Clippy/tests and graphical captures: not run locally; Rust/Xvfb/Vulkan are not preinstalled. Package installation was attempted but did not complete in the restricted environment. PR CI validates the portable Rust tests; post-merge forced nightly is still required for native screenshot/temporal/smoke verification.
- Independent software appearance approval: pending #518. No Metal/hardware visual or performance signoff is claimed.

## Documentation maintenance

This changes verification fixtures/tool defaults, not runtime mechanics or U01–U34 behavior. Owning runtime architecture/invariants and ship engine/landing contracts are unchanged. Updated `docs/validation.md`, `docs/periodic-ci.md` and `scenarios/visual/README.md` map the affected commands, scenarios, baseline transition and review limitation. No generated assets or historical evidence are modified.
