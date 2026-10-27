# #418 — Real-gravity surface support and funded assists

Delivered portable real-scale support/maneuvers in `salimon_ship::surface`, using
the existing #417 trajectory state and real Core/Propulsion owners. The merged
parent is `3c21c96302daa7cb4525d7b23f4062a7d21c90e0` (current main). This is the first
PR in the requested #418 → #419 → #420 stack; nothing is merged by this work.

## Changes and evidence

[Canonical contract and source/test mapping](../../docs/universe-surface-flight.md).
Surface position, orbital/spin velocity and centripetal acceleration have explicit
SI/epoch ownership. Support is a non-tensile friction-limited physical reaction;
no powered hovering is granted by a landed flag. Cubic maneuver paths preserve
position and velocity at activation/touchdown/free-flight handoff. They require
real mass-dependent impulse, bounded configured forward/reverse watts and an
isolated full-path Core reservation. Command interlocks, insufficient thrust,
insufficient charge and provider failures reject atomically. Plan updates are
partition independent and return remaining caller time. No renderer or current
compressed controller/save change is made.

Five new regressions pass, covering multiple radii/gravity/spins, two masses,
independent Newtonian support and force-integral checks, real takeoff→#417 gravity
flight→landing composition, velocity derivatives, control/funding failures and
partitioning. [Original focused output](surface-tests.log).

## Cloud verification — 2026-10-27

Linux 6.18.55, x86_64, 2 CPU cgroup quota / 8 GiB, Rust/Cargo 1.99.1.
Use `/workspace/.salimon-tools/env.sh` and the installed native library setup.
The first workspace attempt failed because ALSA pkg-config metadata was absent.
Debian's actual `libasound.so.2` (1.2.14) and `libudev.so.1` (257) are installed;
local pkg-config records and conventional linker symlinks under
`/tmp/salimon-native` point to those real libraries. No stub library or disabled
audio backend was used. Add that pkgconfig path to PKG_CONFIG_PATH and its lib
directory to LIBRARY_PATH. Git shell networking failed at the proxy; publication
uses the connected GitHub integration with the tested tree.

Final commands/results:

| Command | Result |
| --- | --- |
| `cargo test -p salimon-ship --locked surface:: -- --nocapture` | 5 passed, 0 failed |
| `cargo test --workspace --locked` | 1,066 passed, 0 failed, 38 existing opt-in ignored |
| `cargo build --workspace --locked` | passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 134 passed |
| `python3 scripts/check_docs.py --base 3c21c96302daa7cb4525d7b23f4062a7d21c90e0` | 0 failures |
| `git diff --check` | passed |

An early focused assertion compared a fractional Core subtraction with exact
f64 equality (difference ~1.8 microjoules at 45 GJ); its tolerance now reflects
storage ULPs (10 microjoules), while the independent force-integral comparison
keeps a 1 J budget. An older workspace attempt was interrupted after the support
admission improvement; the final complete run above uses the final source.
The Python runner's printed timeout JSON is an expected negative-path fixture;
the unittest runner reports 134 passing tests, not a timed-out validation suite.

## Boundaries and retained qualification

Ephemerides/gravity must be smooth at the documented 5 ms planning resolution;
input fixtures establish numerical/model contracts, not full scientific source
coverage. The site must come from a safe terrain footprint. Duration is bounded
to 120 s; an impossible short assist rejects instead of exceeding thrust. No
free tuning, arbitrary terrain safety or arbitrary-time ephemeris is claimed.

As with #417, playable canonical terrain/streaming/new-run adoption remains
#424–#427; offline event orchestration #432–#434; contact implementation #420.
These existing scopes are not unimplemented pieces of the delivered portable
support/assist API. The current compressed native controller is preserved until
world migration. No rendered output changes, so no new graphical evidence gate
applies. Physical controls/OS behavior #557, presentation #561 and actual
M1/Metal/hardware performance #563 remain explicitly unverified separate release
qualification. There is no manual/native acceptance blocker for this routine PR.
