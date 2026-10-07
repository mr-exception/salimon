# Issue #125 — Diagnostics internal source split

## Outcome

Separated the dependency-free diagnostics implementation into private modules
without changing public DTOs/methods, metric semantics, refresh behavior or RGBA
output. Implementation based on `b1743cbb3a1984e206ad7a0d6b9a5df7bc157ceb`.

- `lib.rs` retains public contracts, visibility/density, overlay storage/revision
  and immediate rebuild orchestration.
- `aggregation.rs` owns bounded frame history, latest sample, refresh accumulator,
  validated owned domain snapshots, warning count, averages and nearest-rank p95.
- `format.rs` owns text rows, GPU availability labels and metric/memory units.
- `raster.rs` owns panel layout/colors, the embedded font and CPU RGBA drawing.
- Existing ten tests are preserved beside their relevant owners. Three new tests
  cover bounded eviction/zero intervals, nearest-rank p95 and exact refresh/reset
  behavior. Test fixtures remain private and test-only.
- Updated crate architecture/maintenance guides and root architecture/source map.

Runtime still owns when to observe/toggle/reset; renderer still owns GPU work.
No new dependencies or deliberate behavior fixes were introduced.

## Validation

Environment: Linux x86_64, kernel 6.18.44, Rust/rustfmt/Clippy 1.99.0,
Python 3.12.14. Commands run from the repository root (Cargo tools added to PATH).

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test -p salimon-diagnostics --locked` | Passed: 13 tests |
| `cargo test --workspace --locked` | Passed: 263 tests; one existing GPU test ignored by default |
| `cargo build --workspace --locked` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests |
| `git diff --check` | Passed |
| Original-vs-refactored comparison below | Passed: 1,000 steps, identical text and RGBA bytes |

### Reproduce output equivalence

[compare.rs](compare.rs) is a standalone validation harness. It feeds both
implementations identical zero/nonzero frame intervals, CPU/GPU/memory states,
valid/invalid domain observations, toggles, resets, warning events and density
changes, asserting visibility, text, image dimensions, revisions and every RGBA
byte after each of 1,000 steps. This is CPU output coverage, not live GPU evidence.

```sh
mkdir -p artifacts/issue-125
git show b1743cbb3a1984e206ad7a0d6b9a5df7bc157ceb:client/diagnostics/src/lib.rs > artifacts/issue-125/original.rs
rustc --edition 2024 --crate-type rlib --crate-name diagnostics_before artifacts/issue-125/original.rs -o artifacts/issue-125/libdiagnostics_before.rlib
rustc --edition 2024 --crate-type rlib --crate-name diagnostics_after client/diagnostics/src/lib.rs -o artifacts/issue-125/libdiagnostics_after.rlib
rustc --edition 2024 reports/issue-125/compare.rs --extern diagnostics_before=artifacts/issue-125/libdiagnostics_before.rlib --extern diagnostics_after=artifacts/issue-125/libdiagnostics_after.rlib -o artifacts/issue-125/compare
artifacts/issue-125/compare
```

## Limitations

Native graphical smoke/E2E, debug/release staging, and model-tool tests were not
run locally. This environment has no graphical display, Xvfb or Vulkan runtime;
attempted desktop prerequisites installation was blocked by package-manager
permissions/locking. Model assets/tooling were unchanged. The existing PR
pipeline covers platform builds, model tests and Linux graphical checks.
No screenshots are supplied for this internal refactor; the byte-for-byte panel
comparison establishes preserved CPU pixels, without claiming GPU composition,
macOS/Windows fidelity or reference-machine performance. The issue remains open
pending review and merge.
