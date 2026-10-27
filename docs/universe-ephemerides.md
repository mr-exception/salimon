# Time-addressable ephemeris models (#415)

The `ephemerides` module evaluates typed linear stellar motion, explicit elliptic two-body models and cubic Hermite interpolation of imported Horizons samples. Samples retain TT/TDB epochs; UTC arithmetic and implicit time conversion reject. Extrapolation is off by default and requires a finite explicit linear horizon, with a lower-accuracy label. Generated orbits require explicit phase, epoch and right-handed orientation; absent U10 fields are never invented. Typed models can be loaded from checksum-verified pack sidecars. Body tracks build instantaneous U02 frame trees with declared axial spin and transport velocity; cyclic/disconnected frames reject. Exact Horizons endpoints and analytic orbits are tested. Independent intermediate-date JPL model-error calibration, complete release data and production quality acceptance remain unrun; interpolation is a method, not a claimed high-quality JPL replacement.

This is portable private-development implementation, separate from playable runtime adoption. Production source approval remains #659 and full-data acceptance #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

## Cloud model-error verification

A new unmodified official JPL Horizons Earth series at three-hour intervals is
frozen with acquisition URL and SHA256 under `reports/issue-415/cloud/`. Fitting
only six-hour endpoints and comparing the four withheld epochs gives <=0.226 m
position error and <=3.38e-8 m/s velocity error, within the explicitly declared
1 m / 0.001 m/s interpolation-model test envelope. Existing U03 exact-decoder
rounding budgets remain unchanged. The regression is part of universe tests;
`jpl_ephemeris_check` also emits the full U03 comparator results in release mode.

Sampled models now declare `max_interpolation_gap_seconds` (default six hours).
Larger gaps reject interpolation; exact original samples remain available.
Arbitrary dates use only an explicitly supplied valid analytic or bounded lower
accuracy model, never a silent frozen edge. These checks complete the U14 cloud
model/frame acceptance; full-release coverage remains separately #526 and later
playable-world migration is not U14 scope. No human verification is required.
