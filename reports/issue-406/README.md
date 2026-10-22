# Issue 406 — catalog-bound future universe saves

Delivered separate universe version-1 envelope and typed ship/epoch/catalog,
world-definition and discovery contracts; mutable domain sections retain their
own versions and mandatory restore validation. `SaveManager` now treats catalog
incompatibility as fatal, preserving newer/different-world saves. Current runtime
schema 6 and save directories are unchanged.

[Canonical contract](../../docs/save-contract.md#future-universe-new-run-contract--u05-406),
[persistence architecture](../../client/persistence/architecture.md),
[invariants](../../client/persistence/invariants.md).

Linux Rust 1.99.1: persistence contracts, universe tests, workspace tests/build,
all-target Clippy, fmt and documentation lint pass. Tests inject a torn write and
prove recovery of the verified universe generation, then prove catalog mismatch
aborts recovery. Unknown version/delta, nonfinite pose, chronology, duplicate
identity and oversize inputs reject. Production loader verifies pack bytes;
this contract only pins their identity. Runtime adoption, physical world/delta
migration, scanner live contacts and offline propagation remain downstream.
