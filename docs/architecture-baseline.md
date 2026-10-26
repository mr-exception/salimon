# Architecture baseline and domain ownership

## Reproduce

Run `python3 scripts/architecture_audit.py --history-ref 69c8394 --output artifacts/architecture/current.json --summary`.
Source metrics describe the working tree; history is pinned separately so
refactors compare the same 30 first-parent PR-tagged commits. Run from revision
`69c8394` for the original source baseline. CI uses current HEAD and retains the
complete JSON artifact. Audit failures/candidates initially do not block builds;
tooling contracts do block them.

The scanner covers first-party `client/*/src/**/*.rs`, excluding generated
directories and `spatial_contracts.rs`. Vendor and exported assets are excluded.
It records imports, module declarations, crate-reference edges and strongly
connected crate components. Imports include tests and macro text: this is a
lexical inventory, not resolved Rust module/type analysis. Function and branch
counts are complexity proxies. Comments and strings are blanked; character
literals, cfg, macros and aliases can produce false positives or missed edges.
Cargo/Clippy remain authoritative. Size alone never requires a split.

Overlap uses local first-parent subjects ending in `(#number)` and each commit's
first-parent file diff, including empty first-party intersections. It measures
co-change, not merge conflicts or current active-PR overlap. It does not fetch
GitHub or establish review/merge state. Shallow history reduces coverage; CI
checks out full history. JSON lists exact sampled commits and path pairs.

## Original measurements

At `69c8394`: 279 first-party Rust files and no observed crate-reference cycles.
The 30 sampled PR commits repeatedly touched these integration surfaces:

| Path | Lines | Branch tokens | PR commits |
| --- | ---: | ---: | ---: |
| `client/runtime/src/automation.rs` | 1950 | 162 | 18 |
| `client/runtime/src/session.rs` | 967 | 115 | 16 |
| `client/runtime/src/session/persistence.rs` | 833 | 111 | 15 |
| `client/runtime/src/app.rs` | 2046 | 311 | 12 |
| `client/runtime/src/app/input.rs` | 672 | 103 | 9 |

Proposed review thresholds: 1,000 lines, 200 branch tokens or 10 sampled PR
commits touching a path. These are review prompts, never rejection criteria.
Record false positives with path, cohesive responsibility, test weight and
measured rationale in the delivering report; review evidence before tightening
thresholds. Compare extracted files too, so moving code cannot hide complexity.

## Ownership map

| Concern | Stable owner and boundary | Next work |
| --- | --- | --- |
| Native events/focus/cursor | Runtime app input; winit ends here | #640 |
| Domain input admission | Runtime domain composition and typed domain commands | #640 |
| E2E envelope/reader | Runtime automation; reader never touches game state | #639 |
| E2E operations | Runtime domain handlers using production paths | #639 |
| Durable envelope/codec | Persistence crate; runtime checkpoints orchestrates IO | Preserve |
| Save mappings | Domain composition adapters; explicit validated publication | #641 |
| Tick/restore composition | Runtime session; deterministic explicit order | #642 |
| GPU resources/pipelines | Renderer; neutral DTOs from runtime adapters | #643 |
| Contacts/integration | Physics; preserve numerical invariants and stress tests | #644 review |
| Flight/assists | Ship; preserve controller authority and physical budgets | #644 review |

Runtime composes domains; domains must not depend on runtime. Renderer never
imports gameplay state. Persistence owns the durable envelope, not a dynamic
feature registry. Preserve typed snapshots and saved identities. The
[maintenance map](maintenance-map.md) identifies source/test owners and the
[technical architecture](technical-architecture.md) defines crate boundaries.
Large cohesive physics, mesh and flight files require function-level evidence
before extraction. #645 enforcement follows baseline and domain extraction review.
