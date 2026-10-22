# Issue #410 — scientific missing-data inference

Portable U09 delivery, 2026-10-22. Base: `7b96f664594d541d9f0ffae86b950349e7b511b3`
(U07/U08 merged). Review/merge remains owner-controlled. This report records
heuristic model behavior, not production-data or native hardware approval.

## Delivered behavior

- Typed offline single-star requests, immutable catalog completion and separate
  age/partial-orbit/distribution/confidence sidecars. Present measurements and
  their source uncertainty/validity/identity are preserved verbatim.
- Versioned, fixed-seed main-sequence/rocky/giant priors with explicit source
  dependencies; 256-attempt atomic physical screening, minimum mass, Kepler,
  Roche and mutual-Hill/radial separation gates.
- Strict U08 adapter: source limits/unspecified flags, conflicting host values,
  wrong topology and unsupported systems fail rather than become invented data.
- Nine focused regressions and a repeatable 8192-system JSON sanity workload;
  CI runs tests and compares two complete workload outputs.

Sources/tests/owning contracts: [inference](../../client/universe/src/inference.rs),
[U08 adapter](../../client/universe/src/inference/observed.rs),
[tests](../../client/universe/src/inference/tests.rs),
[workload](../../client/universe/examples/inference_sanity.rs),
[architecture](../../client/universe/architecture.md#missing-data-inference--u09-410),
[invariants](../../client/universe/invariants.md) and
[canonical scientific policy](../../docs/universe-inference.md).

## Fixed-seed scientific sanity

[Full JSON evidence](sanity.json), seeds 0–4095 per class. Both classes retain broad
five-bin proposal coverage (each bin 10–40%); this is an anti-collapse check, not
an empirical distribution fit or posterior calibration.

| Measured result | Rocky | Gas giant |
| --- | --- | --- |
| Accepted systems | 4096 | 4096 |
| Candidate attempts | 4096 | 4126 |
| Constraint failures in accepted systems | 0 | 0 |
| Largest relative Kepler residual | 5.551115123125783e-16 | 5.551115123125783e-16 |
| Smallest pericenter / collision-or-Roche limit | 1.686505918246043 | 1.0035754100700753 |
| Stellar mass five log bins | 824, 837, 804, 832, 799 | 827, 838, 808, 833, 790 |
| Semimajor axis five log bins | 804, 790, 852, 831, 819 | 791, 790, 854, 836, 825 |
| Normalized age five linear bins | 820, 794, 822, 840, 820 | 822, 798, 819, 840, 817 |
| Eccentricity five linear bins | 829, 808, 836, 802, 821 | 829, 804, 835, 805, 823 |

Rocky density log bins: 813, 859, 786, 794, 844 across 3000–8000 kg/m³.
Tests separately check 1024 seeds, immutable observed quantities/errors/validity,
terrestrial one-year Kepler reference/inverse, physical conditional branches,
minimum-mass refusal, invalid/nonfinite/extreme input, source coverage/conflicts,
multiplanet order independence and crossing-orbit/second-star refusal.
The real frozen Proxima period/errors survive a **test-only** explicit missing-limit
flag review; direct unreviewed U08 adaptation correctly rejects. No actual Proxima
mass/radius/composition/age or production source approval is claimed.

## Validation

Linux x86_64, Rust/Cargo 1.99.1, Python 3.12.14, locked dependencies. Local tool
setup sourced from `/workspace/.salimon-tools/env.sh`.

| Command | Result |
| --- | --- |
| `cargo test -p salimon-universe --locked` | 53 passed, 0 failed/ignored (9 new inference regressions) |
| `cargo test --workspace --locked` | 833 passed, 0 failed, 28 intentionally ignored opt-in tests |
| `cargo build --workspace --locked` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo run -p salimon-universe --example inference_sanity --locked` (twice), `cmp` | Passed; full output identical, 8192 systems, zero accepted-system physical failures |
| `cargo run -p salimon-universe --example observed_import --locked` (twice), `cmp` | Passed; existing U08 output remains reproducible |
| `python3 -m unittest discover -s scripts -p test_check_docs.py -v` | 5 passed |
| `python3 -m unittest discover -s scripts -p test_import_universe_sources.py -v` | 4 passed |
| `python3 scripts/check_docs.py` | 0 failures, no baseline debt |
| `git diff --check` | Passed |

See [validation summary](validation.txt) for retained test-binary totals and check
results. `python3 scripts/check_docs.py --base 7b96f664594d541d9f0ffae86b950349e7b511b3`
also passed after committing: 0 failures and no baseline debt.
Native/graphical/reference-machine gates were not run (isolated offline domain).

## Documentation impact

Updated crate README/AI/architecture/invariants, root README, technical architecture,
maintenance map, validation guide, approved-design status and observed-ingestion
consumer contract, plus the new canonical inference guide. Source/test/contract
mapping is linked above. U01/U04 schema remains unchanged; age/orbit/evidence
sidecars require explicit U12 packaging. No runtime/scenario/save interface or
renderer/asset changes; those guides remain accurate and need no change.

## Limitations and downstream ownership

The priors are scientifically motivated bounded heuristics, not fitted empirical
stellar/planet populations or calibrated credible intervals. Model classifications
are explicit caller decisions. Physical screens use point quantities and are not
long-term dynamical stability proofs; observation errors are preserved, not
propagated as fabricated certainty. Only single low/intermediate-mass main-sequence
stars, rocky planets and gas giants are admitted. Unsupported observed systems
reject; there is no unmarked approximate replacement. Complete orbit phase/frame/
epoch/orientation and Gaia distance inference remain absent.

Production source approval/full inputs remain existing follow-ups
[#513](https://github.com/mr-exception/salimon/issues/513),
[#524](https://github.com/mr-exception/salimon/issues/524) and
[#526](https://github.com/mr-exception/salimon/issues/526). Canonical population/world
building remains U10/U11; sidecar publication/version verification remains
[U12 #413](https://github.com/mr-exception/salimon/issues/413), full ephemeris evaluation
[U14 #415](https://github.com/mr-exception/salimon/issues/415). These existing scopes
remain blocked by their own requirements; portable U09 completion does not resolve
them. Native playable real-scale integration/signoff remains #436–#439. No native
M1/Metal graphical/performance tests were run for this isolated offline boundary.
