# Issue #402 — Universe U01 schema foundation

Delivered 2026-10-21, stacked on Archive #467. `salimon-universe` is an isolated
portable leaf crate using existing locked Serde/JSON dependencies. It publishes
immutable revisioned systems/stars/planets/moons/individual minor bodies and
statistical populations, exact namespaced source identities, observed/inferred/
fictional per-field provenance, missing measurements, uncertainty and validity
metadata. It rejects duplicate IDs, dangling/wrong-kind/cross-system references,
cycles, invalid quantities/source metadata and unsafe schema input before publication.

Fictional planets/moons require versioned canonical definition/digest references,
independent named terrain/resources/visual seed domains and validated explicit
dependency DAGs. References are not full terrain definitions, verified digest
content or seed-generation algorithms. All fixtures are synthetic, not measured
science. IDs are source namespace/key strings, not row/default-hasher values.
Schema 1 is pinned by an independent checked-in fixture; unknown versions/fields
reject and future migration must be explicit. Review JSON is bounded to 16 MiB;
U04 owns actual partitioned binary packs and large-scale performance.

No runtime consumer, playable-world migration, navigation grants, production
catalog, source download, orbit/clock/physics, terrain evaluator, save migration,
backend or native hardware claim is added. Existing playable world/save/resource
IDs remain unchanged. U02/U04/U05/U06–U12 and native #436–439 retain their scopes.
Affected architecture, source maps, validation and implemented/deferred status are
updated in the same PR; documentation is not postponed to #441.

## Checks

Environment: Linux 6.18.55, Rust 1.99.1. Commands at repository root:

- `cargo test -p salimon-universe --locked`: 8 passed, no failures/ignored tests;
  [raw output](unit.log). Golden v1 roundtrip, deterministic identity/order,
  missing/provenance preservation, reference/cycle/quantity/domain failures,
  invalid/oversized/unknown-schema input and explicit immutable revision replacement.
- `cargo test --workspace --locked`: 775 passed, 27 explicitly ignored, no failures; [tests.log](tests.log).
- `cargo build --workspace --locked`: passed, [build.log](build.log).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed,
  [clippy.log](clippy.log).
- `cargo fmt --all -- --check`, `git diff --check`: passed.

Adding the local workspace member deliberately updates Cargo.lock with only its
existing Serde dependencies. The initial `cargo test -p salimon-universe` generated
that local-member lock entry; all final checks use `--locked`.
No native/Blender/GPU check is applicable to this unconnected portable schema;
no scientific accuracy, full-pack size/throughput, native FPS or downstream
integration acceptance is claimed.
