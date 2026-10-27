# Universe schema invariants

1. A published Catalog revision exposes no mutable source or identity access.
2. Exact namespaced IDs are independent of names/order/session/clock and remain
   stable when a definition is explicitly republished under a new revision.
3. Duplicate global IDs, missing/wrong-system/wrong-kind references and parent
   cycles reject before publication.
4. Per-field observed, inferred and fictional provenance remains explicit;
   missing data never becomes zero or measured certainty. Observed source keys
   and source versions survive interchange.
5. SI quantities, uncertainty and declared validity must satisfy the documented
   finite/positive/order constraints. Validity metadata performs no time transform.
6. Fictional worlds retain versioned canonical definition/seed-domain references;
   a seed alone is never a full world definition. No runtime generation choices.
7. Unknown schemas/fields, duplicate wire fields and malformed/bounded JSON reject;
   schema evolution requires deliberate versioned compatibility tests.
8. Canonical truth is not player-known navigation. Portable ephemeris/gravity
   services preserve explicit epochs, split positions and reference checks.
   Scanner knowledge, playable-world migration and release catalog qualification
   retain their separate owners; no service grants player discovery.

The [architecture](architecture.md) is the canonical API/limit/ownership contract.

9. U02 galaxy anchors remain split cells/local offsets. Narrowing to GPU f32
   occurs after relative subtraction; common frame ancestry cancels first.
10. Parent-relative velocity includes rotating-frame transport; every frame in
    one tree is evaluated at one declared epoch. Unknown/cyclic/stale frames fail.
11. Epoch scale labels survive serialization. Leap seconds, implicit scale
    conversion and unsupported cross-day UTC arithmetic fail explicitly.

12. Source download access never implies reuse approval. Production manifests
    reject restricted/unreviewed sources and synthetic artifacts; raw bytes must
    match frozen sizes/digests before offline staging. No runtime network path.

13. Partition indexes reject overlapping/gapped/out-of-file ranges, duplicate
    identities, unknown versions/fields and conflicting immutable source metadata.
    A requested cell must pass CRC, bounded exact zlib consumption, U01 validation
    and index/revision/source agreement before exposure. Unread cells remain unverified.
14. Storage is lossless for admitted U01 values; CRC32 is corruption detection,
    not cryptographic publication approval. Prototype limits and measured sample
    density never establish production size, catalog completeness or M1 performance.

15. Reference comparisons require complete unique case coverage and identical
    exact frame/epoch/scale; finite Euclidean SI errors are checked against
    explicit finite budgets. No implicit conversion, missing-data pass or
    self-updated golden. Fixture rounding/numerical budgets are not scientific
    model-error or native performance approval.

16. U07 preserves exact Gaia IDs, all selected source columns, measured errors,
negative/missing parallax and missing velocities. No distance or motion is invented.
17. Reviewed cross-matches preserve every source observation; conflicting duplicate
rows/ambiguous matches fail. Canonicalization is input-order independent. Selection
counts and explicit policy accompany results; bounded decoding is distinct from
collector memory. See the [Gaia contract](../../docs/universe-gaia-ingestion.md).

Gaia RA/Dec source errors stay in explicitly named mas fields, separate from
degree-valued coordinates; raw source error/correlation evidence is preserved.

18. U08 states carry explicit ICRF/SSB geometric frame and TDB epoch; imported
coverage is not full model validity. Exact-state lookup refuses unimported dates
and time-scale conversions. Physical header validity/uncertainty stays separate.
19. PS source keys/Gaia matches, raw measurements, literature, signed asymmetric
errors, one-sided errors and limit/minimum-mass distinctions survive ingestion.
Missing orbital elements never become complete states. Canonical/base identity
conflicts reject; source-order changes cannot alter publication. See the
[observed ingestion contract](../../docs/universe-observed-ingestion.md).

20. U09 never overwrites a present quantity or its uncertainty/validity/provenance.
Inferred values retain versioned methods, proposal distributions, qualitative
confidence, registered dependencies and reproducible seeds/attempts. Proposal
support is not a calibrated posterior; derived uncertainty stays unknown.
21. Only complete supported single-star systems passing bounded point-constraint,
Roche, radial-separation and mutual-Hill screens publish a new revision. Failure
never mutates input. U08 limits/conflicts require explicit reconciliation; minimum
mass never becomes observed true mass. Missing phase/epoch/orientation stays
missing. See the [inference contract](../../docs/universe-inference.md).

22. U10 synthesizes fiction only offline from explicit version/seed/reviewed
residual-density regions; observed catalog records/datasets pass through intact.
Reserved namespace collisions, overlapping supports and invalid/oversized inputs
reject atomically. Permanent keyed slots and named SHA-256 lanes do not depend
on input iteration, unrelated regions, install/session state or runtime RNG.
Publication arithmetic fixes cube multiplication order; debug/release builds
on supported platforms must match the same full-publication golden hash.
23. Published U10 output retains the full generation request and typed
position/age/metallicity/body/orbit/minor-population recipes. Actual recipe bytes
match definition-reference hashes; aggregate members remain unallocated and
exclude individually materialized selected bodies. Generation references do not
claim complete U11 terrain definitions. Recipe changes require explicit new
publication/migration; the runtime never silently regenerates a visited world.
See the [builder contract](../../docs/universe-fiction-builder.md).

24. U11 freezes one fixed system/star/planet ID and exact candidate/world hashes.
Changed inputs or unsupported manifests fail; no fallback/reselection per player,
install or launch. Names never replace scientific identity. Earth-distance proof
uses common split coordinates and conservative orbital envelopes at publication.
25. U11's full selected-world manifest is distinct from U10's recipe reference.
All surface consumers use its single height query; the visual domain cannot alter
surface/resources. Fixed semantic deposit IDs, positive starting quantities and
ship-clearing <=55 m walk paths are baseline inputs, not mining/persistence deltas.
Native/playable U24 integration and explicit revision migration remain required.
See the [starting-system contract](../../docs/universe-starting-system.md).


## Incremental private offline packs (#413)

See the [implemented contract](../../docs/universe-pack.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Bounded read-only pack queries (#414)

See the [implemented contract](../../docs/universe-reader.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Time-addressable ephemeris models (#415)

See the [implemented contract](../../docs/universe-ephemerides.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Dominant-source celestial gravity (#416)

See the [implemented contract](../../docs/universe-gravity.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Gravity-aware ship trajectory (#417)

See the [implemented contract](../../docs/universe-trajectory.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.
