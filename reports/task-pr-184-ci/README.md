# PR #184 pipeline repair

## Outcome

All three jobs in [run 38518088509](https://github.com/mr-exception/salimon/actions/runs/38518088509) failed at the same Rust 1.99 Clippy diagnostic: `chunks_exact_to_as_chunks` in `client/renderer/src/capture.rs:83`.

The capture pixel loop now uses `as_chunks::<4>().0`, as suggested by Clippy. Rows are already sliced to `width * 4` bytes, so no remainder is possible. Row-padding removal, RGBA/BGRA channel order and PNG encoding remain unchanged. The dynamic row-stride `chunks_exact` loop stays intact. No lint suppression or quality-gate relaxation was added.

## Validation

- Inspected Ubuntu, macOS and Windows failed job logs; confirmed the identical cause on all three platforms.
- `git diff --check`: passed.
- Local Rust formatting, Clippy, tests and native capture: not run because this repair environment has no Rust toolchain installed.
- The branch update triggers the existing Native builds matrix, including `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`, debug/release staging and Linux native checks. Consult [PR checks](https://github.com/mr-exception/salimon/pull/184/checks) for the final run results.

## Scope

This is a source compatibility fix for the stable-toolchain lint. It introduces no new rendering or gameplay behavior and makes no hardware-performance claim.
