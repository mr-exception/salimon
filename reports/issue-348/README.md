# #348 — Fabricator reference M1 verification (2026-10-22)

## Outcome

Partial native evidence, **not full signoff**. Current main `812a877` was rebuilt
from scratch in an isolated Cargo target on Apple M1 iMac21,1 / 16 GiB / macOS
26.2, using Metal at a 1920×1080 physical drawable. No gameplay, assets, recipes,
input rules, shader or baseline approvals changed. Native OS input is blocked:
the helper reports `macOS event posting access is unavailable`. The source issue
must remain open until its remaining acceptance is exercised.

## Completed checks

- `CARGO_TARGET_DIR=artifacts/native-20261022/target python3 scripts/build_game.py --profile release --output artifacts/native-20261022/build`: passed.
- `SALIMON_REFERENCE_SIZE=1 python3 scripts/salimon-test run scenarios/visual/scout-lighting.json --binary artifacts/native-20261022/build/salimon-client --screenshot-command '["native"]' --artifacts artifacts/native-20261022/348/lighting`: all 21 steps passed, eight actual Metal color-surface captures. [Raw results](evidence/lighting/result.json).
- `SALIMON_REFERENCE_SIZE=1 python3 scripts/salimon-test run scenarios/fabricator-contract.json --binary artifacts/native-20261022/build/salimon-client --summary --artifacts artifacts/native-20261022/348/contract`: all 19 steps passed. This checks initially empty machine/inventory, rejected closed-modal enqueue and empty Q-wheel transaction/release through production protocol commands. It does not cover occupied food/medical slots or native pointer input. [Raw results](evidence/contract/result.json).
- `cargo test -p salimon-client --locked fabrication`: 14 passed, zero failed. Includes intake identity/mass, full-bin retention, power cutoff/restart, single-tray accounting, flight/takeoff and saved-midcraft continuation. [Log](evidence/348-fabrication-tests.log).
- `python3 scripts/check_docs.py`: zero failures. [Log](evidence/348-docs.log).

[Machine/source/binary identity](evidence/environment.json), [staged build metadata](evidence/build-info.json).
Timing samples inside fixed-step scenario logs are diagnostic, **not normal-play
frame pacing or a 60 FPS pass**. No concurrent-work timing comparison is claimed.

## Image inspection

The actual powered Fabricator face reads left-to-right: FABRICATOR, LIVE, IDLE,
JOBS 0, LEFT 0 s, E SCREEN / CRAFT and E PORT / INSERT CARGO. The wall housing,
input aperture and empty tray are visible without an obvious wall intersection
at this named reference camera. On blackout the screen is blank, the cabin is
cooler/dimmer and task emission extinguishes. These are observations of this
camera, not circulation/collision or all-DPI acceptance. Reference cameras alter
presentation only; the player remains at the Core in this fixture.

![Powered Fabricator](evidence/lighting/step-018-fabricator-powered.png)
![Blackout Fabricator](evidence/lighting/step-007-fabricator-blackout.png)

## OS-input attempt and remaining gates

`SALIMON_REFERENCE_SIZE=1 python3 reports/issue-348/native_probe.py --binary artifacts/native-20261022/SalimonReview.app/Contents/MacOS/salimon-client --helper artifacts/native-20261022/input --output artifacts/native-20261022/348/native`
failed before the native E/Q sequence because macOS denied event posting.
The helper was compiled with `swiftc reports/issue-312/input.swift -o artifacts/native-20261022/input`.
[Failure](evidence/348-native.log), [pre-input state/captures](evidence/native/).
This probe is an incomplete reproduction aid, not a passed acceptance harness;
its bounded protocol walking preparation is explicitly separate from OS input.
The empty checks object records no successful OS checks.

Still pending: native physical intake/screen/tray walkthrough; actual occupied
food/medical Q pointer and keyboard selection, repeat/focus/click-through checks;
room circulation and collisions; ship-motion/blackout interactions in the full
crafting loop; supported DPI/resolution coverage; matched normal-play CPU/GPU and
presented-frame measurements. Human art/playability approval remains explicit.
No acceptance is transferred or waived in this partial PR; no closing reference
is appropriate yet.

## Documentation impact

Evidence-only addition. Canonical behavior is unchanged; no architecture or
invariant edit is required. Owners remain [runtime](../../client/runtime/architecture.md),
[runtime invariants](../../client/runtime/invariants.md),
[fabrication](../../client/fabrication/architecture.md),
[fabrication invariants](../../client/fabrication/invariants.md) and the
[validation matrix](../../docs/validation.md). Historical reports are unchanged.
