# Issue 192 — Cockpit blackout authority

Steering now consumes only the powered interval; outage and cockpit exit clear
held input/ramp residue without changing signed throttle, heading or momentum.
Native and automation steering use the same powered ownership/assist gate.
Power restoration resumes selected thrust independently of cockpit occupancy.

Validation: `git diff --check` passed. Added signed blackout/coast/restoration,
exact steering depletion-boundary, exit-ramp and denied-input replay regressions.
Workspace format/lint/tests/build run in Actions; local Rust toolchain unavailable.
Apple M1 signoff excluded by issue scope.
