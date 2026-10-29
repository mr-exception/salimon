# #619 — procedural low-poly organism families

Parent: non-draft [PR #695](https://github.com/mr-exception/salimon/pull/695),
`ec06d921955ede45ad687b732b2a0736a2df9b81`; this child targets
`issue-616-coherent-weather-visuals`. #618/#613/#611 are merged. Merge oldest first.

[Maintained organism contract](../../docs/planetary-organisms.md) maps immutable
world family/variant/part IDs and related bounded morphology/pigments; fixed
cube-face cosmetic surface candidates; runtime f64 surface-basis mapping; and
production instanced faceted-prism geometry. Mats, columns, fronds, filaments,
colonies and giant structural forms each have related variants. Cosmetic density
is a stable subset, never physical collision/resource authority. Developed
fictional biospheres alone admit macro organisms; Vesper remains published sterile.
Existing ecological macro-cover metadata remains the coarse coverage authority;
#622 owns enhanced biological material/animation response and #620 physical large
organism placement/roots/colliders. No canonical new world or fauna is published.

Near geometry is six-sided; the three-sided unresolved LOD switches below 0.75
projected radius pixels and retains every endpoint/part/root. No navigation-style
size enhancement. Fixed maximum 16,384 segments, two draws, two 768 KiB instance
buffers and 48 B/instance bound work. No existing authored asset changes or
external model services are used; source is reviewable parametric Rust geometry.
All affected world/runtime/renderer owning guides and root routing are updated.

## Actual validation

Cloud Debian 13 x86_64, Rust/Cargo 1.99.1, Python 3.12, Mesa 25.0.7 llvmpipe LLVM
19.1.7 Vulkan. Dev/test debug=0, assertions enabled, workspace tools environment.

- `cargo test --workspace --locked organism`: **6 passed**, zero failures;
  two evidence checks opt-in. Stable related forms/IDs/parts, bounded source
  geometry/palette, sterile/microbial exclusions, immutable definitions, exact
  cosmetic quality subsets, closed LOD geometry and runtime f64 mapping pass.
- `SALIMON_ORGANISMS_FILE=artifacts/issue-619-specimens.json cargo test -p
  salimon-client --locked organism_specimen_export -- --ignored --nocapture`:
  **passed**, actual fictional world grammar/runtime geometry, not hand-drawn
  renderer stand-ins or a scientific Moon-life claim.
- With that same file, workspace Vulkan driver and `SALIMON_ORGANISM_DIR`,
  `cargo test -p salimon-renderer --locked organism_production_reference -- --ignored --nocapture`:
  **passed**, twelve distinct original production images and identical source
  part counts at far LOD. Synchronous submit/readback samples 127–249 ms include
  warmup/readback and do not establish hardware FPS/M1/Metal performance.
- Workspace build, all-target Clippy `-D warnings`, formatting, whitespace and
  documentation/base-impact checks pass. Tooling **134 passed**.
- Native `planetary-vacuum.json` on the production binary under Xvfb/Mesa:
  **6/6 passed**, confirming unchanged sterile/airless Vesper startup.

An initial cosmetic fixture selected a sparse polar region and incorrectly
assumed occupied cover. The final controlled fixture uses its habitable equatorial
region; source admission/rarity is preserved. The failed log remains retained.

Originals: [mat](Mat-0.png), [related mat](Mat-1.png), [column](Column-0.png),
[related column](Column-1.png), [frond](Frond-0.png), [related frond](Frond-1.png),
[filaments](Filament-0.png), [related filaments](Filament-1.png),
[colony](Colony-0.png), [related colony](Colony-1.png), [giant](Giant-0.png),
[related giant](Giant-1.png), [sterile Vesper](vesper-sterile.png).
[Hashes](media-sha256.txt), [source specimens (gzip)](world-runtime-specimens.json.gz),
[decompressed hash](source-sha256.txt), [native result](native-summary.json).
Cameras are fitted separately at true generated scale to inspect each morphology.

Originals remain candidates for [#561](https://github.com/mr-exception/salimon/issues/561)
subjective review; physical platform #557 and actual hardware/M1/Metal budgets
#563 remain unverified. No automatic approved-baseline promotion, unsupported
ecological simulation, terrain destruction, collision or harvesting is claimed.
