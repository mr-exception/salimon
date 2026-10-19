# Issue 340 — Shared Core fabrication draw

Registered recipe watts as a separate Core consumer. The ordinary bus funds
thrust, cockpit, gravity and fabrication over one shared exhaustion boundary;
assist reservations remain isolated. Runtime advances the FIFO through shared
fixed ticks while away, landed, flying or assisting. Completion splits its draw
from remaining idle time; blocked/idle jobs draw nothing. Core telemetry separately
reports actual Fabricator watts and includes the load in total/depletion/braking.

Validation: focused runtime power tests cover fractional blackout/recharge,
no banking/double debit, completion/tray idle and stalled tick partitioning.
Ship load tests cover simultaneous consumers, invalid input and assist isolation.
Full runtime/ship regressions and workspace gates are recorded on the final stack.
`cargo fmt --all` and `git diff --check` passed. Linux x86_64 / Rust 1.99.1.

At an exhaustion boundary funded time is floored to Duration nanoseconds; less
than one ns of charged work can be lost, never granted free. No disk persistence
or native reference signoff is claimed; #348 tracks native gameplay verification.
