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

## Warning-only boundary and active-PR review (#645)

Run `python3 scripts/architecture_review.py --base origin/main --output artifacts/architecture/review.json`.
The base must be the actual PR parent (including a stacked parent's SHA), not an
arbitrary default branch. The working tree is compared with that base. CI retains
JSON and plain-text diagnostics alongside the size/history inventory. Every
candidate names its path, rule and reason; findings never fail the build in this
initial phase. A tooling execution failure remains visible through the step
outcome, rather than being reported as zero violations.

`scripts/architecture-policy.json` freezes the observed crate-reference edges
at the merged #638–#642 state. Unlisted directions prompt review, including
supporting domains importing runtime or renderer importing gameplay. New
cross-domain references and crate/super imports prompt ownership review even
when their crate direction is allowed. New strongly connected components are
compared to the actual base. Changed files meeting the original size/branch
thresholds get a broad-file prompt. All are lexical candidates: cfg/test imports,
aliases, macros, changed component membership and cohesive large files require
review; Cargo/Clippy remain authoritative. The check does not resolve every Rust
module import or prove absence of architectural cycles.

CI reads all open PRs, including drafts, with paginated file lists and read-only
repository permissions. It excludes the current PR and records each competing
head SHA and common changed path. Inputs never execute PR code or interpolate
PR titles/bodies into shell commands. Locally, pass `--active-prs FILE` with a
JSON array of `{number, head_sha, draft, paths}` and optional `--current-pr N`.
Missing/API-failed inventory is explicitly unavailable, not an empty backlog.
A 3,000-file API ceiling fails collection instead of presenting partial coverage.
Inventories are point-in-time observations; recheck before branching/merging.
Path overlap is a review signal, not proof of conflicting hunks or dependency.

For a false positive, inspect source/tests and record the cohesive owner,
contract, exact rule/path/detail and reason in the PR/report. Add a narrow policy
exception with `owner` and `reason` only after review; exact matching prevents a
blanket path exemption hiding future changes. Changing thresholds or allowed
edges requires the same review, with measured rationale and affected tests.
Unexplained direction/cycle changes go to the owning-domain reviewer; shared
contracts and unresolved ownership decisions go to Project Q&A before adoption.
Do not tighten any warning into a failing gate until baseline review has accepted
its false-positive rate and existing valid pipelines have been exercised.

PR planning: dependent work branches from its prerequisite; independent disjoint
owners may branch from main when the requested workflow permits it. Shared-path
work first compares hunks/contracts; overlap alone does not justify rejection or
a forced rebase. An explicitly requested linear stack takes precedence: this
batch is #645 → #643 → #644, with each child targeting its parent's head. Preserve
that parent while reviewing and merge oldest first. Each implementation PR carries
its own closing reference from creation.

Measure change using fixed history windows and retained source revisions: compare
candidate counts by rule/path, suppression count/reasons, active-PR shared-path
counts, and actual reviewer-confirmed conflicting hunks. Re-sample after future
independent changes; do not call co-change or a lower file length a measured
reduction in merge conflicts. This initial delivery establishes observations and
review policy, not a demonstrated improvement in future PR overlap.
