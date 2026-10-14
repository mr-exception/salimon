# Issue 193 — vector cockpit navigation

The existing wide console now shows persistent velocity magnitude and ship-local
unit direction, heading/travel angle, signed throttle and engine acceleration,
cached total mass, nearby total relative speed and full-reverse stop estimates.
The separate Core monitor retains its atlas region and behavior.

Ship owns telemetry and feasibility; runtime maps values and renderer caches
rounded visible readings. Estimates assume fixed heading/mass and clear travel,
include reverse plus idle energy, exclude future charging and reserved energy,
and explicitly reject misalignment, unavailable propulsion and insufficient
stored energy. There is no automatic braking or steering.

Validation (Linux x86_64, Rust 1.99.0):

- `cargo fmt --all`: passed.
- `git diff --check`: passed.
- `CARGO_TARGET_DIR=/tmp/salimon-target CARGO_PROFILE_DEV_CODEGEN_UNITS=1 CARGO_PROFILE_TEST_CODEGEN_UNITS=1 cargo test -p salimon-ship --locked -j 1`: 60 passed.
- Full workspace `cargo test --workspace --locked -j 2` with the same environment: passed (including renderer cache tests and runtime mapping).
- Single codegen unit works around this container's archive builder producing empty object files with defaults.
- Graphical cockpit evidence is not run at this point; shader/asset geometry is unchanged.
- Apple M1 manual visual/performance signoff is outside this cloud issue.

## CI follow-up (2026-10-14)

[Native builds run](https://github.com/mr-exception/salimon/actions/runs/38613645207)
passed Linux, macOS and Windows at `ef9d47f0cf6d4f3d97c010d1c1210d7471e76bde`. Linux included Mesa software-Vulkan
baseline/evidence routes and packaged-input smoke. This evidence predates the
small subnormal-velocity normalization regression added to #193; its focused
ship check passes separately. It does not establish physical Apple M1 signoff.
