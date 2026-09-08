# Runtime

The `salimon-client` binary is the client composition entry point. For task 1 it
prints the repository-shell status and exits successfully; it creates no window
and performs no simulation.

Run from the repository root with `cargo run --locked -p salimon-client`.
Task 2 adds lifecycle/orchestration for native windowing and rendering. Keep
domain behavior in its owning modules and OS-specific operations in platform
adapters. The runtime must not acquire backend responsibilities in Phase 0.
