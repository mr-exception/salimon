# Issue #433 — chronological offline systems and scanner

Implemented on `codex/offline-universe-five-tasks` after #432 (`b8e97c4`).
Publication and final shared verification belong to the requested single PR.

The production launch now settles motion, ordinary Core/Fabricator power,
conversion, Recovery and eligible construction in chronological source order,
then funds the five-second scanner. Only fatigue recovers among survival meters;
hunger, health, active effects and active gameplay time freeze. Existing physical
cargo/frame sources remain frozen relative to their established containment.
No new engine, autopilot or Jump Drive authority is granted.

An idle, grounded ship without runnable work/conversion uses an analytic funded
pulse window and count. Monotone actual Core/service/pulse cost finds its funded
prefix in logarithmic work, retaining early successful observations before later
blackout. The remaining interval advances three-hour TTL and encounter history
without replaying every pulse. Curved flight, changing power/work and a possibly
in-range orbiting star use actual chronological physical samples; no long orbital
chord is certified as linear. Nacre and Vesper use the real published motion/frame.
The fast path is allowed only when the star is provably outside range for every
orbital direction, or when no energy exists to discover anything.

Scanner admission now uses fractional authoritative Core storage, validates both
contacts and chart before debit, and never erases an earlier funded pulse when a
later pulse fails. Canonical launch settles its final sub-tick remainder before
publication, aligning scanner, motion and work exactly. Legacy construction-only
sub-tick debt is settled before introducing the migrated motion epoch; engineering
fixtures retain their historical queued-remainder contract.

## Focused validation

Linux cloud, Rust/Cargo 1.99.1, locked debug profile, existing non-root toolchain
and sysroot. Parent/source hashes are retained in the final batch evidence.

- `cargo test -p salimon-client --locked session::scanner`: **15 passed**. New
  cases cover whole/split launch agreement at zero/low/full energy; funded early
  contact then blackout/three-hour expiry; a year-long analytic pulse count;
  Fabricator starvation and frozen survival/fatigue-only recovery; pause versus
  close; fractional-joule pulse admission; actual curved departure retaining an
  early in-range pulse; and a late counter-overflow failure rolling back launch.
- `cargo test -p salimon-client --locked session::offline`: **17 passed** before
  the final two scanner-only regressions; includes existing conversion recovery,
  exact construction receipts/energy/remainders, active-assist reservation,
  damage chronology, long absence and failed checkpoint publication cases.
- `cargo clippy -p salimon-ship -p salimon-client --all-targets --locked -- -D warnings`: **passed**.
- `python scripts/check_docs.py --base e68ccb8`: **passed**, zero findings.
- Final shared workspace/script/format and native evidence are
  shared with the remaining batch tasks, without duplicating unchanged suites.

Iteration failures were fixture/setup issues: the starvation fixture initially
supplied enough joules to finish its 10-second recipe, then was corrected to a
600 kJ budget. An initial unconditional remainder flush changed an engineering
fixture's historical queued-work contract; canonical-only flushing preserves it.
No tolerance, physics budget, TTL, cadence or source contract was loosened.

## Documentation and limits

[Canonical motion/timed-system contract](../../docs/universe-offline-propagation.md),
[scanner](../../docs/universe-scanner.md), [construction chronology](../../docs/construction-offline.md)
and [save contract](../../docs/save-contract.md) map the actual owners and tests.
The two-body authored local model does not establish full Gaia-pack coverage or
observed ephemeris validity. Hardware/Metal, physical input and subjective review
remain #563/#557/#561, separately unverified. Native progress/verified publication
is #434; final documentation audit and integrated evidence are #441/#435.


## Shared batch acceptance

Subsequent implementation and final cloud evidence are complete in the same
combined PR: [integrated results, original media and limits](../issue-435/README.md).
That final index supersedes earlier pending-batch statements while preserving
historical focused-check revisions/counts above. No merge or hardware acceptance
is implied. The audit precedes integrated verification; this link is additional
evidence, not a dependency from #441 to #435.
