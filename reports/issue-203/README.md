# Issue 203 — Assisted flight energy reservation

Implemented isolated atomic budgets for current landing/takeoff paths, rejection
feedback, completion refund, blackout continuation and deterministic spending.
Portable reservation API is reusable by recovery when its path is implemented.

Validation: `git diff --check` passed. Added reservation and controller brownout
regressions; existing all-body timing/partition tests cover reserved paths.
Rust toolchain is absent locally and installation endpoints are unavailable;
workspace format/lint/tests/build are delegated to GitHub Actions. Native Apple
M1 signoff is intentionally excluded, as specified by the issue.

Estimate is conservative full forward power × duration for kinematic assists.
Recovery landing itself is separate planned work.
