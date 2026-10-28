# #425 — Canonical surface gameplay and persistence

Parent: #675 / `codex/issue-424-canonical-terrain`. Vesper now supplies the same frozen terrain to ship contact/landing, character/EVA, mining, fragments and resources. Five published outcrops have stable semantic IDs and original masses independent of session seed. Existing carry, cargo, Fabricator and power rules remain authoritative.

Save schema 24 binds Canonical mode to the exact frozen body/version/digest, validates depletion and ship/world agreement, and restores callbacks. Historical saves 1–23 remain readable; fixture source values are preserved. See [gameplay contract](../../docs/universe-gameplay.md) and [save contract](../../docs/save-contract.md). All five owning domain guide sets and runtime routing/validation docs are updated here. Native normal startup changes separately in #426.

## Checks and evidence

Linux x86_64, Rust 1.99.1, 2 CPU / 8 GiB `salimon-dev` container; production wgpu/Xvfb/Mesa llvmpipe Vulkan. Commands use `/opt/rust/bin`, `RUSTUP_TOOLCHAIN=/opt/rust`, `--locked -j2`.

- `cargo test --workspace --locked -j2`: PASS, 1,101 tests; 38 existing opt-in ignored. The retained final log also includes the one-test targeted rerun (1,102 total logged passes).
- `cargo clippy --workspace --all-targets --locked -j2 -- -D warnings`, `cargo fmt --all -- --check`: PASS.
- Five canonical session tests exercise actual bounded mining ticks, conserved fragments, cargo/Fabricator once-only intake, cache eviction/restore, corrupt save atomic rejection, funded takeoff/landing partition equivalence and zero-energy rejection. Historical save contracts and affected domain suites also pass; full logs retained.
- `python3 -m unittest discover -s scripts -p test_salimon_test.py`: 22 PASS.
- `python3 scripts/salimon-test run scenarios/evidence/universe-{boarding,mining,flight}.json --binary target/debug/salimon-client --screenshot-command '["native"]'` (one invocation per scenario under Xvfb): respectively **66/66, 24/24, 13/13 PASS**. Original PNGs and compressed raw result JSON are alongside this report. Real actions cover ramp/EVA/reboarding, mining/pickup/drop/depletion restore, takeoff/landing/restore. Rendered captures inspected.
- Documentation and whitespace checks: PASS.

Initial checks exposed a constructor test using exact floating equality (corrected to 1e-9 m numerical tolerance), a mining fixture asking a bounded tick to advance 2 seconds (corrected to 125 actual 16 ms ticks), and a frozen checkpoint expecting the old schema header. The latter permits only the 23→24 header replacement and still compares all remaining bytes exactly. Final reruns above pass; historical fixture bytes are retained unchanged.

No software-GPU timing establishes physical GPU/Metal behavior or 60 FPS. Hardware and subjective release qualification remain separately owned by #557/#561/#563. Canonical geography and published resource/visual/terrain seeds were not altered.
