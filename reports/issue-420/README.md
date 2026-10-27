# #420 — Continuous moving-body and terrain contact

Delivered on exact [#419 / PR #671](https://github.com/mr-exception/salimon/pull/671)
parent `0adc5c4220c3ee795e18ca3d8ebc52f3d9b7945b`, targeting
`issue-419-atmospheric-drag`. Review/merge #670 (#418), then #671 (#419), then this
PR. Nothing is merged or prematurely closed by this work.

[Canonical contract, proofs, source/test mapping and integration limits](../../docs/universe-contact.md).

## Result

Continuous motion-envelope broad phase and certified earliest-interval search
replace static/proximity assumptions in the real-scale flight API. Moving spheres
include spin velocity; moving authoritative triangle vertices preserve face,
edge and corner contacts and barycentric surface velocity. Stable canonical
body/primitive IDs resolve ties. Dense #417/#419 drift is checked before endpoint
gravity, so a proposed endpoint inside a planet still resolves its first contact
outside the solid. Body-relative grounded/rebound/fatal consequences, shutdown
observation, remaining time and unused funded-thrust receipts publish atomically.
Clear space exactly preserves the atmospheric integrator. Provider, geometry,
numeric, stale/mutated-witness and work-limit failures publish nothing.

## Cloud verification — 2026-10-27

Linux 6.18.55 x86_64 / 2 CPU cgroup quota / 8 GiB / Rust/Cargo 1.99.1, using the
same real native libraries/setup recorded in [#418](../issue-418/README.md).
No new dependency or renderer, current compressed-controller, input or save-schema
change. Final verification uses the final split source modules:

| Command | Result |
| --- | --- |
| `cargo test -p salimon-ship --locked contact:: -- --nocapture` | 13 passed, 0 failed; [original focused log](contact-tests.log) |
| `cargo test --workspace --locked` | 1,086 passed, 0 failed, 38 existing opt-in ignored |
| `cargo build --workspace --locked` | passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passed |
| `cargo fmt --all -- --check` | passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | 134 passed |
| `python3 scripts/check_docs.py --base 0adc5c4220c3ee795e18ca3d8ebc52f3d9b7945b` | 0 failures |
| `git diff --check` and final staged whitespace check | passed |

Tests exercise actual production gravity/drag/contact composition, independent
moving-sphere crossing time, an endpoint inside the source's gravity exclusion
radius, curved ephemeris crossing with clear endpoint chords, moving triangle
faces/edges/corners, triangle data from the production representative-terrain
query, orbital-frame/large-galactic-anchor invariance, stable tie order, paid
elapsed thrust, exact clear-space compatibility, body-frame impact energy and
all three consequences. A 1,002-body fixture runs only two narrow candidates;
Earth-radius tangent and 1 cm near-miss each require fewer than 200 samples.
Work exhaustion is deliberately tested as an atomic error, never fabricated
collision or successful movement. Input fixture thresholds are explicit and do
not approve new health/balance values.

Initial compile iteration required an explicit three-vector array type and one
module import after splitting the new owners. Two initial exact-equality checks
on Hermite-derived 100 m/s values failed at ~1e-14 m/s; final checks use 1e-9 m/s,
while independent event-time, spatial, energy and refinement assertions retain
their budgets. The complete final suite above passes. The Python suite's printed
timeout JSON is its expected failure-path fixture, not failed verification.

## Precision, inputs and adoption

Contact is proven for the supplied dense motion and collision snapshot, not an
unbounded approximation of arbitrary ephemerides. Epoch spans are <=60 s; source
providers own curve-fit accuracy and complete authoritative tile coverage. Missing
data must fail. Contact does not relax gravity interior guards, fill terrain
valleys with a nominal sphere, infer new physical radii or grant safe Landed state.
The hit records ordinary 1 mm tolerance and any unavoidable nanosecond-grid
uncertainty at uncapped speeds. Scene/work limits fail explicitly and need caller
refinement. Software query counts are not hardware FPS claims.

Default rebound constants preserve existing ship behavior. Fatal speed policy is
an explicit health-owner input (default nonfatal), not new unapproved damage
balance. The caller must honor the returned mode/propulsion shutdown, settle only
elapsed funding, and dispatch remaining time through support/rebound/terminal
handling rather than continued powered flight. This preserves #417's external
Core-allocation boundary; no second debit or automatic engine restart occurs.

Canonical playable streaming/terrain adoption remains #424/#425 and world/new-run
migration #426/#427; offline event and save publication remains #432–#434. These
existing owners consume the delivered real-scale domain APIs; this PR does not
pretend the future full universe/streaming client already exists. Current native
compressed gameplay and render output are unchanged, so no new graphical
capture gate applies. Physical/OS #557, human presentation #561 and actual
M1/Metal/GPU budgets #563 remain separately unverified release qualification;
no required routine cloud criterion is deferred here.

Final API audit also exercises modified hit records: nonunit normals and invalid
spatial tolerances reject before response, preventing fabricated rebound energy
or bypassed witness matching. The affected and complete cloud gates were rerun.
