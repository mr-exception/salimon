# Issue #429 — Five-second powered scanner

Implemented `universe::scanner::pulses` and the ordinary session/Core adapter.
Fixed automatic five-second pulse boundaries, inclusive celestial range, future
40% noncelestial range, continuous/per-pulse costs and unpowered expiry are
explicit. No manual scan or alternate passive mode exists.

Analytical swept-range tests retain an encounter that enters and leaves range
between the endpoints of an offline interval. Latest pulse lookup is constant
work per supplied object/piece, including a 10^12 ms interval; it never loops
through the pulse count. Atomic `scan_linear` validates whole batches and
requires a host-certified same-frame affine path/funding window. Nonlinear
trajectory and chronological powered-window production remains #432/#433,
whose existing scope explicitly consumes this API. This PR does not certify
unimplemented gravity-only closed-game propagation.

The canonical runtime's currently represented Vesper state is body-local, with
exact center/velocity in that frame. Its absent galactic phase/velocity is never
invented. `Position` distinguishes body-local and split galactic observations;
mismatched frames and invalid/unknown state reject. Other celestial sources can
be detected only when actual evaluated same-frame states are supplied. Neither
catalog loading nor scientific identity grants prior player knowledge.

The runtime samples the completed authoritative tick at crossed pulse boundaries
(maximum one 16 ms tick latency), charges real Core joules after other work, and
never backdates a fabricated state. Initial configurable defaults are 1e9 m,
1 kW and 10 kJ/pulse, not approved balance/hardware targets. Pause/terminal freezes
running simulation. Inspection exposes actual source, freshness and errors.

See [contract and source/test mapping](../../docs/universe-scanner.md), owning
[universe architecture](../../client/universe/architecture.md) and
[runtime invariants](../../client/runtime/invariants.md).

Cloud Linux x86_64, Rust 1.99.1. No rendered behavior changed, so no image claim.
Checkpoint mapping and launch handling follow in #431; physical/platform/hardware
qualification stays #561/#557/#563. Initial validation corrected an exact decimal
float test assertion, a fixture with no Body record, and a large inspection JSON
macro expansion by building the scanner observation separately.

Validation at the final source revision:
- `cargo test -p salimon-universe --locked`: 99 passed, zero failed.
- `cargo test -p salimon-client --locked scanner`: one production-session test passed.
- `cargo test --workspace --locked`: 1,118 passed, zero failed, 38 existing opt-in ignored.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 scripts/check_docs.py --base codex/issue-428-scanner-contacts`: zero failures.

## Final stack review correction

Bulk offline sweeps now preserve the first sighting and exact pulse count as well
as the latest sample, using at most two chronological events per object. The
5/10/15-second crossing regression asserts first=5 s, last=15 s and count=3.
A tiny-relative-speed regression also prevents inf-inf roots from inventing a
crossing, including subnormal motion across a pulse boundary. Public last-detection
API and bounded work remain unchanged. This owning-issue fix is propagated through
the existing child PRs without force-pushing or dropping their changes.

Correction validation: 100 universe tests and the real runtime scanner test pass;
workspace all-target Clippy, fmt and parent-relative docs checks pass. Live contact
transactions are staged before Core debit so rejected state cannot repeatedly bill
the same pulse. The final chart PR adds a saved-counter rejection regression and
reruns full-stack validation after this correction.
