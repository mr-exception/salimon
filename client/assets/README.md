# Asset registry

Run `npm run assets:validate` from `client/`. The version 1 registry maps stable
art IDs and integer revisions to ordered parts, layers and state part lists.
Pivots and authored anchors are normalized within the unpadded content. Domain
anchor and footprint references are checked against the Scout layout. PNG
content rectangles use pixel coordinates and cover the union of nonzero alpha.
Outer transparent pixels are excluded when comparing dimensions with source
density. Icon SVGs carry no world footprint.

Placeholder, draft, approved, production and deprecated are art lifecycle
states. Every entry records origin, license, attribution, redistribution rights
and reference provenance. Existing procedural art remains the current renderer
input. A failed decorative load resolves to a placeholder outcome. An essential
failure returns an explicit retry outcome with the original failure; the
caller can keep the last usable view and retry the load. IDs and revisions
describe presentation. The save schema continues to store gameplay IDs.

## Runtime package

`npm run build` emits `dist/assets/art/manifest.json` beside SHA-256-named
lossless WebP atlas pages and standalone SVG icons. Use `npm run assets:build
-- /path/to/output` to generate the package alone. The encoder uses the
lockfile's Sharp/libwebp build and the pipeline runs offline after dependencies
are installed. Atlas pages group assets by category, strip outer transparent
padding, preserve placement offsets and extrude one pixel with a one-pixel
transparent gutter. The default upper bound on either atlas dimension is 2,048
pixels including gutters. Each runtime part records its page URL (relative to
the manifest), content rectangle and original offset. Draft and deprecated
entries stay in authoring. Runtime entries need recorded redistribution rights,
and icons use a restricted standalone SVG subset.

The command reports package file bytes and an RGBA8 decoded-size estimate
without mipmaps, framebuffers, allocator overhead, or the JSON manifest.
Output under `dist/` contains only runtime files. Editable exports stay under
`art/sources/`.
