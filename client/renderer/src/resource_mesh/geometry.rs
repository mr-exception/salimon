//! Validated immutable GLB triangles and stable indexed vertex packing; no GPU state.
use super::{FLOATS, RendererError};

pub(super) fn geometry(bytes: &[u8]) -> Result<Vec<[f32; FLOATS]>, RendererError> {
    let fail = |message: &str| RendererError::new("load resource mesh", message);
    let asset =
        gltf::Gltf::from_slice(bytes).map_err(|e| RendererError::new("load resource mesh", e))?;
    let blob = asset
        .blob
        .as_deref()
        .ok_or_else(|| fail("missing binary chunk"))?;
    let mut vertices = Vec::new();
    for node in asset.nodes() {
        if node.transform().decomposed() != ([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]) {
            return Err(fail("expected baked identity node transforms"));
        }
        let Some(mesh) = node.mesh() else { continue };
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err(fail("expected triangles"));
            }
            let reader = primitive.reader(|b| (b.index() == 0).then_some(blob));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or_else(|| fail("missing positions"))?
                .collect();
            let normals: Vec<_> = reader
                .read_normals()
                .ok_or_else(|| fail("missing normals"))?
                .collect();
            let indices: Vec<u32> = reader.read_indices().map_or_else(
                || (0..positions.len() as u32).collect(),
                |v| v.into_u32().collect(),
            );
            let material = primitive.material();
            if material.alpha_mode() != gltf::material::AlphaMode::Opaque {
                return Err(fail("expected opaque material"));
            }
            let color = material.pbr_metallic_roughness().base_color_factor();
            for index in indices {
                let p = positions
                    .get(index as usize)
                    .ok_or_else(|| fail("invalid position index"))?;
                let n = normals
                    .get(index as usize)
                    .ok_or_else(|| fail("invalid normal index"))?;
                if p.iter().any(|v| !v.is_finite() || v.abs() > 0.48)
                    || n.iter().any(|v| !v.is_finite())
                {
                    return Err(fail("geometry outside centered unit cube"));
                }
                // A cheap fixed local fill emphasizes the authored planar facets.
                let shade = 0.65 + 0.35 * (n[0] * 0.3 + n[1] * 0.8 + n[2] * 0.5).clamp(0.0, 1.0);
                vertices.push([
                    p[0],
                    p[1],
                    p[2],
                    color[0] * shade,
                    color[1] * shade,
                    color[2] * shade,
                    1.0,
                ]);
            }
        }
    }
    if vertices.is_empty() {
        return Err(fail("empty geometry"));
    }
    Ok(vertices)
}

// Deduplicate position/material vertices once, preserving triangle order and
// facet color boundaries. Indices address the concatenated immutable stream.
pub(super) fn indexed_geometry(
    meshes: &[Vec<[f32; FLOATS]>; 19],
) -> (Vec<f32>, Vec<u32>, [std::ops::Range<u32>; 19]) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let ranges = std::array::from_fn(|mesh| {
        let start = indices.len() as u32;
        let mut unique = std::collections::BTreeMap::new();
        for vertex in &meshes[mesh] {
            let key = vertex.map(f32::to_bits);
            let index = *unique.entry(key).or_insert_with(|| {
                let index = (vertices.len() / FLOATS) as u32;
                vertices.extend_from_slice(vertex);
                index
            });
            indices.push(index);
        }
        start..indices.len() as u32
    });
    (vertices, indices, ranges)
}
