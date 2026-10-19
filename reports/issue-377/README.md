# Issue #377 — Q-wheel implementation, survival integration blocked

2026-10-19. Reviewable portable implementation on top of #346/#347. This PR is
**draft** and does not close #377: its explicit blocker #368 is still open.
The original issue's survival acceptance is not waived by the passing tests.

## Implemented

- Hold Q opens five radial slots from the existing 3-food/2-medical inventory;
  release cancels, click consumes that exact occupied slot once and closes.
  Typed `ConsumableSlot` replaces the superseded first-occupied consumption API.
  No duplicate inventory or equipment capacity is introduced.
- Empty segments are disabled. Names, original ration/medical artwork, hover and
  the current `NO ACTIVE EFFECT` description are shown. Tab/arrows and Enter
  provide keyboard selection/consumption; equipment remains on keys 1–5.
- Opening releases held gameplay inputs and native pointer capture. World/ship
  ticking continues. World input/look/mining are isolated until closing. Focus,
  lifecycle, cursor exit, repeated Q, pre-held mouse and click-through are guarded.
  Consumption cannot replay as a mining press. Escape cancels and releases the
  cursor using current shell policy.
- Renderer owns cached bitmap and shared physical-pixel hit geometry; portable
  commands own selection/consumption. Native and protocol input use these routes.
- The mined-resource Fabricator scenario now consumes its collected ration and
  medkit through Q; the declarative route also verifies empty-wheel cancellation.

## Blocked and not claimed

[#368](https://github.com/mr-exception/salimon/issues/368) is unfinished. Hunger,
health, fatigue and the survival/death lifecycle are not implemented on this
stack's base. Consequently there is no healing/status application, meaningful
full-meter/fatigue-zero validation, or death-state cancellation validation here.
These must be wired and tested against #368 and the survival lifecycle before
marking this PR ready or closing #377. No placeholder vitals system is added.
The future Escape pause menu also needs to use the same cancellation/reset hook;
current Escape releases the cursor and does not freeze simulation.

Native OS delivery, rendered visual signoff and M1 performance are unrun. No
live display, Xvfb or Vulkan ICD is available; native visual verification remains
#348. CPU image/layout tests are not native screenshots or GPU evidence.

## Validation

Linux x86_64, Rust/Cargo 1.99.1 stable.

- `cargo test --workspace --locked`: 576 passed, 0 failed, 22 existing opt-in ignored.
- `cargo test -p salimon-client --locked wheel`: 5 passed (native dispatch/focus,
  quick tap, repeats, exact item removal, pre-held mouse, keyboard, input gating,
  continuing flight/ticks, crafting/tour and empty slots).
- `cargo test -p salimon-renderer --locked consumable_wheel`: 2 passed (all five
  sectors, fit/hit parity at multiple drawable sizes, invalid input, cache/art).
- `cargo test -p salimon-client --locked fabrication_scenarios`: 2 passed after
  final declarative Q route and explicit no-click-through assertion.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 93 passed.

The full workspace run preceded the final UI effect-label wording and scenario
assertion/data refinements; focused renderer/input/scenario checks and final
build/Clippy cover those refinements. The 22 ignored tests retain their existing
opt-in native/benchmark/offline-asset prerequisites.

## Persistence stack refresh (2026-10-19)

Rebased the wheel onto the completed #363–#365 persistence stack (#388 → #389 →
#390), retaining both persistence and wheel architecture/invariant/maintenance
sections. PR #387 now targets the #365 branch so its diff contains the wheel
layer and integration changes. #368 remains an explicit unresolved blocker and
the PR remains draft.

Adapted the full #365 save/load/collect/consume scenario to the production
exact-slot Q route instead of the superseded category API. Saving with Q held,
an empty slot hovered and the primary button down preserves authoritative
inventory/session bytes; restore clears wheel/modal/hover/held mouse without
consuming or replaying anything. The resumed scenario consumes one food and one
medical item, with a real closed-wheel mouse release routed through Mining(false),
as native/protocol dispatch does. An initial test bypassed that release route;
the corrected production-route fixture and final full suite pass.

Final refreshed-tree validation: `cargo test --workspace --locked` passed
**584 tests, 0 failures; 22 existing opt-in ignored**. `cargo build --workspace
--locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
`cargo fmt --all -- --check`, `git diff --check` passed. `python -m unittest
 discover -s scripts -p 'test_*.py'` passed all 93 tests. Graphical/OS-input/M1
checks remain unrun under the original limits above; this refresh does not claim
survival effects, pause/death integration or native visual signoff.

## Durable checkpoint stack refresh (2026-10-19)

Updated the existing draft on top of #392 (#366) after #388–#390 were merged.
The update preserves the existing wheel implementation and all persistence
changes. Resolved appended runtime architecture/invariant sections by retaining
both contracts; the app/input changes merged without source conflicts. PR #387
retains GitHub's managed `main` base: GitHub rejected changing the base because
this PR is part of a stack. Its history includes #392 as a merge parent; merge
#392 first, then review the remaining wheel diff. No force update was used.

Added a real native filesystem checkpoint regression with Q held, an empty slot
hovered and primary mouse held. Opening/hovering the wheel does not change the
discrete save event signature or authoritative bytes. Reopening the durable save
clears open/held/hover/primary state and cannot replay consumption. Existing
exact-slot populated-inventory and continuation scenarios remain passing.

Refreshed combined tree: `cargo test --workspace --locked`: **592 passed, 0
failed; 22 existing opt-in ignored**. Workspace build, all-target Clippy with
warnings denied, formatting, diff checks and 93 Python script tests passed.
The underlying #366 layer separately passed Linux Vulkan launch/OS input and
three real clean-exit/relaunch/corrupt-fallback launches; see
[its report](../issue-366/README.md). This refresh does not claim Q-wheel native
visual/M1 signoff. #368 and the survival/effect/death integrations remain pending,
and #387 remains draft. No changes were merged automatically.
