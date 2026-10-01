//! Conservative physical mining output, with no pending mass or abstract inventory.
use crate::resources::{
    FragmentId, RawMaterial, ResourceDeposit, ResourceFragment, ResourceTransform,
};
use crate::{CELESTIAL_BODIES, WorldPosition};

pub const FRAGMENT_MAX_MASS_KG: f64 = 2.0;

/// A growing physical piece receives extraction until full, then a new piece is created.
/// This avoids producing a tiny object every simulation frame. No extracted mass is hidden.
#[derive(Default)]
pub(crate) struct FragmentOutput {
    fragments: Vec<ResourceFragment>,
}

impl FragmentOutput {
    pub(crate) fn fragments(&self) -> &[ResourceFragment] {
        &self.fragments
    }

    pub(crate) fn emit(&mut self, deposit: ResourceDeposit, mut mass: f64) {
        while mass > 0.0 {
            let tail = self
                .fragments
                .iter()
                .rposition(|piece| piece.source() == deposit.id());
            let growing = tail
                .filter(|index| self.fragments[*index].material().mass_kg() < FRAGMENT_MAX_MASS_KG);
            let previous = growing.map_or(0.0, |index| self.fragments[index].material().mass_kg());
            let added = mass.min(FRAGMENT_MAX_MASS_KG - previous);
            let material = RawMaterial::new(deposit.material().resource(), previous + added)
                .expect("positive extracted mass has valid material volume");
            if let Some(index) = growing {
                let piece = self.fragments[index];
                self.fragments[index] =
                    ResourceFragment::new(piece.id(), piece.source(), material, piece.transform());
            } else {
                let ordinal = self
                    .fragments
                    .iter()
                    .filter(|piece| piece.source() == deposit.id())
                    .count();
                let pose =
                    ResourceTransform::new(spawn_position(deposit, ordinal), [0.0, 0.0, 0.0, 1.0])
                        .expect("surface fragment anchor is finite");
                self.fragments.push(ResourceFragment::new(
                    FragmentId(self.fragments.len() as u64 + 1),
                    deposit.id(),
                    material,
                    pose,
                ));
            }
            mass -= added;
        }
    }
}

/// World-axis cube side, derived from physical solid volume rather than a presentation scale.
pub fn side_meters(fragment: ResourceFragment) -> f64 {
    fragment.material().volume_m3().cbrt()
}

fn spawn_position(deposit: ResourceDeposit, ordinal: usize) -> WorldPosition {
    let body = CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == deposit.id().body)
        .unwrap();
    let radial = deposit.position().offset_from(body.center);
    let length = radial.iter().map(|v| v * v).sum::<f64>().sqrt();
    // Contracts also allow synthetic test deposits away from a body surface.
    let up = if length > 0.0 {
        radial.map(|v| v / length)
    } else {
        [0.0, 1.0, 0.0]
    };
    let axis = if up[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = cross(up, axis);
    let norm = tangent.iter().map(|v| v * v).sum::<f64>().sqrt();
    let tangent = tangent.map(|v| v / norm);
    let bitangent = cross(up, tangent);
    // Reserve full-piece space so growing pieces never shift position or intersect the sphere.
    let side = (FRAGMENT_MAX_MASS_KG
        / deposit.material().resource().definition().density_kg_per_m3)
        .cbrt();
    let support = side * up.iter().map(|v| v.abs()).sum::<f64>() * 0.5 + 0.005;
    let column = 0.8 + (ordinal % 8) as f64 * 0.3;
    let row = (ordinal / 8) as f64 * 0.3;
    deposit.position().translated(std::array::from_fn(|i| {
        up[i] * support + tangent[i] * column + bitangent[i] * row
    }))
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
