# Issue #469 — Player-known chart and coordinate provenance

Implemented a durable chart independent of immutable scientific catalog truth.
It starts empty and obtains verified knowledge only from actual scanner encounter
history. Earth has no autocomplete, initial coordinate or route, even when its ID
exists in the catalog. Observed chart knowledge survives contact expiry; unknown,
inferred, transmitted and observed evidence remain distinct.

Leads retain a versioned source identity, received UTC, named coordinate frame and
explicit unknown/radius uncertainty. Identical-source replay is idempotent; changed
source data requires a new identity. Runtime admission stages/validates the source
before publication. Inferred/transmitted hints cannot verify themselves or create
observed routes. Actual transmission/recovery production remains #470's existing
scope; this PR supplies its validated admission/display/persistence boundary and
does not invent completed transmissions or galactic astrometry.

P switches the canonical cockpit projector between fresh scanner contacts and the
chart; M selects a known entry and Enter records a same-frame observed route.
The display identifies observed/historical/inferred/transmitted status, sampled
center-distance, ship-local azimuth/elevation, uncertainty, saved route count and
explicit frame-unavailable state. Vertical targets have undefined azimuth (AZ--).
The chart never starts engines, thrust, autopilot, jumps or teleports. Earth and
Absenat remain independent voluntary goals, with no new wormhole claim.

Schema 26 validates the chart/scanner pair before capture and publication,
including exact encountered identities/states, timestamps, duplicates and catalog
revision. Canonical versions 1–25 migrate from actual scanner history only.
View/selection are transient; discoveries, lead evidence and immutable route
snapshots roundtrip. Rediscovery and changed range parameters retain that source.
The chart is bounded to scanner capacity plus 256 leads and 4,096 route records;
geometry remains bounded to 64 projected contacts/227 cuboids. No broad catalog
search, guessed frame transform or all-star per-frame rendering is introduced.

Source/test mapping: [scanner/chart contract](../../docs/universe-scanner.md),
[save contract](../../docs/save-contract.md),
[Archive/navigation boundary](../../docs/archive-recovery-and-navigation.md),
[universe architecture](../../client/universe/architecture.md),
[runtime invariants](../../client/runtime/invariants.md). Owning renderer and
persistence guides plus root routing/validation/status notes are updated together.
Stale current-envelope version references are corrected; historical reports remain.

## Production evidence

Original cloud Linux Xvfb 1280x800 scale-1 captures use production wgpu Vulkan on
Mesa llvmpipe (two software-renderer threads), with the existing declared landed
`canonical-flight` cockpit fixture and dedicated scanner reference camera. Protocol
input/camera placement is not physical keyboard, normal-play boarding or display
qualification. The quiet three-hour offline adapter settles ordinary Core work and
ages contacts; it does not implement or certify #432/#433's future orbit/scanner
catch-up host. Missing orbital phase/velocity is never invented.

- [Empty chart](empty.png): no player knowledge before a paid observation.
- [Observed route](observed.png): selected known Vesper and voluntary route count.
- [Restored route](resumed.png): route/discovery retained after actual save/restore.
- [Historical chart](historical.png): scanner has zero fresh contacts, while the
  chart retains the observation and route with explicit HISTORICAL status.

`verify_images.py` reads originals and state summaries, checking the fixed
projection ROI and source separation. `image-metrics.json`, `capture-context.json`,
`state-summaries.json` and compressed full results/protocol/stderr retain exact
provenance, hashes and actual outcomes. These are not promoted visual baselines.

## Final integration correction

Review found that bulk offline queries preserved only the latest sample, losing
actual first-sighting time/count. Fixed in owning #429 PR #681 (08b44eef), then
merged through #682/#683 without force pushes before this implementation's final
checks. Sweeps now preserve earliest/latest/count analytically; tiny nonzero
motion cannot create overflow-root false crossings. Live contact validation is
staged before Core debit; a saved-counter rejection regression here proves failed
refreshes cannot repeatedly bill the same pulse. The owning report retains its
focused evidence, while this report records full-stack regression.

Initial integration fixed a missing renderer DTO state field, one Clippy nesting
warning and the declared pre-chart legacy fixture. An initial chart capture exposed
an anonymous planet label/undefined zero azimuth; final presentation uses the
published observed Vesper name and AZ--. Preliminary captures are not final evidence.

Physical input/platform #557, human appearance/readability/comfort #561 and real
GPU/M1/Metal budgets #563 remain separately unverified. Scientific full-data
coverage #524/#526 and global-frame/offline host adoption #427/#432/#433 keep their
existing scopes. Software results do not establish physical 60 FPS or source/balance
approval, and no undiscovered locations or approved baselines were promoted.

## Validation at the final stack revision

- `cargo test -p salimon-universe --locked`: **104 passed**, zero failed.
- `cargo test -p salimon-client --locked chart`: **5 passed**, covering actual
  save/restore, provenance/uncertainty admission, route/power nonmutation, legacy
  migration, range/rediscovery and rejected-counter billing conservation.
- `cargo test -p salimon-client --locked scanner`: **5 passed**.
- `cargo test --workspace --locked`: **1,133 passed**, zero failed; 38 existing
  opt-in stress/device/evidence tests ignored under their original conditions.
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **134 passed**;
  expected negative-path fixture messages are not suite failures.
- `python3 scripts/check_docs.py --base codex/issue-430-scanner-hologram`: zero failures.
- Production native chart route: **34/34 passed**, four original captures.
- Production scanner/hologram regression on this same final binary: **28/28 passed**.
- `python3 reports/issue-469/verify_images.py`: passed (original-image and matching
  state assertions), metrics retained beside the report.

Merge order: #680 → #681 → #682 → #683 → this PR. The final parent head is
`21c2d8ea151e35654652718b1b5a36d9e789a4d3`; each ancestor contains the previous
PR's head, including the final owning pulse correction. No PR was merged here.
