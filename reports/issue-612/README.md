# Issue #612 — terrain-bound canonical geology

Base: `57618415ad9c2cd301d7f0c0b2a89506956bf721` (main). Prerequisites:
#611 merged; #424 delivered in merged #675, present in base ancestry.

World now exposes immutable versioned geology definitions, full upstream-terrain
binding, stable province identities, spherical continuous regional fields,
ordered lightweight history, landform/substrate/layer/exposure observations and
strict decode/compatibility checks. The joint surface query uses exactly the
existing terrain evaluator. Vesper's original definition/resources/terrain remain
unchanged; representative Earth/Moon/Mars remain explicitly historical fixtures.

The [canonical contract](../../docs/planetary-geology.md) maps behavior to sources
and tests; owning world guides, root routing and approved-design status are updated.
No renderer/runtime code changes occur in this PR, so no graphical behavior or
platform performance claim is made. Surface appearance is the next issue #613.
Active geology, climate, biospheres, new terrain/resource migrations and measured
scientific geological reconstructions remain outside this issue. Conceptual
process extents and stratification are authored model values, not measurements.

## Verification

Environment: cloud Debian 13 x86_64, Rust/Cargo 1.99.1; workspace checks run in
an isolated Debian trixie container with GCC 14, desktop build libraries and the
same source checkout. `CARGO_BUILD_JOBS=4`, `CARGO_PROFILE_DEV_DEBUG=0` and
`CARGO_PROFILE_TEST_DEBUG=0` limit compilation resources without disabling assertions.

- Focused geology test binary: **6 passed**, including frozen numerical and identity vectors.
- `python3 scripts/check_docs.py --base 57618415ad9c2cd301d7f0c0b2a89506956bf721`: **0 failures**.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **134 passed** with Python 3.12.14. Deliberate failure/timeout fixtures in the log are assertions of error handling; the suite exits successfully.
- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --locked`: **1,139 passed, 0 failed, 38 existing opt-in ignored**.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.

Full workspace/build/Clippy logs are retained beside this report. No required
cloud-executable criterion failed or remains unrun. Graphical checks are not
applicable to this portable-only change; #613 owns renderer integration.

An initial focused test intentionally exposed the not-yet-frozen identity sentinel
(4 passed, 1 failed); the computed version-1 identity was then frozen in the golden.
An independently calculated numerical region vector also guards sampling output.
Host package installation lacked root access; required workspace libraries were
installed in the isolated verification container. Neither setup limitation is
reported as a passing build. Human appearance/hardware qualification stays with
#561/#563; no M1/Metal/Windows/macOS claim follows from Linux tests.
