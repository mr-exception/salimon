# Universe schema maintenance

Read the active issue, root [AGENTS.md](../../AGENTS.md), this crate's
[architecture](architecture.md) and [invariants](invariants.md) before editing.
`identity.rs` owns stable source IDs; `records.rs` owns public input DTOs;
`validation.rs` validates publication; `lib.rs` exposes immutable catalog access.

Preserve the independent schema-1 fixture, missing/uncertain source distinctions
and existing playable world's IDs. Extend versioned wire contracts deliberately;
no runtime/renderer coupling, new scientific assumptions, migration, data import
or world evaluation is implied by a schema change. Run the architecture's checks
and update affected docs/report in the same PR.

U02: `coordinates.rs` owns split galactic cells, system/body/local transforms and
moving-frame velocity; `epoch.rs` owns split calendar instants and scale gates.
Read their architecture contract before changing numerical/time behavior.

U06 source metadata is in `sources.rs`; checksum verification and atomic raw-byte
staging are in `scripts/import_universe_sources.py`. Never promote fixture or
restricted/unreviewed source data into production by changing a checksum alone.
