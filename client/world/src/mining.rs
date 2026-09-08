//! Portable aimed extraction. Runtime supplies proximity candidates and obstruction distance.
use std::time::Duration;

#[path = "biological_session.rs"]
mod biological_session;
#[path = "mining_save.rs"]
mod save;
#[path = "sphere_session.rs"]
mod sphere_session;
pub use biological_session::HarvestError;
pub use save::MiningSaveState;

use crate::resource_generation::SurfaceDeposit;
use crate::resources::{DepositId, DepositState, ResourceDeposit, ResourceFragment};
use crate::{BodyRole, WorldPosition};

pub const MINING_RANGE_METERS: f64 = 4.0;
pub const MINING_RATE_KG_PER_SECOND: f64 = 2.0;

#[derive(Clone, Copy, Debug)]
pub struct MiningRay {
    pub origin: WorldPosition,
    pub direction: [f64; 3],
}

impl MiningRay {
    pub fn new(origin: WorldPosition, target: WorldPosition) -> Option<Self> {
        let direction = target.offset_from(origin);
        let length = dot(direction, direction).sqrt();
        (origin.is_finite() && length.is_finite() && length > 0.0).then(|| Self {
            origin,
            direction: direction.map(|v| v / length),
        })
    }

    /// First forward intersection, including rays beginning within the bound.
    pub fn sphere_distance(self, center: WorldPosition, radius: f64) -> Option<f64> {
        let offset = center.offset_from(self.origin);
        let along = dot(offset, self.direction);
        // Perpendicular-distance form avoids subtracting two planetary-scale squares.
        let perpendicular = std::array::from_fn(|i| offset[i] - along * self.direction[i]);
        let discriminant = radius * radius - dot(perpendicular, perpendicular);
        if discriminant < 0.0 {
            return None;
        }
        let half_chord = discriminant.sqrt();
        if along + half_chord < 0.0 {
            None
        } else {
            Some((along - half_chord).max(0.0))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiningTarget {
    pub id: DepositId,
    pub distance_meters: f64,
}

pub fn aimed_deposit(
    ray: MiningRay,
    deposits: &[SurfaceDeposit],
    obstruction_distance: Option<f64>,
) -> Option<MiningTarget> {
    aimed_deposit_with_terrain(
        ray,
        deposits,
        obstruction_distance,
        crate::terrain::TerrainMode::Flat,
    )
}
pub fn aimed_deposit_with_terrain(
    ray: MiningRay,
    deposits: &[SurfaceDeposit],
    obstruction_distance: Option<f64>,
    terrain: crate::terrain::TerrainMode,
) -> Option<MiningTarget> {
    let planet_distance = terrain_obstruction(ray, terrain, MINING_RANGE_METERS);
    let blocked_at = planet_distance
        .into_iter()
        .chain(obstruction_distance)
        .min_by(f64::total_cmp)
        .unwrap_or(f64::INFINITY);
    deposits
        .iter()
        .filter(|entry| entry.deposit.state() != DepositState::Depleted)
        .filter_map(|entry| {
            let distance =
                ray.sphere_distance(entry.deposit.position(), entry.bounds_radius_meters)?;
            (distance <= MINING_RANGE_METERS && distance < blocked_at).then_some(MiningTarget {
                id: entry.deposit.id(),
                distance_meters: distance,
            })
        })
        .min_by(|a, b| {
            a.distance_meters
                .total_cmp(&b.distance_meters)
                .then_with(|| a.id.local.cmp(&b.id.local))
        })
}

/// Returns the actual mass difference, preserving conservation despite floating-point rounding.
pub fn extract(deposit: &mut ResourceDeposit, delta: Duration) -> f64 {
    let before = deposit.remaining_mass_kg();
    let removed =
        (MINING_RATE_KG_PER_SECOND * delta.as_secs_f64()).min(deposit.remaining_mass_kg());
    deposit
        .set_remaining_mass_kg(deposit.remaining_mass_kg() - removed)
        .expect("bounded extraction can only decrease finite remaining mass");
    before - deposit.remaining_mass_kg()
}

/// Sparse local-world journal keyed by stable deposit identity, separate from active lists.
/// Keep this session while streaming, stowing tools, or rebuilding presentation;
/// create a new session when starting a different world/seed configuration.
#[derive(Default)]
pub struct MiningSession {
    carry: crate::carrying::CarrySlot,
    spherical: crate::spherical_mining::SphericalMining,
    attached_sphere_id: Option<crate::resources::FragmentId>,
    deposit_revision: u64,
    remaining: std::collections::HashMap<DepositId, f64>,
    extracted_mass_kg: f64,
    fragments: crate::resource_fragments::FragmentOutput,
}

impl MiningSession {
    /// Changes only when deposit mass changes, not when fragments move or release.
    pub const fn deposit_revision(&self) -> u64 {
        self.deposit_revision
    }

    pub fn apply_to(&self, deposits: &mut [SurfaceDeposit]) {
        for entry in deposits {
            self.restore(&mut entry.deposit);
        }
    }

    fn restore(&self, deposit: &mut ResourceDeposit) {
        if let Some(mass) = self.remaining.get(&deposit.id()) {
            deposit
                .set_remaining_mass_kg((*mass).min(deposit.remaining_mass_kg()))
                .expect("restoration can only decrease validated remaining mass");
        }
    }

    pub fn extract(&mut self, deposit: &mut ResourceDeposit, delta: Duration) -> f64 {
        let pose = crate::resources::ResourceTransform::new(
            crate::resource_fragments::spawn_position(*deposit),
            [0.0, 0.0, 0.0, 1.0],
        )
        .expect("finite surface anchor");
        self.extract_with_spawn(deposit, delta, |_, _, _| Some(pose))
    }

    /// Resolve a new entity's pose before debiting its mass. Returning None pauses
    /// emission without consuming mass or identity. Growth never calls spawn.
    pub fn extract_with_spawn(
        &mut self,
        deposit: &mut ResourceDeposit,
        delta: Duration,
        spawn: impl FnMut(
            crate::resources::FragmentId,
            crate::resources::RawMaterial,
            &[ResourceFragment],
        ) -> Option<crate::resources::ResourceTransform>,
    ) -> f64 {
        // Freshly generated or stale active copies must reconcile before extraction,
        // so they cannot replenish the journal or emit already removed mass again.
        self.restore(deposit);
        let requested =
            (MINING_RATE_KG_PER_SECOND * delta.as_secs_f64()).min(deposit.remaining_mass_kg());
        let removed = self.fragments.emit(*deposit, requested, spawn);
        if removed > 0.0 {
            deposit
                .set_remaining_mass_kg(deposit.remaining_mass_kg() - removed)
                .expect("bounded output only decreases mass");
            self.remaining
                .insert(deposit.id(), deposit.remaining_mass_kg());
            self.deposit_revision += 1;
            self.extracted_mass_kg += removed;
        }
        removed
    }

    /// Shared one-object slot. External owners persist/validate their own fixture source.
    pub fn carried_object(&self) -> Option<crate::carrying::WorldObjectId> {
        self.carry.object()
    }
    pub fn carry_fixture(&mut self, id: u64) -> bool {
        self.carry
            .pick_up(crate::carrying::WorldObjectId::Fixture(id))
    }
    pub fn release_fixture(&mut self, id: u64) -> bool {
        if self.carry.object() != Some(crate::carrying::WorldObjectId::Fixture(id)) {
            return false;
        }
        self.carry.release();
        true
    }

    pub fn carried_id(&self) -> Option<crate::resources::FragmentId> {
        match self.carry.object() {
            Some(crate::carrying::WorldObjectId::ResourceFragment(id)) => Some(id),
            _ => None,
        }
    }

    pub fn pick_up(&mut self, id: crate::resources::FragmentId) -> bool {
        if self.fragments.get(id).is_some_and(|f| {
            f.natural_shape().is_some()
                && (f.contact_radius_meters()
                    > crate::environment::harvest::MAX_CARRY_RADIUS_METERS
                    || f.material().mass_kg() > crate::environment::harvest::MAX_CARRY_MASS_KG)
        }) || self.attached_sphere_id == Some(id)
            || self.fragments.get(id).is_none()
            || !self
                .carry
                .pick_up(crate::carrying::WorldObjectId::ResourceFragment(id))
        {
            return false;
        }
        // Extraction must never grow a piece after the player has collected it.
        self.fragments.seal(id);
        let piece = *self
            .fragments
            .get_mut(id)
            .expect("picked up fragment exists");
        self.fragments.record(piece);
        true
    }

    pub fn move_carried(&mut self, pose: crate::resources::ResourceTransform) {
        if let Some(id) = self.carried_id() {
            let piece = self.fragments.get_mut(id).expect("carried identity exists");
            if piece.transform() == pose {
                return;
            }
            piece.set_transform(pose);
            let changed = *piece;
            self.fragments.record(changed);
        }
    }

    pub fn drop_carried(&mut self, pose: crate::resources::ResourceTransform) -> bool {
        if self.carried_id().is_none() {
            return false;
        }
        self.move_carried(pose);
        let id = self.carried_id().expect("checked carried identity");
        self.carry.release();
        let piece = *self.fragments.get(id).expect("carried identity exists");
        self.fragments.record(piece);
        true
    }

    /// Move an existing loose physical entity with its supporting reference frame.
    /// Carried objects are controlled exclusively by the carry slot.
    pub fn move_loose(
        &mut self,
        id: crate::resources::FragmentId,
        pose: crate::resources::ResourceTransform,
    ) -> bool {
        if self.carried_id() == Some(id) || self.attached_sphere_id == Some(id) {
            return false;
        }
        let Some(piece) = self.fragments.get_mut(id) else {
            return false;
        };
        if piece.transform() == pose {
            return true;
        }
        piece.set_transform(pose);
        let changed = *piece;
        self.fragments.record(changed);
        true
    }

    /// Coalesced physical mutations, ordered by stable ID; reads/streaming emit nothing.
    pub fn drain_fragment_changes(
        &mut self,
    ) -> impl Iterator<Item = (crate::resources::FragmentId, Option<ResourceFragment>)> {
        self.fragments.drain_changes()
    }

    /// Reclaim one original reserved receipt into the existing single carry slot.
    /// Reservation owners validate receipts against their source before this boundary.
    pub fn return_reserved_fragment(&mut self, fragment: ResourceFragment) -> bool {
        if self.carried_object().is_some()
            || self.attached_sphere_id.is_some()
            || !crate::resources::RawMaterial::new(
                fragment.material().resource(),
                fragment.material().mass_kg(),
            )
            .is_ok_and(|m| m.volume_m3() == fragment.material().volume_m3())
            || crate::resources::ResourceTransform::new(
                fragment.transform().position(),
                fragment.transform().orientation_xyzw(),
            )
            .is_err()
            || !self.fragments.return_reserved(fragment)
        {
            return false;
        }
        assert!(
            self.pick_up(fragment.id()),
            "reserved return passed single-slot admission"
        );
        true
    }

    /// Atomically return a validated reservation batch as loose physical pieces.
    /// The caller owns receipt provenance and collision-free destination admission.
    pub fn return_reserved_loose(&mut self, pieces: &[ResourceFragment]) -> bool {
        let mut ids = std::collections::HashSet::new();
        let next = self.save_state().next_id;
        if pieces.iter().any(|f| {
            f.id().0 == 0
                || f.id().0 > next
                || !ids.insert(f.id())
                || self.fragment(f.id()).is_some()
                || !crate::resources::RawMaterial::new(
                    f.material().resource(),
                    f.material().mass_kg(),
                )
                .is_ok_and(|m| m == f.material())
                || crate::resources::ResourceTransform::new(
                    f.transform().position(),
                    f.transform().orientation_xyzw(),
                )
                .is_err()
        }) {
            return false;
        }
        for f in pieces {
            assert!(
                self.fragments.return_reserved(*f),
                "batch preflight admitted receipt"
            );
        }
        true
    }

    /// Transfer externally owned salvage into sealed physical objects. The caller
    /// must remove the source mass atomically; extraction totals do not increase.
    pub fn import_salvage(
        &mut self,
        pieces: &[(
            crate::resources::DepositId,
            crate::resources::RawMaterial,
            crate::resources::ResourceTransform,
        )],
    ) -> Option<Vec<crate::resources::FragmentId>> {
        self.fragments.import_salvage(pieces)
    }

    /// Physical despawn/Core consumption; never reuse an entity identity.
    pub fn remove_fragment(&mut self, id: crate::resources::FragmentId) -> bool {
        if self.carried_id() == Some(id) {
            self.carry.release();
        }
        if self.attached_sphere_id == Some(id) {
            self.interrupt_spherical();
        }
        self.fragments.remove(id)
    }

    /// Diagnostic extraction total, never spendable inventory or carried mass.
    pub const fn extracted_mass_kg(&self) -> f64 {
        self.extracted_mass_kg
    }

    /// Stable-ID lookup; ordered iteration remains available through `fragments`.
    pub fn fragment(&self, id: crate::resources::FragmentId) -> Option<&ResourceFragment> {
        self.fragments.get(id)
    }

    /// Physical session entities, independent of active deposit streaming and tool equipment.
    pub fn fragments(&self) -> &[ResourceFragment] {
        self.fragments.fragments()
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

/// Bounded short interaction ray. Flat mode preserves the analytic legacy query.
/// Terrain uses 5 cm samples and a 20-step root bracket, including below nominal radius.
pub fn terrain_obstruction(
    ray: MiningRay,
    terrain: crate::terrain::TerrainMode,
    range: f64,
) -> Option<f64> {
    terrain
        .bodies()
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .filter_map(|body| {
            if terrain == crate::terrain::TerrainMode::Flat || terrain.definition(body.id).is_none()
            {
                return ray.sphere_distance(body.center, body.radius_meters);
            }
            if terrain.clearance(*body, ray.origin.meters()).abs() > range + 1.0 {
                return None;
            }
            let point = |distance| {
                ray.origin
                    .translated(ray.direction.map(|v| v * distance))
                    .meters()
            };
            let mut previous = 0.0;
            for i in 0..=80 {
                let next = range * f64::from(i) / 80.0;
                if terrain.clearance(*body, point(next)) <= 0.0 {
                    let mut low = previous;
                    let mut high = next;
                    for _ in 0..20 {
                        let mid = (low + high) * 0.5;
                        if terrain.clearance(*body, point(mid)) > 0.0 {
                            low = mid;
                        } else {
                            high = mid;
                        }
                    }
                    return Some(high);
                }
                previous = next;
            }
            None
        })
        .min_by(f64::total_cmp)
}
