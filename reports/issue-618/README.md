# Issue #618 — rare biosphere publications and ecological regions

Parent: non-draft PR #690, branch `issue-617-active-geology`, commit
`9ac7b577b155b6ca31381ce3cbb3e51f68483a9b`. This preserves one linear stack over
#615/PR #689, with merged #612 geology and #611 design in ancestry. #618 depends
on #615's climate; #617 is its sequential PR parent, not an ecology blocker.

World now separates bounded geology/climate-based ecological potential from a
fixed published Unknown/Sterile/MicrobialOnly/Developed decision. Rare fictional
selection has independent versioned seed domains; observed bodies retain no
procedural life claim and Vesper remains explicitly sterile. Continuous spherical
realm/community descriptors, family grammar identity and sparse exact physical
patch anchors survive concurrent queries and stream/restore without rerolling.
No actual organism, mutable biology, regrowth, resource recipe, collider, chart
marker or renderer geometry is introduced; those remain #619–#621.

Strict publication and bounded sorted catalog codecs bind exact catalog body,
climate and definition IDs. Missing or revised save references fail closed instead
of falling back to another body or silently migrating old biological identities.
Existing U11/session save schemas, Vesper publication bytes and feedstock identities
remain unchanged. The [ecology contract](../../docs/planetary-ecology.md), owning
world guides, architecture/maintenance/validation/root routing and approved-design
status are updated together.

## Verification

Debian 13 Linux x86_64, Rust/Cargo 1.99.1, existing repaired development environment
selected by `. /workspace/.salimon-tools/env.sh`.

- `cargo test -p salimon-world -p salimon-ship --locked`: **250 passed, zero failures,
  three existing opt-in ignored**. Eight focused ecology tests cover seed-batch
  rarity (981 sterile, 16 microbial, three developed out of 1,000 suitable seeds),
  an independently calculated potential vector, existence versus suitability,
  observed-body and sterile rules, spherical seams/climatic variation, original
  physical anchors, concurrent hosts, stream/save reconstruction, malformed
  publication, duplicate/missing catalog references, revision mismatch and Vesper.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- `python3 scripts/check_docs.py --base issue-617-active-geology`: passed against the committed diff, zero failures.

[Raw affected tests](tests.log), [Clippy](clippy.log), [workspace build](build.log).
The full unchanged runtime/tooling baseline is retained in [#615's report](../issue-615/README.md):
1,152 workspace and 134 tooling tests passed. This child reruns affected world and
ship tests and workspace build/lint; it does not claim the unchanged full runtime
matrix was rerun here. The initial homogeneous numerical test expected zero cold
weight; it was corrected to the independent fixed equilibrium temperature and
published community formula before the final successful suite.

## Limits and downstream ownership

Potential and rarity coefficients are explicit procedural modeling parameters,
not measured habitability, a scientific alien-life occurrence rate or approved
biological asset appearance. Observed sources gain neither proven alien life nor
proven sterility. No liquid/hydrology, fauna, food safety, survival or physics
contract changes occur. Publication/catalog persistence is implemented here;
organism/depletion runtime save owners remain #620/#621. No renderer, particle,
audio mixer or collider changes occur, so graphical captures are not applicable
to these metadata contracts. Subjective/physical/hardware qualification remains
separately unverified in #561/#557/#563, with no M1/Metal performance claim.
