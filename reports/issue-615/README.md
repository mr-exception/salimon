# Issue #615 — deterministic regional climate and weather

Base: `ab5ebb80394b8e74b72e3c156120cc51a65ca9b1` (main). Design PR #611,
geology #612/PR #686 and atmosphere #419/PR #671 are merged in that ancestry.

World now owns validated independent climate publications bound to exact geology,
canonical strict codecs/identities, direct TT epoch sampling, continuous spherical
fronts, tangent wind and bounded pressure/temperature/cloud/condensable/visibility
metadata. Vacuum excludes all airborne effects. No terrain, resource, save schema,
physics, renderer, survival or player-knowledge definition is changed.

The [climate contract](../../docs/planetary-climate.md), owning world guides,
maintenance/architecture/validation routing, root guide and approved-design status
map the implementation and retained scopes. The production cross-domain test in
`client/ship/tests/regional_weather.rs` feeds evaluator wind into real U18 gas and
drag, preserving explicit density/rotation/frame ownership.

## Verification

Environment: Debian 13 Linux x86_64, Rust/Cargo 1.99.1, Python 3.12.14.
Commands use `. /workspace/.salimon-tools/env.sh` to select the existing toolchain
and extracted development libraries. No credentials or generated assets changed.

- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `python3 scripts/check_docs.py --base ab5ebb80`: zero failures.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 134 passed.
- `cargo test --workspace --locked`: **1,152 passed, 0 failed, 39 existing opt-in ignored**.
  Includes seven focused climate tests and the production U18 composition test.

Initial setup lacked ALSA development metadata and then its matching shared
library; extracted `libasound2-dev`, `libasound2t64` and `libudev-dev` repaired the
host sysroot. The first workspace attempts failed at setup/linking; they are not
reported as passing. A test digit-grouping lint was fixed. The final commands
above use the repaired environment. Tooling logs include intentional subprocess
failure/timeout fixtures; the 134-test suite itself exits successfully.

## Limits and retained ownership

The climate model is a bounded stylized authoring contract, not a measured weather
forecast/phase diagram. No real-body atmosphere/life claim or new source ingestion
is supplied. Callers must publish explicit reviewed input definitions. Vesper's
existing terrain/resources/publication remain byte-identical.

Weather rendering remains #616; acoustic presentation #627; offline trajectory and
clock host integration remain #432–#434. This issue supplies stateless queries
usable by those hosts without claiming their routes already exist. No renderer or
runtime presentation code changes, so production graphical capture is not applicable
for this metadata-only feature. Human presentation/hardware qualification #561/#563
and physical platform checks #557 remain unverified; no M1/Metal claim follows.

Final raw logs: [workspace tests](workspace-tests.log), [build](build.log),
[Clippy](clippy.log), [tooling](tooling.log), [documentation](docs.log).

## CI documentation correction

The first Ubuntu CI run rejected the added ship composition test because the
ship owning guide had not been updated. The pre-commit local comparison inspected
`base...HEAD` and missed the then-uncommitted test. This PR now updates
`client/ship/README.ai.md`; validation is repeated against the committed diff.
No source or test behavior changed. Windows/macOS fiction-publication goldens
passed on the initial revision; that is scoped publication evidence, not physical
platform or graphics qualification.
