# Issue #346 — consumable HUD and retired shortcuts

Implemented on 2026-10-19. Five dedicated one-item consumable slots (three food,
two medical) map from the existing inventory, independently of the equipment
row. Original faceted ration/medical package icons match the authored tray's
amber stripes/red cross. Empty category markers remain visible. The combined
HUD's cache, DPI fitting and message clearance include both rows.

Removed native and automation M/N consumption dispatch and its unused runtime
latches; precision-tour N retains its existing camera function. Q selection is
owned by #377. No survival effect, asset regeneration or new inventory is added.

## Validation

Environment: Linux x86_64, Rust/Cargo 1.99.1 stable.

- `cargo fmt --all -- --check`: passed.
- `cargo test --workspace --locked`: passed, 566 passed, 0 failed, 22 opt-in ignored.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed (see final branch validation).
- Renderer tests cover separate row contents/cache, unchanged equipment pixels,
  empty slots, scaling and small drawable fitting; native mapping regression
  establishes that M/N no longer consume or change gameplay view.
- Python runner checks initially failed short timeout assertions under concurrent
  first-time compilation. Final idle rerun of
  `python -m unittest discover -s scripts -p 'test_*.py'`: passed, 93 tests.
- Native visual/GPU/OS-input smoke: not run; this environment has no display,
  Xvfb or Vulkan ICD. Reference visual signoff remains #348. CPU bitmap tests
  do not establish native appearance/performance.
