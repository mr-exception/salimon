# Issue 339 — Physical Fabricator intake

Implements session-owned fabrication and a universal E port with interior, 3 m,
30 degree, front-face and solid-sight gates. Existing Core/cockpit/door priority
and F carrying behavior are preserved. Intake works during blackout. The runtime
commits a validated carried identity once and immediately clears its mass,
physics, anchor and interpolation entries. Overflow is discarded; full reserves
reject without consuming. This deliberately corrects the #338 full-bin contract
to match #339. The adapter anchor is replaced by authored geometry in #342.

Validation: `cargo test -p salimon-client --locked session::fabrication`
passed 2 adapter regressions; `cargo test -p salimon-fabrication --locked`
passed 22 domain tests. `cargo fmt --all` and `git diff --check` passed.
Linux x86_64, Rust 1.99.1. Full runtime/workspace checks follow on the stack. Native visual signoff remains tracked by #348.
