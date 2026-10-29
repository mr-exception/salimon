# Hybrid astronomical sky (#423)

The canonical gameplay view now draws an analytically authored Milky Way band
and the published host star. `renderer/src/sky.rs` and `sky.wgsl` own presentation;
`runtime/src/app/sky.rs` maps the fixed starting body's existing illumination
axes, published star radius and camera-relative f64 distances into angular disks.
The band is authored art, not an observed star census or navigation layer. The
historical compressed engineering fixtures retain their existing background.

Optional verified pack cells may contain a `sky` sidecar: a JSON array of
`salimon_universe::sky::StarSource` values. Each source must reference a star in
that cell's catalog and provide a split galactic position, explicit reference
Epoch, positive SI radius and optional finite velocity, temperature and
luminosity. Missing velocity holds the reference position; missing temperature
and luminosity use explicitly approximate solar presentation defaults. Unknown
radius/position cannot become an invented star. Duplicate IDs, invalid physical
values, over 128 sources or over 256 KiB reject. The streaming IO worker verifies
and decodes the sidecar before publishing the cell; the frame thread never reads
a file. Existing packs without this optional sidecar remain compatible.

The canonical streaming observer is the published system anchor plus current
body-local camera position. This follows the current fixed-axis playable adapter;
full rotating/orbiting frame adoption remains #427. Star motion uses elapsed
simulation time from each supplied reference epoch, not a fabricated UTC-to-TT
conversion or a claim of current-epoch astrometric precision. The existing host
illumination has no certified orbital phase. These are explicit visual model
limits; scientific flight/ephemeris APIs remain unchanged.

`sky::StarSource::project` subtracts split positions before normalization and
f32 narrowing, uses radius/distance angular size, and applies an approximate
inverse-square exposure and temperature color. Runtime deduplicates streamed
identities, ranks by apparent brightness with a stable identity tie-break, and
admits at most 127 plus the host; dim sources
below the exposure cutoff disappear. Quality controls affect this bounded
presentation only. At most two draws and a fixed 4 KiB star buffer are used;
there is no all-catalog per-frame scan. Subpixel sources use a small optical
point-spread footprint; resolved disks retain physical angular scale. There are
no enlarged body markers, names, chart writes, scanner returns or new discoveries.
The background uses reverse-Z depth zero without writing depth; subsequent
opaque scene geometry occludes sky and stars normally.

## Verification

Run workspace tests/build/Clippy/fmt, tooling and docs checks. The opt-in
`cargo test -p salimon-renderer --locked sky_production_reference -- --ignored --nocapture`
requires `SALIMON_SKY_DIR` and a working Vulkan adapter. It exports the actual
production shader at origin and a trillion-meter translation, asserts identical
pixels, checks a visible resolved star and verifies complete near-depth
occlusion. Malformed DTOs/counts reject. Universe projection tests cover split
origins, physical angular radius, motion and unknown-velocity holding. Report
originals and actual environment/timings; human appearance #561 and hardware/M1
budgets #563 remain separate unverified release qualification.
