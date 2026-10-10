//! Permanent one-world-object carrying rule. Equipped gear uses separate systems.
use crate::mining::MiningRay;
use crate::resource_fragments::side_meters;
use crate::resources::{FragmentId, ResourceFragment};

pub const PICKUP_RANGE_METERS: f64 = 3.0;

/// An identity in the physical world, never an abstract inventory quantity.
/// Future carriable object kinds must share this same slot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldObjectId {
    ResourceFragment(FragmentId),
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

/// Target the oriented conservative cube, excluding the carried object. Hull and terrain
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
            let q = piece.transform().orientation_xyzw();
            let inverse = [-q[0], -q[1], -q[2], q[3]];
            let center = rotate(inverse, piece.transform().position().offset_from(ray.origin));
            let direction = rotate(inverse, ray.direction);
            let half = side_meters(*piece) * 0.5;
            let mut near: f64 = 0.0;
            let mut far = PICKUP_RANGE_METERS.min(blocked_at);
            for (axis, component) in center.iter().enumerate() {
                let direction = direction[axis];
                if direction.abs() < 1e-12 {
                    if component.abs() > half {
                        return None;
                    }
                } else {
                    let a = (component - half) / direction;
                    let b = (component + half) / direction;
                    near = near.max(a.min(b));
                    far = far.min(a.max(b));
                }
            }
            (near <= far && near < blocked_at).then_some((piece.id(), near))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.0.cmp(&b.0.0)))
        .map(|target| target.0)
}

fn rotate(q: [f64; 4], p: [f64; 3]) -> [f64; 3] {
    let v = [q[0], q[1], q[2]];
    let cross = |a: [f64; 3], b: [f64; 3]| [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]];
    let t = cross(v, p).map(|v| v * 2.0);
    let c = cross(v, t);
    std::array::from_fn(|i| p[i] + q[3] * t[i] + c[i])
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
    fn targeting_checks_cube_range_occlusion_and_carried_identity() {
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
