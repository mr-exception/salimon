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
    slots: std::collections::HashMap<FragmentId, usize>,
    changes: std::collections::BTreeMap<u64, Option<ResourceFragment>>,
    next_id: u64,
    sealed: std::collections::HashSet<FragmentId>,
}

impl FragmentOutput {
    /// Attached spherical output uses the same ID/journal storage, but is sealed
    /// against legacy extraction-tail growth immediately.
    pub(crate) fn upsert_sphere(
        &mut self,
        id: Option<FragmentId>,
        sphere: crate::spherical_mining::GrowingSphere,
        pose: ResourceTransform,
    ) -> FragmentId {
        let id = id.unwrap_or_else(|| {
            self.next_id = self
                .next_id
                .checked_add(1)
                .expect("session fragment ID space exhausted");
            FragmentId(self.next_id)
        });
        let piece = ResourceFragment::new(id, sphere.source, sphere.material, pose);
        if let Some(existing) = self.get_mut(id) {
            *existing = piece;
        } else {
            self.slots.insert(id, self.fragments.len());
            self.fragments.push(piece);
        }
        self.sealed.insert(id);
        self.record(piece);
        id
    }

    pub(crate) fn fragments(&self) -> &[ResourceFragment] {
        &self.fragments
    }

    pub(crate) fn get(&self, id: FragmentId) -> Option<&ResourceFragment> {
        self.slots.get(&id).map(|&slot| &self.fragments[slot])
    }

    pub(crate) fn get_mut(&mut self, id: FragmentId) -> Option<&mut ResourceFragment> {
        let slot = *self.slots.get(&id)?;
        Some(&mut self.fragments[slot])
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
        let Some(index) = self.slots.remove(&id) else {
            return false;
        };
        self.fragments.remove(index);
        // Removal is uncommon. Keep the canonical Vec order (never swap_remove)
        // and repair only shifted slots; hash iteration never controls simulation.
        for (slot, piece) in self.fragments.iter().enumerate().skip(index) {
            self.slots.insert(piece.id(), slot);
        }
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
                self.slots.insert(id, self.fragments.len());
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
    fn sphere(output: &mut FragmentOutput, mass: f64) -> FragmentId {
        let material = RawMaterial::new(ResourceId::IronOre, mass).unwrap();
        let id = output.upsert_sphere(None, crate::spherical_mining::GrowingSphere {
            source: DepositId { body: CelestialBodyId::Earth, local: 305 }, material,
        }, ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0]).unwrap());
        let piece = *output.get(id).unwrap();
        assert_eq!(piece.id(), id);
        assert_eq!(piece.material().mass_kg(), mass);
        let expected = (3.0 * mass / (4.0 * std::f64::consts::PI * 3500.0)).cbrt().clamp(0.005, 2.0);
        assert!((crate::resource_size::fragment_contact_radius_meters(material) - expected).abs() < 1e-14);
        id
    }

    fn assert_index(output: &FragmentOutput) {
        assert_eq!(output.slots.len(), output.fragments.len());
        for (slot, piece) in output.fragments.iter().enumerate() {
            assert_eq!(output.slots.get(&piece.id()), Some(&slot));
            assert_eq!(output.get(piece.id()), Some(piece));
        }
    }

    #[test]
    fn ordered_removal_growth_and_journal_keep_index_consistent() {
        let mut output = FragmentOutput::default();
        let ids: Vec<_> = (0..30).map(|i| sphere(&mut output, if i % 2 == 0 {10.0} else {0.000003})).collect();
        assert_index(&output);
        for id in [ids[0], ids[15], ids[29], ids[3]] {
            assert!(output.remove(id));
            assert!(!output.remove(id));
            assert!(output.get(id).is_none());
            assert_index(&output);
        }
        let survivors: Vec<_> = ids.iter().copied().filter(|id| output.get(*id).is_some()).collect();
        assert_eq!(output.fragments.iter().map(|p|p.id()).collect::<Vec<_>>(), survivors);
        let new = sphere(&mut output, 10.0);
        assert!(new.0 > ids.last().unwrap().0);
        let old = *output.get(survivors[1]).unwrap();
        output.upsert_sphere(Some(old.id()), crate::spherical_mining::GrowingSphere {
            source: old.source(), material: RawMaterial::new(ResourceId::IronOre, 2.0).unwrap(),
        }, old.transform());
        assert_index(&output);
        assert_eq!(output.get(old.id()).unwrap().material().mass_kg(), 2.0);
        let changes: Vec<_> = output.drain_changes().collect();
        assert!(changes.windows(2).all(|w| w[0].0.0 < w[1].0.0));
        for id in [ids[0], ids[15], ids[29], ids[3]] {
            assert_eq!(changes.iter().find(|(key,_)|*key == id).unwrap().1, None);
        }
    }

    #[test]
    #[ignore = "opt-in indexed vs historical linear lookup attribution, CPU only"]
    fn fragment_index_cost() {
        use std::{hint::black_box, time::Instant};
        for active in [500, 1000] {
            for inactive in [0, 10_000, 50_000] {
                let mut output = FragmentOutput::default();
                let ids: Vec<_> = (0..active + inactive).map(|_| sphere(&mut output, 10.0)).collect();
                assert_index(&output);
                let queries = &ids[inactive..];
                let capacity = output.slots.capacity();
                for sample in 0..20 {
                    let now = Instant::now();
                    for &id in queries {
                        black_box(output.fragments.iter_mut().find(|p| p.id() == black_box(id)).unwrap());
                    }
                    let linear_ns = now.elapsed().as_nanos();
                    let now = Instant::now();
                    for &id in queries { black_box(output.get_mut(black_box(id)).unwrap()); }
                    let indexed_ns = now.elapsed().as_nanos();
                    assert_eq!(output.slots.capacity(), capacity);
                    println!("INDEX_COST active={active} inactive={inactive} sample={sample} linear_ns={linear_ns} indexed_ns={indexed_ns} index_capacity={capacity}");
                }
            }
        }
    }

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
