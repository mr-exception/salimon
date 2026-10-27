# #419 — Planet-specific atmospheric drag

Delivered on exact parent `5c7fbb604247e7cf666147c7abc9c37424323387`
([#418 PR #670](https://github.com/mr-exception/salimon/pull/670)). This child PR
targets `issue-418-real-gravity-assists`; #420 will target this branch. Review and
merge #418 → #419 → #420. Nothing is merged by this work.

[Canonical model/source/test/integration contract](../../docs/universe-atmosphere.md).
Planet-specific density/scale height/ceiling, rigid co-rotation, explicit wind,
projected area and current mass feed a stable quadratic-drag operator. Symmetric
composition uses the production gravity/funded-thrust integrator. Online and
offline use the same fixed-step operation; closed-game engine shutdown stays a
caller obligation. Landing/takeoff compensation now funds actual drag force and
checks wind support, instead of following the old vacuum budget through air.
Provider/numerical errors publish no partial trajectory or selector state.

## Cloud verification — 2026-10-27

Same Linux 6.18.55 / x86_64 / 2 CPU quota / 8 GiB / Rust 1.99.1 environment and
actual ALSA/udev linker setup recorded in [#418](../issue-418/README.md).
No new dependency, source data, renderer, UI, save schema or compressed-controller
change. All validation below ran against this child's source, with no required
cloud criterion deferred:

| Command | Result |
| --- | --- |
| `cargo test -p salimon-ship --locked atmosphere:: -- --nocapture` | 7 passed, 0 failed; [original output](atmosphere-tests.log) |
| `cargo test --workspace --locked` | 1,073 passed, 0 failed, 38 existing opt-in ignored |
| `cargo build --workspace --locked` | passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 134 passed |
| `python3 scripts/check_docs.py --base 5c7fbb604247e7cf666147c7abc9c37424323387` | 0 failures |
| `git diff --check` | passed |

The focused suite checks Earth/Mars-parameter density laws, spin/wind and explicit
epochs, independent exact constant-gas drag, stiff-step no reversal, mass scaling,
bit-exact vacuum compatibility, terminal-speed agreement, ~4x refinement ratio
against a 1 ms reference, orbital specific-energy loss, exact online/offline
replay and rollback. A production #418 assist consumes >1 MJ additional energy
with its declared dense/large-area fixture, while unaffordable wind rejects with
no Core debit. These are numerical fixture/model checks, not source accuracy or
new approved gameplay tuning. The tooling suite's printed timeout JSON is its
expected failure-path fixture; the unittest run itself passes.

## Model and adoption limits

The exponential continuum/Cd-area model has explicit altitude and smooth-field
validity; it is not Mach-dependent aerodynamics, rarefied gas, weather, heating
or damage. Model inputs must be supplied explicitly; unknown data returns an
error, not guessed vacuum. The caller must refine dt for its error budget even
though the stable fixed-gas operator cannot overshoot relative velocity. A drag
velocity overflow rejects rather than silently clamping physics.

As with the parent, canonical playable-world/terrain/new-run adoption remains
#424–#427; closed-game event/save orchestration remains #432–#434. #420 owns
continuous contact before an endpoint force query could enter a solid body.
No rendered output changes; no new graphical capture requirement applies.
Physical devices/OS #557, subjective presentation #561 and actual M1/Metal/GPU
budgets #563 remain explicitly unverified separate release qualification, not
manual acceptance gates for this cloud implementation.
