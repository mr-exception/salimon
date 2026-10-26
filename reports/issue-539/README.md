# Physical cargo door and open-bin delivery (#539)

Implemented against main c01ccf62. First PR in the requested stack
539 → 540 → 575 → 582 → 592 → 568. No merge is performed by this task.

The aft cargo aperture now has independently saved opposed sliding leaves and a
fixed manual release grip on either side. E at the port-side grip reverses the
request. Full motion sweeps check the player's body and every fragment, including
the carried object, before closure/retraction; per-tick checks stop obstructed
motion. Partial openings remain conservatively impassable. The manual linkage
works during blackout/in-flight and never changes the external ramp, ship thrust
or pressure state. Presentation uses four bounded ordinary instances (48 cube
triangles, shared geometry), with world positions converted through the ship's
f64 frame. This is simple mechanical geometry, not a new PBR/material system.

The original Blender-authored bin shells now expose their 4 cm walls, 12 cm base
and open top to physical fragments and pickup rays. Walking retains the original
conservative envelopes. Existing single-object carry, stable identities,
ship-local anchors, motion, streaming, mass ledger and disk checkpoint sources
are retained. There is no virtual storage, special sleep flag, mass exclusion or
zero-G attachment. Bin support height is selected by the geometry adapter;
gravity/contact response remains in salimon-physics.

Schema 9 requires finite door position/direction and rebinds transient geometry
on restore without ticking. v1–v8 initialize the previously open aperture; current
missing source rejects atomically. Restoring an obstructing door also rejects.
Door request/end states participate in normal checkpoint triggering.

## Verification

Environment: macOS 26.2, rustc/cargo 1.89.0, repository toolchain and locked crates.
Commands and full outputs are in [validation](validation/).

- `cargo test --workspace --locked`: pass. Runtime: 359 passed, 13 explicitly
  ignored opt-in tests; all other crate tests and doc tests passed. Subsequent
  door/restore changes were covered by the focused checks below.
- `cargo test -p salimon-client --locked cargo -- --nocapture`: pass, 6 passed,
  1 opt-in ignored. Includes exact bin identity/mass/position across save/restore,
  loose/carried sweep blocking, partial-motion restore, blackout operation,
  schema migration and existing real cargo route.
- `cargo test -p salimon-character --locked cargo_door -- --nocapture`: pass,
  including powered and zero-G traversal blocking/reopening from both sides.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `cargo fmt --all` and `git diff --check`: pass.
- `python3 scripts/check_docs.py`: pass, zero findings.
- `python3 -m unittest discover -s scripts -p test_check_docs.py -v`: pass.
- `cargo build -p salimon-client --locked`: pass.
- `python3 scripts/salimon-test run scenarios/evidence/cargo-door.json --binary target/debug/salimon-client --screenshot-command '["native"]' --summary`:
  pass, all 17 steps, three actual GPU surface captures. The scripted route walks
  aft around the Core before approaching the handle and checks open/closed/reopened
  authoritative state. Original state/protocol/context/result files are retained
  in [native](native/), with exact raw states/protocol in
  [the compressed archive](native/raw-state-and-protocol.tar.gz) and a SHA-256 inventory. This is production macOS wgpu capture, not software Vulkan,
  hosted Linux CI, physical keyboard evidence or hardware frame-budget approval.

[Open](native/step-009-cargo-open.png) ·
[Closed](native/step-013-cargo-closed.png) ·
[Reopened](native/step-017-cargo-reopened.png)

Initial regression checks exposed an overbroad interaction target and legacy
payload compatibility errors; the precise grip gate and explicit migration
correct them. Early draft native routes attempted to strafe through the Core or
used the opposite strafe direction; those fixture errors were corrected before
the retained successful run. Passing screenshots are not appearance approval.
Human presentation remains #561, OS/input qualification #557 and hardware
stress/frame budgets #563. No source approval or gameplay economy is changed.

Build SHA-256: `9510315ed167292427ad69f50479776dcef54b2057b8cdc19eaae3473ec8866d`.

Post-commit per-parent documentation impact gate: zero failures.
