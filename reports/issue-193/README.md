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
