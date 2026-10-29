# #620 — canonical physical organisms, roots and safe clearance

Parent: non-draft [PR #696](https://github.com/mr-exception/salimon/pull/696),
`720515d8a1dd7062fa202093835c9655a37eba69`; this child targets
`issue-619-organism-families`. #425/#611 are merged and #619 is usable in ancestry.
This is the fifth PR in the requested linear batch; merge the stack oldest first.

[Maintained behavior/source/test contract](../../docs/physical-organisms.md).
World publishes a fixed terrain/ecology/grammar-bound population with stable
site/object/part identities, exact source endpoints and conservative part bounds.
Roots require support; unsafe slopes, unsupported roots, inter-object overlaps
and named opening/ramp/player/resource/pad exclusions reject. Changing source or
exclusions invalidates the strict baseline binding instead of silently moving
objects. Cosmetic density/LOD never controls physical population or resources.
Vesper's explicit sterile publication, terrain and original resources remain intact.

The common bounded f64 sweep belongs to `salimon-physics`; character and ship gain
only typed dependencies on that kernel. Character surface/EVA substeps sweep the
full body; ship sweeps a conservative full-hull circumsphere, including installed
construction solids. Landing/spawn checks reject blocked sites. Contact safely
stops motion/engines/assist without refunding paid Core work. Unsafe restored
initial overlap holds pose/energy and retains source geometry instead of panicking,
removing a tree or inventing a clear landing. Collider residency is installed
before movement and stays independent of camera/render residency. Renderer uses
exactly the same source specimen anchors, bases, part endpoints and radii.

No third-party dependency versions changed: Cargo.lock adds only the two local
physics dependency edges. Existing authored assets remain unchanged. All six
affected owning-guide groups and root architecture/maintenance/validation/source/
save guides are synchronized in this PR.

## Actual validation

Cloud Debian 13 x86_64; Rust/Cargo 1.99.1, Python 3.12, Mesa 25.0.7 llvmpipe LLVM
19.1.7 Vulkan. Workspace tools env; dev/test debug=0, assertions enabled.

- `cargo test --workspace --locked`: **1,187 passed, zero failures, 45 opt-in
  ignored** at the broad integration revision. This includes existing gameplay,
  ship assists/energy, physics, saves, construction, terrain and source tests.
  Subsequent safety-hold and evidence-fixture refinements received the final
  focused checks below; the retained broad log is not relabeled as a new run.
- Final `cargo test --workspace --locked organism`: **13 passed**, 3 opt-in
  ignored; `cargo test --workspace --locked static_`: **4 passed**.
  World placement/exclusion/binding and physics high-speed/retreat/translated-frame
  tests pass; unsafe-overlap runtime checks preserve pose/energy/source geometry.
- `SALIMON_PHYSICAL_ORGANISMS_FILE=artifacts/issue-620-physical.json cargo test -p
  salimon-client --locked physical_population -- --include-ignored --nocapture`:
  **2 passed**. Controlled fixture seed **18**, population
  `4a608693bc571f26cdc8dbeebd252303987a167452498d1df205957d0154a7e3`, four physical
  organisms. Both live controllers hit the same giant source object
  `55f0943f84a982c3d9d28961194b00fd2b21273197937b07cbe1aef3a7afc289`:
  character stops 3.564 m before its tangent origin, ship 16.595 m before it,
  with ship velocity zero and engines off. The larger ship clearance is the
  deliberate conservative authored-hull circumsphere, not exact convex contact.
- Passing that exact physical source export as `SALIMON_ORGANISMS_FILE`, with
  `SALIMON_ORGANISM_DIR=artifacts/issue-620-final` and workspace Vulkan driver,
  `cargo test -p salimon-renderer --locked organism_production_reference -- --ignored --nocapture`:
  **passed**, four original source-anchored/oriented production images, with
  unchanged part counts at unresolved LOD. Synchronous submit/readback samples
  123–155 ms include readback; they are not hardware FPS/M1/Metal qualification.
- Final workspace build/all-target Clippy `-D warnings`, rustfmt, whitespace,
  docs `--base issue-619-organism-families`: pass. Tooling **134 passed**.
- Native `universe-journey.json` under Xvfb/Mesa on the rebuilt production client:
  **25/25 passed**, 25.371 s, retaining startup/restore/walking/ramp/boarding.

Early fixture checks incorrectly assumed a 0.3 m character radius rather than
using the actual shared constant; corrected to assert against that constant.
The first positive population had no rare giant, so it could not establish the
explicit giant-clearance criterion. The controlled fixture now searches a bounded
fixed set of fixture seeds and selects seed 18; production rarity/placement rules
were not changed. Failed logs remain alongside passing final evidence.

Originals: [physical giant](physical-0-Giant.png), [column](physical-1-Column.png),
[frond](physical-2-Frond.png), [second column](physical-3-Column.png),
[native Vesper](vesper-start.png). [Media hashes](media-sha256.txt),
[exact source geometry/colliders (gzip)](physical-source.json.gz),
[baseline binding](population.binding), [source hashes](source-sha256.txt),
[native summary](native-summary.json).

## Limits retained explicitly

Colliders are simplified conservative per-part boxes; ship clearance intentionally
uses its full-hull circumsphere. This may stop earlier than exact visible contact.
The physical fixture is declared fictional engineering data, not a scientific
Moon-life claim or newly published playable world. Current Vesper is sterile;
full moving/orbiting pack/body-frame adoption remains #427. Baseline geometry is
immutable and reconstructed; mutable harvested/removable part state and conserved
resource/mass allocation remain #621. There is no new fauna, soft-body physics,
terrain destruction, regrowth, cave support or new combat/damage convention.
Originals stay candidates for [#561](https://github.com/mr-exception/salimon/issues/561);
actual physical platform #557 and hardware/M1/Metal budgets #563 remain unverified.
