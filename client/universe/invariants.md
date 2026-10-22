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
8. Canonical truth is not player-known navigation. No production catalog, physics,
   gravity, scanner, save migration or world replacement is implemented here.

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
