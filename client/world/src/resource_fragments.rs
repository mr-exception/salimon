//! Conservative physical mining output, with no pending mass or abstract inventory.
use crate::resources::{
    FragmentId, RawMaterial, ResourceDeposit, ResourceFragment, ResourceTransform,
};
use crate::{CELESTIAL_BODIES, WorldPosition};

pub const FRAGMENT_MAX_MASS_KG: f64 = 2.0;
/// A residual of a completed fill is left in the source, never debited or hidden.
/// Initial small requests remain valid; this is not a minimum fragment mass.
pub const FILL_ROUNDING_REMAINDER_KG: f64 = 1e-12;

/// A growing physical piece receives extraction until full, then a new piece is created.
/// This avoids producing a tiny object every simulation frame. No extracted mass is hidden.
#[derive(Default)]
pub(crate) struct FragmentOutput {
    fragments: Vec<ResourceFragment>,
    changes: std::collections::BTreeMap<u64, Option<ResourceFragment>>,
    next_id: u64,
    sealed: std::collections::HashSet<FragmentId>,
}

impl FragmentOutput {
    pub(crate) fn fragments(&self) -> &[ResourceFragment] {
        &self.fragments
    }

    pub(crate) fn get_mut(&mut self, id: FragmentId) -> Option<&mut ResourceFragment> {
        self.fragments.iter_mut().find(|piece| piece.id() == id)
    }

    pub(crate) fn record(&mut self, piece: ResourceFragment) {
        self.changes.insert(piece.id().0, Some(piece));
    }

    pub(crate) fn drain_changes(
        &mut self,
    ) -> impl Iterator<Item = (FragmentId, Option<ResourceFragment>)> {
        std::mem::take(&mut self.changes)
            .into_iter()
            .map(|(id, p)| (FragmentId(id), p))
    }

    pub(crate) fn remove(&mut self, id: FragmentId) -> bool {
        let Some(index) = self.fragments.iter().position(|p| p.id() == id) else {
            return false;
        };
        self.fragments.remove(index);
        self.sealed.remove(&id);
        self.changes.insert(id.0, None);
        true
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
            // Repeated f64 additions can leave a microscopic residual after filling
            // a full piece. Creating it would impose micrometre solver substeps.
            // Return only mass actually emitted; the source retains the residual.
            if mass < requested && mass <= FILL_ROUNDING_REMAINDER_KG {
                break;
            }
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
                self.changes
                    .insert(piece.id().0, Some(self.fragments[index]));
            } else {
                let id = FragmentId(self.next_id + 1);
                let Some(pose) = spawn(id, material, &self.fragments) else {
                    break;
                };
                let fragment = ResourceFragment::new(id, deposit.id(), material, pose);
                self.fragments.push(fragment);
                self.next_id = id.0;
                self.changes.insert(id.0, Some(fragment));
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

#[cfg(test)]
mod mutation_tests {
    use super::*;
    use crate::{
        CelestialBodyId,
        mining::MiningSession,
        resources::{DepositId, ResourceId},
    };
    use std::time::Duration;
    #[test]
    fn growth_is_coalesced_and_consumed_identity_is_never_reused() {
        let mut session = MiningSession::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 42,
            },
            RawMaterial::new(ResourceId::IronOre, 10.0).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            10.0,
        )
        .unwrap();
        session.extract(&mut deposit, Duration::from_millis(100));
        session.extract(&mut deposit, Duration::from_millis(100));
        let changes: Vec<_> = session.drain_fragment_changes().collect();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].1.unwrap().material().mass_kg(), 0.4);
        let id = changes[0].0;
        session.pick_up(id);
        assert!(session.remove_fragment(id));
        assert_eq!(session.carried_id(), None);
        assert_eq!(
            session.drain_fragment_changes().collect::<Vec<_>>(),
            vec![(id, None)]
        );
        session.extract(&mut deposit, Duration::from_secs(1));
        assert_ne!(session.fragments()[0].id(), id);
        assert_eq!(session.drain_fragment_changes().count(), 1);
        let _ = session.fragments();
        assert_eq!(session.drain_fragment_changes().count(), 0);
    }
}
