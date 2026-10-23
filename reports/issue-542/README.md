# #542 — Blueprint placement and reserved projects

B02 adds the normal-play blueprint tool: C opens/stows, 1–7 selects,
R rotates, LMB places, E inspects and RMB cancels the aimed project. Escape
closes the tool. Hollow green/red previews expose clearance and proposed
requirements; placed projects remain hollow cyan reservations. There is no
separate worksite or Start action. Mining and Archive interaction are suspended
while the tool owns those actions.

## Authority and scope

`salimon-ship::structure::placement` owns validated catalogs, exact quarter-turn
geometry, half-open 3 m cell coverage, fine snap, unique identities, reservations,
ownership, flight/support admission and final-completion revalidation. An unbuilt
neighbor cannot support another project. Player/cargo/authored fixtures and
protected access lanes reject placement; the runtime repeats current admission
on click. A single room can reserve multiple decks/cells. Runtime uses the
existing ship-local frame and real character/cargo dimensions; no old asset
anchor is moved or rounded. Structural support is an explicit coarse baseline
graph, independent of the precision art layout.

The seven-entry JSON catalog has `approved_economy:false`. Displayed material
and time estimates are **proposals awaiting #537**, documented in Project Q&A.
This delivery consumes no resources, runs no construction timer, installs no
solid geometry and changes no ship mass. Delivery/intake #543, progress #544,
solid installation #545 and full lifecycle/save integration #550/#551 retain
those requirements. Their funding policy must wait for #537 approval.

Schema 8 saves durable projects; legacy schemas 1–7 migrate with an empty set
without ticking. Restore rejects duplicate IDs, inconsistent geometry, altered
catalog estimates, wrong ownership and static/protected-route violations before
publishing state. UI selection/preview are transient. Renderer instances gain
unit-quaternion orientation for thin ship-relative wireframe edges, retaining
the existing instanced draw. Existing instances use identity orientation.

## Verification

Cloud Linux, Rust 1.90.0, Xvfb 1280×800 scale 1, Mesa llvmpipe Vulkan.
No physical-input, hardware GPU/Metal, normal-play FPS or appearance approval.
Exact commands and results are recorded below after final serial capture.

Workspace gates use `PATH=/workspace/tools/cargo/bin:$PATH`,
`RUSTUP_HOME=/workspace/tools/rustup`, `CARGO_HOME=/workspace/tools/cargo`,
`RUSTUP_TOOLCHAIN=1.90.0`. Native routes use `DISPLAY=:101`,
`WGPU_BACKEND=vulkan`, `VK_DRIVER_FILES=/workspace/tools/lvp.json`.

Initial failures remain in validation: the old save-envelope test assumed schema
7 was unsupported; it now checks current+1. Clippy rejected a helper below the
test module; it was moved. The initial 78-step native route passed state checks
but its original images exposed transient feedback hiding requirements; the
action bar now combines those lines while building. Earlier images are retained
as that failed UI attempt, not the corrected presentation.

## Documentation impact

Updated ship/runtime/renderer/character guides, persistence schema/contract,
canonical construction delivery boundary, Project Q&A, validation and maintenance
map. No Universe U01–U34 scientific/runtime behavior changes. Source mapping:
placement domain → ship guides/tests; production controls/save/catalog → runtime
guides/tests and blueprint-placement scenario; orientation → renderer
architecture/invariants and existing camera-relative regression.

## Final results

Checked implementation: `cea243a`; exact input/binary hashes are in
[environment.json](environment.json). Workspace tests **894 passed, 0 failed,
31 opt-in ignored**; Clippy all targets with warnings denied, debug/release
workspace builds, formatting and patch checks pass. Raw logs are in
[validation](validation/). Nine domain placement tests cover all-pairs of safety
conditions, reservations, exact rotations and multi-deck support; six runtime
construction tests cover production actions, atomic restore and unchanged mass.

`python scripts/salimon-test run scenarios/evidence/blueprint-placement.json
--binary target/release/salimon-client --artifacts artifacts/issue-542/final
--summary` passes **79 steps**. The corresponding cockpit-nose route passes
**31 steps**, exercising identity-oriented existing geometry and both cockpit
paths. Runs were serial on the final release binary. Originals and losslessly
compressed raw protocol/state/results are in [native](native/) and
[cockpit](cockpit/). This is production native rendering/input-protocol evidence,
not OS-injected physical input or normal-play timing.

Inspect the [four-cell module preview](native/step-054-multi-cell-module-preview.png)
for corrected concurrent resource/status feedback. The native route places,
inspects, rejects an overlap and cancels; it also declines an occupied multi-deck
room. Domain tests prove one valid multi-deck reservation; the native route does
not claim a completed room or installed hull. Original images retain floor-edge
aliasing; #561 owns subjective presentation/readability review. No baseline or
mask is promoted.
