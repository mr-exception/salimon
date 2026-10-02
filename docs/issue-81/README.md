# Authored scout spatial contracts — completion evidence

Implementation: `0d7a19417ccc982b74d2e0f11bb74429ed50a7a0` on `main`.

The scout adapter extracts all 22 collision boxes and the seat, door and spawn
markers from the authored glTF nodes. It derives engine/cargo anchors and cargo
room bounds, publishes `client/assets/ship/spatial-contracts.json`, and generates
the character crate's cargo, thruster and anchor constants. Character movement,
body expansion, interaction rules and gravity remain Rust gameplay policy.
Small Blender serialization differences snap to the preservation baseline;
intentional spatial edits outside that tolerance propagate to generated outputs.

## Validation

[Native builds run 37036906205](https://github.com/mr-exception/salimon/actions/runs/37036906205)
completed successfully for the implementation commit:

| Platform | Job | Coverage |
| --- | --- | --- |
| Ubuntu 24.04 | 110937239322 | Python contracts, Rust format/clippy/tests, debug/release builds, deterministic E2E including resource collection and ship EVA, packaged smoke with real X11 input |
| macOS 14 | 110937239552 | Python contracts, Rust format/clippy/tests, debug/release builds |
| Windows 2022 | 110937239696 | Python contracts, Rust format/clippy/tests, debug/release builds |

Follow-up local checks on 2026-10-02:

- Model suite: 32 tests, successful with one explicit Blender integration skip.
- Detailed ship validation: 5,938 triangles, 120 primitives, 13 materials,
  485,332-byte GLB; hierarchy, monitors, collision and interaction checks pass.
- Generated sidecar and all three Rust modules match contracts extracted from
  the checked-in Blender-produced GLB.
- Regression tests reject renamed seat/door/spawn markers and cargo/engine
  proxies, rotated/scaled collision boxes, zero dimensions and unsupported shapes.
- Edited engine/cargo proxy transforms propagate into runtime anchors and
  generated collision/layout outputs.
- Failed staged export validation preserves every published artifact.

The follow-up adds tests and evidence only; runtime assets and Rust behavior are
unchanged. Cargo and Blender are unavailable in this follow-up environment, so
native coverage above is the verified CI run for the implementation. A new
Blender export was not run here. macOS/Windows graphical desktop smoke and the
optional Linux visual-checkpoint step were not run by this CI workflow.

The detailed validator still imports legacy geometry definitions; removing that
dependency belongs to #82, followed by generator retirement in #83.
