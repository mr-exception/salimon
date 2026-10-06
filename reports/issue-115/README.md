# Issue #115 — shared Rust math evaluation

Date: 2026-10-06. Base revision: `9b0a245`.

## Outcome

Adopted a deliberately small, dependency-free `salimon-math` crate for six
compatible `f64` vector primitives. The
[architecture decision and inventory](../../docs/technical-architecture.md#shared-math-decision-115)
compares character, ship, runtime fragment/carrying/composition, world and
renderer helpers, including precision and normalization differences.

Migrated character and ship component arithmetic, fragment simulation vector
arithmetic/length, and carrying's cross product. Removed 18 local helper
implementations; no gameplay DTO, frame, normalization policy or coordinate
conversion changed. `WorldPosition` and resource pose validation stay world-owned;
quaternion and frame semantics stay domain-owned. No external dependency was
added; Cargo.lock changes only workspace package/dependency entries and Cargo's
ordering of the runtime dependency list.

Updated coding conventions, architecture/dependency/maintenance maps and local
crate guides. Six new regression tests cover right-handed cross products,
large-anchor subtraction before narrowing, IEEE arithmetic edges, character and
fragment normalization boundaries/fallbacks, and ship quaternion composition,
unit normalization and identity fallback. Existing shortest-arc interpolation,
movement/EVA, assists, contacts, resource validation and renderer precision tests
also pass.

## Validation

Environment: Ubuntu 24.04.3 LTS, x86_64; rustc 1.99.0
(`b940084d7 2026-09-28`), cargo 1.99.0 (`5f94df478 2026-08-27`).
A Rust toolchain was installed because this workspace initially had none.

| Command/check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed on final source |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 251 passed, 0 failed, 1 existing GPU-only test ignored |
| `cargo build --workspace --locked` | Passed |
| `git diff --check` | Passed |
| Relative file links in changed Markdown guides, resolved against each parent directory with Python pathlib | Passed |

Used `/root/.cargo/bin/cargo` for these commands. Initial `cargo test --workspace`
updated the lockfile for the new internal package; subsequent gates used
`--locked`. The only edits after the locked lint/test gates were import placement
and whitespace; final formatting/build checks covered those edits.

## Limits and follow-up

Native graphical scenarios, screenshots, model tooling and macOS/Windows runs
were not run locally: this change is CPU arithmetic extraction with no shaders,
assets or presentation changes. The local environment has no Xvfb/software
Vulkan setup. The existing GPU-only ignored test remains unrun; CPU gates do not
establish graphical fidelity or hardware performance. Repository CI provides
cross-platform/native gates after the PR opens.

World and runtime app primitives remain potential compatible consumers for a
later narrow migration. `hypot`, renderer scaled normalization, quaternion
validation and domain fallback policies intentionally remain separate. No
blocker remains for review; issue closure is deferred until PR merge.
