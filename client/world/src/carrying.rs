//! Permanent one-world-object carrying rule. Equipped gear uses separate systems.
use crate::mining::MiningRay;
use crate::resources::{FragmentId, ResourceFragment};

pub const PICKUP_RANGE_METERS: f64 = 3.0;

/// An identity in the physical world, never an abstract inventory quantity.
/// Future carriable object kinds must share this same slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldObjectId {
    ResourceFragment(FragmentId),
    Fixture(u64),
}

#[derive(Default)]
pub struct CarrySlot {
    object: Option<WorldObjectId>,
}

impl CarrySlot {
    pub const fn object(&self) -> Option<WorldObjectId> {
        self.object
    }

    /// There is deliberately no capacity setting or progression override.
    pub fn pick_up(&mut self, object: WorldObjectId) -> bool {
        if self.object.is_some() {
            return false;
        }
        self.object = Some(object);
        true
    }

    pub fn release(&mut self) -> Option<WorldObjectId> {
        self.object.take()
    }
}

/// Target the mass/density sphere, excluding the carried object. Hull and terrain
/// occlusion are supplied by composition; equal-distance ties use stable identity.
pub fn aimed_fragment(
    ray: MiningRay,
    fragments: &[ResourceFragment],
    carried: Option<WorldObjectId>,
    blocked_at: f64,
) -> Option<FragmentId> {
    fragments
        .iter()
        .filter_map(|piece| {
            if carried == Some(WorldObjectId::ResourceFragment(piece.id())) {
                return None;
            }
            let distance = ray.sphere_distance(
                piece.transform().position(),
                crate::resource_size::fragment_contact_radius_meters(piece.material()),
            )?;
            (distance <= PICKUP_RANGE_METERS && distance < blocked_at)
                .then_some((piece.id(), distance))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.0.cmp(&b.0.0)))
        .map(|target| target.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::{DepositId, RawMaterial, ResourceId, ResourceTransform};
    use crate::{CelestialBodyId, WorldPosition};

    fn piece(id: u64, z: f64) -> ResourceFragment {
        ResourceFragment::new(
            FragmentId(id),
            DepositId {
                body: CelestialBodyId::Earth,
                local: 0,
            },
            RawMaterial::new(ResourceId::SilicateRock, 2.0).unwrap(),
            ResourceTransform::new(WorldPosition::new(0.0, 0.0, z), [0.0, 0.0, 0.0, 1.0]).unwrap(),
        )
    }

    #[test]
    fn sphere_pickup_rejects_old_cube_corners_and_ignores_rotation() {
        let fragment = piece(1, 1.0);
        let radius = crate::resource_size::fragment_contact_radius_meters(fragment.material());
        let miss = MiningRay::new(
            WorldPosition::new(radius * 0.9, radius * 0.9, 0.0),
            WorldPosition::new(radius * 0.9, radius * 0.9, 2.0),
        )
        .unwrap();
        assert_eq!(aimed_fragment(miss, &[fragment], None, f64::INFINITY), None);
        let hit = MiningRay::new(
            WorldPosition::new(radius * 0.5, 0.0, 0.0),
            WorldPosition::new(radius * 0.5, 0.0, 2.0),
        )
        .unwrap();
        let mut rotated = fragment;
        rotated.set_transform(
            ResourceTransform::new(
                fragment.transform().position(),
                [0.0, 0.0, 0.7_f64.sin(), 0.7_f64.cos()],
            )
            .unwrap(),
        );
        assert_eq!(
            aimed_fragment(hit, &[fragment], None, f64::INFINITY),
            Some(fragment.id())
        );
        assert_eq!(
            aimed_fragment(hit, &[rotated], None, f64::INFINITY),
            Some(fragment.id())
        );
    }

    #[test]
    fn permanent_single_slot_rejects_replacement_until_release() {
        let mut slot = CarrySlot::default();
        let first = WorldObjectId::ResourceFragment(FragmentId(1));
        assert!(slot.pick_up(first));
        assert!(!slot.pick_up(WorldObjectId::ResourceFragment(FragmentId(2))));
        assert_eq!(slot.object(), Some(first));
        assert_eq!(slot.release(), Some(first));
        assert!(slot.pick_up(WorldObjectId::ResourceFragment(FragmentId(2))));
    }

    #[test]
    fn targeting_checks_sphere_range_occlusion_and_carried_identity() {
        let ray = MiningRay::new(
            WorldPosition::new(0.0, 0.0, 0.0),
            WorldPosition::new(0.0, 0.0, 1.0),
        )
        .unwrap();
        let pieces = [piece(2, 2.0), piece(1, 1.0), piece(3, 4.0)];
        assert_eq!(
            aimed_fragment(ray, &pieces, None, f64::INFINITY),
            Some(FragmentId(1))
        );
        assert_eq!(
            aimed_fragment(
                ray,
                &pieces,
                Some(WorldObjectId::ResourceFragment(FragmentId(1))),
                f64::INFINITY
            ),
            Some(FragmentId(2))
        );
        assert_eq!(aimed_fragment(ray, &pieces, None, 0.5), None);
        assert_eq!(aimed_fragment(ray, &pieces[2..], None, f64::INFINITY), None);
        let miss = MiningRay::new(
            WorldPosition::new(0.0, 0.0, 0.0),
            WorldPosition::new(1.0, 0.0, 1.0),
        )
        .unwrap();
        assert_eq!(aimed_fragment(miss, &pieces, None, f64::INFINITY), None);
    }
}
