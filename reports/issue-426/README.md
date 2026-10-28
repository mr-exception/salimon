# #426 — New journeys outside the landed Vesper scout

Normal native startup now constructs the frozen Vesper world and places the traveler at the published tangent offset [14,0], standing outside the authored hull and facing the ship. The frozen minimum Earth distance is 37.55002420841507 light-years. Conservative capsule/hull overlap is checked before publication; a failure is an error, never silent planet reselection.

Terminal restarts increment the serial and construct this canonical source, granting existing two rations, medkit and Fabricator reserves once. Restore never grants supplies. Historical active checkpoints remain resumable until #427; fresh restart from those nonflat worlds selects Vesper. Explicit flat fixtures retain their policy. Suit oxygen depletion remains deferred.

Sources: `client/runtime/src/session/canonical.rs`, `session/lifecycle.rs`, `app.rs`. [Gameplay contract](../../docs/universe-gameplay.md), runtime owning guides, root README and starting-system/save/routing/validation guides updated here. Parent: #676.

## Validation

Linux x86_64, Rust 1.99.1, established 2 CPU / 8 GiB container. `/opt/rust/bin`, `RUSTUP_TOOLCHAIN=/opt/rust`; cargo commands use `--locked -j2`.

- `cargo test -p salimon-client --locked canonical_ -j2`: **11 PASS**, including three new journey tests: deterministic identity/clearance/facing, exact save roundtrip/no repeated grants, historical restart and ordinary ramp entry.
- `cargo test -p salimon-client --locked session::lifecycle -j2`: **6 PASS**.
- Workspace all-target Clippy with `-D warnings`, client build, formatting: PASS.
- Python runner unit suite: **22 PASS**. Documentation and whitespace checks: PASS.
- `xvfb-run -a -s '-screen 0 1280x800x24' python3 scripts/salimon-test run scenarios/evidence/universe-journey.json --binary target/debug/salimon-client --screenshot-command '["native"]'`: **25/25 PASS**, production wgpu software-Vulkan/Mesa llvmpipe, explicit lvp ICD and two threads. Original captures show published external start, rear approach and actual cabin entry; inspected. Raw final observations retained compressed.

Two earlier routes failed: the first assumed the wrong ship-local axis for the world tangent offset, the second entered ramp swing clearance before opening. The final route uses the actual pose and opens outside the ramp. Portable and native routes pass without relaxing spawn, collision, geography or gate rules. Both failed raw results are retained.

Physical GPU/Metal/60 FPS and subjective presentation remain unverified and separately owned by #557/#561/#563. No software-renderer timing establishes hardware claims.

## CI follow-up

[Initial Source build 40634664126](https://github.com/mr-exception/salimon/actions/runs/40634664126) compiled successfully but failed one old cockpit-title assertion: it expected `Compressed Solar System` after normal startup switched to the neutral `Salimon` title. Both expected strings now match the intended title; the actual aimed-cockpit/empty-context behavior stays tested. `cargo test -p salimon-client --locked cockpit_prompt_is_shown_only -j2`: 1 PASS, retained in `title-test.log`. Initial failed CI remains recorded; the new head reruns CI. macOS/Windows publication golden jobs passed on the initial head; these are portable publication checks, not native gameplay qualification.
