# Issue 195 — portable flight and Core regression

Adds four deterministic E2E routes and matching evidence variants for inertial
flight, dead-Core mechanical refueling/restart, crash settling/recovery with
reservation and inertial takeoff, and physically onboard 500-fragment cargo.
The named `realistic-flight` suite also runs the existing moving-EVA route.
New setup fixtures only establish initial conditions; actions use production
commands. Read-only inspection now exposes power/conversion/navigation state.

Headless tests replay every new route and verify each assertion. Additional
runtime tests protect cargo mass/identity during ship movement, power changes
and Core consumption, and carried-fragment zero-G drift/power restoration.
Existing domain tests retain detailed numeric bounds and frame-partition checks.

Validation (Linux x86_64, Rust 1.99.0):

- Locked workspace tests passed with one codegen unit (`CARGO_TARGET_DIR=/tmp/salimon-target`, `CARGO_PROFILE_DEV_CODEGEN_UNITS=1`, `CARGO_PROFILE_TEST_CODEGEN_UNITS=1`, `-j 2`).
- `cargo test -p salimon-client flight_regression --locked`: six new checks passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 55 passed.
- Formatting, diff checks and workspace Clippy passed before handoff (same build environment).
- Native software-rendered route results and any remaining environment limits are
  recorded below. Apple M1 hardware signoff remains in separate native tasks.

## CI follow-up (2026-10-14)

[Native builds run](https://github.com/mr-exception/salimon/actions/runs/38614292870)
passed Linux, macOS and Windows at `3fdc74f902e78786253d39e866d7e69e0f243eb4`. Linux included Mesa software-Vulkan
baseline/evidence routes and packaged-input smoke. This evidence predates the
small subnormal-velocity normalization regression added to #193; its focused
ship check passes separately. It does not establish physical Apple M1 signoff.
