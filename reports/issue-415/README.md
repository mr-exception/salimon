# Issue #415: Time-addressable ephemeris models

The `ephemerides` module evaluates typed linear stellar motion, explicit elliptic two-body models and cubic Hermite interpolation of imported Horizons samples. Samples retain TT/TDB epochs; UTC arithmetic and implicit time conversion reject. Extrapolation is off by default and requires a finite explicit linear horizon, with a lower-accuracy label. Generated orbits require explicit phase, epoch and right-handed orientation; absent U10 fields are never invented. Typed models can be loaded from checksum-verified pack sidecars. Body tracks build instantaneous U02 frame trees with declared axial spin and transport velocity; cyclic/disconnected frames reject. Exact Horizons endpoints and analytic orbits are tested. Independent intermediate-date JPL model-error calibration, complete release data and production quality acceptance remain unrun; interpolation is a method, not a claimed high-quality JPL replacement.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-universe --locked`: 80 passed, zero failures.
- Exact imported Earth states, analytic Kepler endpoints, interpolation/fallback boundaries, scale errors and rotating-frame/cycle checks pass.
- All-target universe Clippy with warnings denied and docs/base-impact checks pass; formatting applied.
- No independent intermediate-epoch JPL error budget or live playable integration is claimed.
