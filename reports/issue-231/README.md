# #231 — continuous held mining and runtime integration

Native and automation left mouse now use the portable spherical lifecycle and
10 kg per-tool config. Session retains one stable attached identity, conservative
source debit and streaming tombstones, and coalesced physical mutation events.
Attached reserved entries follow tool front-left in f64 and cannot be picked up
or simulated loose. Release keeps the same ID/mass/pose and enters existing
loose physics. Continued hold creates a fresh identity next tick after cap.

Mouse-up, unequip, focus/cursor/view/lifecycle resets release synchronously.
Native/automation camera input shares a zero-time aim update so away/back between
simulation ticks releases without reconnecting a previous piece. F remains only
carry/drop, with no mining alternate. Mining never debits Core energy.

Automation retains existing counts/total mass, adding attached status, reserved
mass, capacity and attached identity, plus stable-ID fragment inspection. Scenario baselines/evidence update obsolete
2 kg-tail expectations to sealed partial pieces plus 10 kg releases. Carry/transfer walkthroughs select exposed 10 kg pieces (IDs 4 and 3) rather
than aiming through the new pile at the first buried piece. Stable-ID assertions
continue tracking onboard cargo when other pieces leave the nearby list; the
ship-local position baseline reflects the larger carried object. The revised
continuous-emission route covers auto continuation, immediate aim loss,
reacquisition while held and complete depletion/conservation.

## Validation

Linux x86_64, Python 3.12, Rust stable 1.99.0, locked dependencies.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test -p salimon-world --locked --test spherical_mining`: four tests passed,
  including stable session IDs, reserved-mass protection and stale-copy conservation.
- `cargo test -p salimon-client --locked automation::tests`: all 24 passed,
  including continuous-cap mining, aim loss, F isolation, carry, transfer and streaming.
- `cargo test --workspace --exclude salimon-client --locked`: passed all remaining
  domain/renderer/physics contracts, with explicit existing opt-in skips.
- `python -m unittest discover -s scripts -p 'test_*.py'`: all 58 passed.
- `python -m unittest discover -s scripts -p test_baseline_scenarios.py`: both pass;
  visual evidence variants preserve baseline actions/assertions.
- `git diff --check`: passed.

Earlier walkthrough failures exposed obsolete piece-count/mass/target assumptions,
then a nearby-array index disappearing in flight. Corrected scenarios preserve
explicit IDs through the new inspection map; assertions remain enforced.
Earlier runner checks hit existing 100 ms subprocess fixture timeouts during
concurrent compiler load; the final complete Python suite passed without timeout
changes. No live GPU, native M1 or sphere-visual verification is claimed.

## Boundaries and follow-ups

Legacy extraction helpers remain for fixture/benchmark population and their
portable tests; interactive input no longer uses facet emission. Full sphere
rendering is #232; radius-based contacts, safe release clearance/velocity are
#233; final spherical carry/drop/Core geometry migration is #234; performance
regressions are #235 and native M1 signoff is #236 (art signoff also #229).
Current interim visuals/loose contacts still use checked-in chunk/shard assets;
this PR makes no sphere-rendering/native visual claim. Existing Core conversion
and single-object carrying remain available after release. There is no disk
save/backend format; session state remains in memory.

A cap tick releases one sphere and starts the next on the next fixed tick;
unused work from a capped tick is not banked. At 2 kg/s, 320 ticks produce
10.224 kg (10 kg + 7 × 0.032 kg), rather than an unbounded split loop or dust.
