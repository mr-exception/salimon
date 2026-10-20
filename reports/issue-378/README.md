# Issue #378 — four nutritional items and captured food modifiers

## Outcome

Implemented basic ration, nutrient gel, complete meal and energy drink definitions
in the existing survival catalog, sharing stable IDs with the Fabricator recipe
book. Basic food and existing medkits remain initially unlocked; all three advanced
foods remain locked. No acquisition mechanism, inventory expansion or new hotkeys.

Gel halves active fatigue drain for ten minutes. Meal feeds +70 in ten six-second
ticks and boosts eligible natural regeneration for five minutes. Drink adds +10
hunger and reduces fatigue penalties for five minutes without restoring fatigue
or bypassing exhaustion bans. Basic ration remains +40 immediately. Existing
medkit recovery is unchanged. Modifiers use validated serializable data, one-stack
refresh, strongest same-kind selection and the existing active effect pipeline.
A saved fractional remainder preserves exact fatigue drain across nanosecond
partitions. Old saves default that missing field to zero; captured old recipe/
effect catalogs remain intact rather than being silently replaced on load.

Explicit provisional settings: meal natural-regeneration multiplier 20000 bps;
drink fatigue-penalty relief 5000 bps. Advanced production quantities, durations,
power and masses are documented in the [fabrication architecture](../../client/fabrication/architecture.md#advanced-food-definitions-378).
These are balance defaults subject to #384, not newly approved fixed values.
The [canonical food contract](../../client/survival/architecture.md#four-food-catalog-378)
documents every item and numerical rule.

## Coverage

- Survival tests prove exact hunger amounts, expiry, regeneration eligibility,
  exhausted permissions, no fatigue recharge, refresh/stack limits, offline freeze,
  validated configuration and old-save decoding.
- Nanosecond-partition/save continuation checks retain fractional drain.
- Fabricator regressions verify the five-definition book and initial availability.
- Runtime tests exercise all four foods through ingredient insertion, funded Core
  production, tray collection, exact-slot consumption and whole-game save/restore.
  Advanced production uses explicitly unlocked fixture books; production defaults
  reject their enqueue atomically. No progression API was added.
- Status details display continuous modifiers; physical restrictions and HUD share
  the same policy. Existing stale click, full-meter waste and modal guards remain.

## Validation

Environment: Linux x86_64, Rust/Cargo 1.99.1, base `8666fac`.

| Command/check | Result |
| --- | --- |
| `cargo test --workspace --locked` | 681 passed, 22 opt-in ignored, 0 failed; includes all six new tests |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo build --workspace --locked` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `git diff --check` | Passed |
| Changed-document relative file links | 131 checked, passed (before this report) |

An initial Fabricator test still expected every recipe to be unlocked; updated
that obsolete assertion to verify exactly basic food plus medkit availability.
The final workspace run includes the added runtime four-food round-trip test.

## Boundaries and remaining verification

- Native Q-wheel input/presentation is the already tracked #377 draft; this PR
  provides catalog/effects through the shared exact-identity transaction.
- Reference Apple M1/Metal visual/input signoff remains #376. Native GUI checks
  were not run on this Linux session; no native appearance or FPS pass is claimed.
- Advanced food acquisition remains deferred. The fixture unlock used by tests
  cannot be invoked through gameplay input or automation.
- Provisional numerical balance requires the existing #384 scenario/signoff work.
- No dependency metadata or existing draft PR was changed. #368 is open but its
  implementation PR #394 is merged, and #370 is closed/implemented; #378's
  implementation prerequisites were therefore present in main before this work.
