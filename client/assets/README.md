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
