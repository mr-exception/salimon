# Internal Hall spatial prototype (#472)

Adds an explicitly gated `hall-prototype` E2E scene with concentric monument rings,
a symbolic primordial presence and central gate. World owns exact slot geometry,
bounded near-arc queries and conservative capsule sweeps; runtime owns ephemeral
camera/instance mapping through the existing production wgpu pipeline. Normal
startup, menus, saves and account/unlock state have no Hall access.

The synthetic fixture represents one billion empty monument slots without a
registry allocation. Capacity is 40/60/80/...; exact integer boundaries survive
large IDs. A 128 m spatial window retains at most 1,024 signs, with a three-piece
metre-scale authored primitive kit nearby and one instance per distant silhouette.
The floor has no exterior wall; gate/core and monument envelopes stop the planar
visitor. This is an internal spatial/controller prototype, not completion or
backend infrastructure. #473–#475 remain blocked on actual account/authoritative
completion infrastructure; no local registry fallback is introduced.

Three world tests and the runtime adapter test pass, including default-startup
and explicit-E2E admission assertions. The production capture matrix passes
**29/29 steps** with six original captures and byte-identical arrival/revisit
pixels and source. [Native evidence](native-evidence.json) retains exact context,
source hashes, spatial counts and raw frame samples.
Final gates are recorded in `verification.json`.

The first native attempt could not connect to X11: the prior display server had
exited while its socket remained. [Failed startup](startup-failed.json) is retained;
the passing run uses a live persistent Xvfb :100 session.
The driver retains six original views, exact source/binary hashes, actual
adapter/viewport and raw frame samples; it requires identical source/pixels on
revisit. Camera fixture repositioning is explicitly separate from swept walking.

```sh
cargo test -p salimon-world --locked hall::
cargo test -p salimon-client --locked hall_prototype
python3 scripts/verify_hall_prototype.py --binary target/debug/salimon-client --output artifacts/issue-472/native
```

See [the maintained contract](../../docs/hall-prototype.md) for controls, ownership,
geometry and exact boundaries. Human appearance #561, physical platform/input
#557 and hardware/M1 budgets #563 remain separately unverified. No final art or
visual baseline approval, inscription, ranking, unlock, merge or release is claimed.

Documentation impact: world/runtime architecture and invariants, narrative status,
maintenance, validation and runner guides map the implemented prototype and keep
future backend and human qualification distinct.

Original views: [arrival](step-005-hall-arrival.png), [rings](step-008-hall-overview.png),
[monolith](step-010-hall-monolith.png), [collision](step-022-hall-collision.png),
[outer population](step-027-hall-billion-outer.png), [revisit](step-029-hall-revisit.png).
