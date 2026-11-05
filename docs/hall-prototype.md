# Internal Hall spatial prototype (#472)

> **Corpus migration status (#791): Historical native implementation.** The current [React overhead 2D world](current-client-status.md) uses a separate **cockpit-only 3D radar**. Any older Rust, wgpu, 3D assets, first-person view or issue-delivery statements below concern the [archived native tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) rather than shipped browser features. Keep historical mechanics and balance evidence as input; reimplement in the browser with integer-gram inventory and its own versioned saves. The approved 2D hazard health maximum is 200 HP, not a migrated legacy health implementation. See the [migration register](2d-migration-register.md) for readiness and ownership.


The explicit `--e2e --scenario hall-prototype` fixture presents the approved
circular Hall through the production wgpu renderer. It is absent from normal
startup, menus, saves, achievements and unlock state. This backend-independent
prototype grants no journey completion, account access or inscription submission.
Those contracts remain #473–#475 and require actual backend infrastructure.

World `hall.rs` owns a deterministic synthetic layout, bounded spatial selection
and conservative horizontal capsule collision. Ring r has `40+20*(r-1)` slots,
32 m initial radius and 16 m radial separation. Zero-based slot boundaries use
integer arithmetic, including ranks beyond exact f64 integer representation.
The fixture supplies one billion synthetic empty signs; none represents an account.
Only arcs within 128 m are enumerated, with at most 1,024 resident signs. Objects
within 48 m use the authored three-piece metre-scale monolith kit (plinth, slab,
cap); farther signs use one silhouette instance. The existing production cuboid
triangle mesh supplies the kit, without a new asset export or GPU pipeline.

Runtime `hall_prototype.rs` maps that source into renderer-neutral instances.
A symbolic bright primordial presence and monumental gate share the center;
neither is a literal historical Big Bang coordinate, catalog body or fast travel.
The floor patch extends beyond the spatial window and follows the visitor; there
is no exterior wall. The infinite floor supplies fixed standing height. Capsule
movement sweeps the conservative monument plinth envelopes and gate supports;
the central presence has an explicit 10 m exclusion. This is a dedicated planar
prototype controller, not survival physics or the final ceremony controller.

Developer cameras `arrival`, `overview`, `sign` and `outer` are selected through
protocol `hall_view`. These explicitly reposition the fixture for inspection;
they are not traversal evidence. `hall_walk` takes a two-element x/z displacement
in metres, rejects nonfinite/greater-than-one-metre steps, and uses the same
collision owner as admitted W/A/S/D key taps (0.5 m per initial press). Movement
uses Hall axes; it has no autopilot, vertical flight or survival-save mutation.
The fixture is silent and hides gameplay overlays. No Hall entry or preview is
added to the ordinary user interface.

Run from the repository root with working Linux X11/software Vulkan:

```sh
cargo test -p salimon-world --locked hall::
cargo test -p salimon-client --locked hall_prototype
python3 scripts/verify_hall_prototype.py --binary target/debug/salimon-client --output artifacts/hall-prototype
```

The driver requires production GPU readbacks, actual context/raw frame evidence,
six original views, collision assertions and identical source/pixels after a
revisit. Run workspace build/tests, all-target Clippy, fmt, script and docs gates.
Software timings do not qualify physical GPU/M1 FPS. Human art review #561,
physical platforms #557 and hardware budgets #563 remain separate; no baseline
is promoted. See the [narrative contract](absenat-and-hall.md) and
[completion report](https://github.com/mr-exception/salimon/blob/8f112bb6e6be9f813174efb47d706a8601930d04/reports/issue-472/README.md).
