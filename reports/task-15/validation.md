# Task 15 — thruster collision native validation

2026-09-29, Apple Silicon macOS, Rust 1.89.0, native Metal renderer.
The checked-in asset version 9 was validated with
`validate_salimon_phase0_ship.py`: 5,506 triangles, 107 primitives, 13
materials, 471,036-byte GLB. `cargo build --workspace --locked`,
`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
and `cargo test --workspace --locked` passed after the runtime/E2E fixes in
this validation commit.

The native `landed-earth` fixture started inside the ship. The E2E control
interface drove the regular look, movement, door interaction and fixed-step
update paths. The player walked to the aft door, opened it, and crossed to
`Surface` at ship-local X about -13.08 m. The visible ship and thrusters were
inspected in the native window.

- At port Z = +6.983 m, 80 frames of forward input toward the rear engine
  stopped at X = -10.04 m. A further 25 frames with jump and forward held
  did not move the player through the engine.
- At starboard Z = -6.999 m, the same rear approach stopped at X = -10.04 m.
- At Z = -9.309 m, the route outside the starboard engine remained clear:
  160 frames of forward movement reached X = -3.353 m. Returning to X =
  -4.873 m and pushing inward stopped at Z = -8.780 m, matching the outer
  engine body expanded by the 0.24 m player radius.

The four source-derived body/fin proxies are checked against the editable
generator, glTF/GLB metadata, and portable controller bounds by the asset
validator. Rust regression tests cover both engines, approach directions,
clearance, and transformed landed frames. The native run above covers an
Earth-landed orientation. [Starboard side view](starboard-side.jpg) is a
native game capture; the position observations come from authoritative E2E
state, rather than image estimation.

[GitHub issue #24](https://github.com/mr-exception/salimon/issues/24)
