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
    sealed: std::collections::HashSet<FragmentId>,
}

impl FragmentOutput {
    pub(crate) fn fragments(&self) -> &[ResourceFragment] {
        &self.fragments
    }

    pub(crate) fn get_mut(&mut self, id: FragmentId) -> Option<&mut ResourceFragment> {
        self.fragments.iter_mut().find(|piece| piece.id() == id)
    }

    pub(crate) fn seal(&mut self, id: FragmentId) {
        self.sealed.insert(id);
    }

    pub(crate) fn emit(
        &mut self,
        deposit: ResourceDeposit,
        mut mass: f64,
        mut spawn: impl FnMut(FragmentId, RawMaterial, &[ResourceFragment]) -> Option<ResourceTransform>,
    ) -> f64 {
        let requested = mass;
        while mass > 0.0 {
            let tail = self.fragments.iter().rposition(|piece| {
                piece.source() == deposit.id() && !self.sealed.contains(&piece.id())
            });
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
                let id = FragmentId(self.fragments.len() as u64 + 1);
                let Some(pose) = spawn(id, material, &self.fragments) else {
                    break;
                };
                self.fragments
                    .push(ResourceFragment::new(id, deposit.id(), material, pose));
            }
            mass -= added;
        }
        requested - mass
    }
}

/// Authoritative gameplay cube side; solid material volume remains density-derived.
pub fn side_meters(fragment: ResourceFragment) -> f64 {
    crate::resource_size::fragment_side_meters(fragment.material())
}

pub(crate) fn spawn_position(deposit: ResourceDeposit) -> WorldPosition {
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
    // Geometry-free callers use the top of the conservative deposit bound.
    // Native mining supplies the authored surface via extract_with_spawn.
    let full = RawMaterial::new(deposit.material().resource(), FRAGMENT_MAX_MASS_KG)
        .expect("full fragment material is valid");
    let support = crate::resource_size::fragment_contact_radius_meters(full) + 0.005;
    let edge = crate::resource_size::deposit_radius_meters(deposit.material());
    deposit
        .position()
        .translated(up.map(|v| v * (edge + support)))
}
