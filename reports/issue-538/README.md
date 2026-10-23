# Starter Scout cargo and habitation envelope — #538

The linked cabin library now contains a separate aft cargo compartment, a full-height bulkhead with a 3 m internal opening, two open storage shells, stowed fold-away bunk, galley and hygiene fixture. The original outer hull, landing gear, cockpit/Core/Fabricator/Archive anchors and 2.8 m boarding ramp retain their scale. No Jump Drive is installed.

The authoring recipe, saved library, exports, generated collision bounds, character movement/sight/placement and maintained guides change together. Existing exit routes now turn through the internal doorway. The fragment-transfer route returns to the cargo side before targeting its stored fragment, rather than reaching through a wall. No physics, mass, sleep, damping or elapsed-time rule was changed.

## Delivered and deferred boundaries

This is the S01 authored envelope. The internal opening is currently open: **an operable closable internal door is not implemented here**. Its independent interaction/state, powered/unpowered behavior, occupied-sweep safety, physical bin intake/contacts, retrieval and zero-G containment remain [#539](https://github.com/mr-exception/salimon/issues/539). Bins have conservative exterior placement bounds until that work; this report does not claim usable internal storage. Habitation prompts, animations, privacy and applicable survival behavior remain [#540](https://github.com/mr-exception/salimon/issues/540). These are existing downstream implementation issues, not passed criteria or duplicate follow-ups.

Source presentation review remains [#561](https://github.com/mr-exception/salimon/issues/561): inspect the original cargo/habitation views for proportions, full-height separation, station approaches, warm/cool hierarchy, opaque frames, visible fixture attachment, glass intersections and used-but-maintained appearance. Physical input/platform experiments remain #557. The deliberate increased asset limits and actual hardware performance require #563 review; software Vulkan does not certify M1/Metal or 60 FPS.

## Measured asset and spatial contracts

- Export: **6,767 triangles / 134 primitives / 13 materials / 581,396 bytes / 173 nodes**.
- Deliberate limits: **6,900 triangles / 140 primitives / 600 KiB GLB / 180 nodes**; existing one opaque plus one glass draw is retained.
- Bulkhead X=-4.65 m; two bins flank the centered carrying route. Bunk is ahead of the Archive approach; galley/hygiene use the cabin walls.
- Eight new authored proxies (seven fixture bounds plus a separate header) feed the generated layout. Header clearance applies to jumping, zero-G movement, floor placement and sight.
- Existing ceiling fixtures are split around the structural wall. Continuous wall/bin shells and omitted hidden roof caps avoid competing contact surfaces. No overlap exemption or numerical tolerance was added.
- Future aft/upper expansion markers remain authored interfaces; they do not round old precision anchors or install a modular hull.

## Verification

Working directory: repository root. Rust 1.90.0; Python 3.12.14 tooling; Blender **4.5.3 LTS** via the official `bpy==4.5.3` wheel on CPython 3.11.17. The machine's preinstalled Blender 4.3.2 was not used for exports. The exact [wheel-backed launcher](validation/blender453-launcher.py) is retained. See [environment and source/binary hashes](environment.json).

The cloud lacked Rust, Xvfb and Mesa ICDs initially. Dependencies were installed/extracted under `/workspace/tools`; the regular apt install failed on filesystem permissions. Rust and package downloads used the configured proxy; no blocked destination was bypassed. Full Blender download access was denied; the supported Blender version was obtained from PyPI instead.

For these commands, `PATH=/workspace/tools/cargo/bin:$PATH`, `RUSTUP_HOME=/workspace/tools/rustup`, `CARGO_HOME=/workspace/tools/cargo`, `RUSTUP_TOOLCHAIN=1.90.0`; authoring Python is `/workspace/tools/blender-python/bin/python`, and `BLENDER=/workspace/tools/bin/blender453`.

| Command | Actual result |
| --- | --- |
| `python models/assets/ships/salimon-scout/export.py --blender /workspace/tools/bin/blender453` | PASS, measured asset above |
| `blender453 --background --python models/assets/ships/salimon-scout/verify_source.py` | PASS, 173 source/export objects and 6,767 triangles preserved |
| `blender453 --background --python models/assets/ships/salimon-scout/verify_modular.py` | PASS, linked-library edit propagation retained |
| `BLENDER=/workspace/tools/bin/blender453 python -m unittest discover -s models/tests -v` | PASS, **99 tests**, no skips after installing the pinned glTF validator |
| `cargo build --release --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo test --release --workspace --locked` | PASS, **876 passed / 31 existing opt-in ignored**, after rebasing onto main |
| `cargo build --workspace --locked` | PASS, including post-rebase build |
| `cargo fmt --all -- --check` | PASS |
| `SALIMON_SCOUT_FEEDBACK_DIR=... VK_DRIVER_FILES=/workspace/tools/lvp.json WGPU_BACKEND=vulkan cargo test --release -p salimon-renderer --locked scout_feedback_gpu_evidence -- --ignored --nocapture` | PASS, production ship renderer; frozen pairs equal in every case, including four new starter views |
| `DISPLAY=:101 WGPU_BACKEND=vulkan VK_DRIVER_FILES=/workspace/tools/lvp.json python scripts/salimon-test run scenarios/evidence/starter-layout.json --binary target/release/salimon-client --artifacts artifacts/issue-538/starter-final --summary` | PASS, **41 steps**, including side-wall rejection, ramp exit/re-entry and Archive targeting |
| Same runner with `scenarios/evidence/cockpit-nose.json` | PASS, **31 steps**, both cockpit routes |
| Same runner with `scenarios/fragment-transfer.json --screenshot-command '[]'` | PASS, **285 steps**, including post-rebase rerun; structured state, no screenshot claim |

### Original graphical evidence

Native application, Xvfb 1280×800, scale 1, Linux Mesa llvmpipe Vulkan. Input is the production protocol with 16 ms fixed steps, not OS-injected or physical input. Original screenshots and compressed protocol/state records are in [native-route](native-route/); the complete result, stdout and protocol JSONL are losslessly gzip-compressed to avoid duplicating large snapshots. Original production-renderer images, adapter data and raw CSV are in [gpu](gpu/README.md). That offscreen run overlapped compilation/native checks and is **image evidence only**, not matched performance evidence or normal-play FPS.

- [Cargo/bulkhead](gpu/starter-cargo-forward.png), [cargo/ramp](gpu/starter-cargo-aft.png)
- [Bunk/hygiene](gpu/starter-bunk-hygiene.png), [galley](gpu/starter-galley.png)
- [Archive approach and live prompt](native-route/step-041-archive-access.png)
- [Ramp exterior clearance](native-route/step-024-ramp-clearance.png)

### Failed attempts retained

Initial source export attempts rejected coplanar ceiling, bulkhead/header, bin-shell and fixture contact surfaces. The final recipe removes the competing faces or gives deliberate fitted joints; it does not exempt those failures. The initial build also ran before generated starter constants existed and failed; only the post-export build passes.

The first model run failed three stale inventory assertions (29 old proxies/no cargo nodes). These now assert 37 proxies and the new starter nodes. The second model run passed 99 tests with two optional glTF integrations skipped; after `npm_config_cache=/tmp/salimon-npm-cache npm ci --prefix models/tools/gltf`, the final run passes all 99 without skips.

Earlier workspace runs failed five empty-cabin traversal fixtures and then legacy exit/return routes. Tests now use clear window approaches and the actual internal opening. The original 41-step native route initially failed Archive targeting while its binary still contained the earlier bunk position; the final clean asset/build and route pass. Fragment-transfer failures first stopped at the bulkhead and later correctly rejected targeting a fragment through that bulkhead. The route now walks into the cargo compartment. Raw logs of the failed iterations are retained under [validation](validation/); none are relabeled as passes.

## Documentation impact

Updated the canonical technical architecture, approved construction delivery boundary, asset/source guides and character architecture/invariants. Generated layout and export ownership remain with the existing exporter. No Universe U01–U34 runtime/scientific behavior changes; its maintenance gate does not add a separate implementation scope here. Existing source validation and model schema tests check named paths/interfaces; `git diff --check` checks patch integrity.

## Final delivery and main synchronization

[PR #590](https://github.com/mr-exception/salimon/pull/590) is the first branch in
the requested #538 → #542 → #559 → #566 review stack. No merge was performed.
The final implementation was rebased onto main `2ecb271`; the checked revision
is recorded in `environment.json`. After rebasing, workspace tests pass **876 / 0
failed / 31 opt-in ignored**, Clippy, debug/release builds, formatting and **118
script tests** pass. Native routes were rerun serially on that release binary:
[starter, 41 steps](rebased-starter/), [fragment transfer/retrieval, 285 steps](rebased-fragment/),
[cockpit, 31 steps](rebased-cockpit/). These directories retain originals and
losslessly compressed raw protocol/results. They supersede the earlier native
route as final build provenance; earlier records remain historical evidence.
The GLB and source-library hashes are unchanged through main synchronization.
The offscreen GPU originals were generated before synchronization, with their
original environment retained in `validation/pre-main-sync-environment.json`.

The final workspace run also exposed the renderer's duplicated old 544 KiB /
6,500 triangle limits. Its test now decodes the same canonical asset manifest
using an already pinned test dependency. Initial missing-dependency/type-check
failures were corrected before the successful full rerun; no gate is waived.
