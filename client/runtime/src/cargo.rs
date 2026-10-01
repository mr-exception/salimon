//! Derived membership of real loose entities in the ship's physical cargo room.
//! No inventory state is stored: pickup/drop and ship support remain authoritative.
use crate::mining::MiningTool;
use salimon_character::{CARGO_ROOM_MAX_METERS, CARGO_ROOM_MIN_METERS};
use salimon_world::resources::ResourceFragment;

pub(crate) fn contains(local: [f64; 3], clearance: f64) -> bool {
    clearance.is_finite()
        && clearance > 0.0
        && (0..3).all(|i| {
            local[i].is_finite()
                && local[i] - clearance >= CARGO_ROOM_MIN_METERS[i] - 1e-6
                && local[i] + clearance <= CARGO_ROOM_MAX_METERS[i] + 1e-6
        })
}

/// Use the supporting ship-local pose to avoid large-world precision loss.
/// Conservative sphere bounds match interior placement clearance for rotated cubes.
pub(crate) fn stored(tool: &MiningTool, piece: ResourceFragment) -> bool {
    tool.session.carried_id() != Some(piece.id())
        && tool.ship_fragments.get(&piece.id()).is_some_and(|&local| {
            contains(
                local,
                salimon_world::resource_fragments::side_meters(piece) * 0.5 * 3.0_f64.sqrt(),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn containment_requires_the_entire_object_and_rejects_invalid_bounds() {
        let min = CARGO_ROOM_MIN_METERS;
        let max = CARGO_ROOM_MAX_METERS;
        let center = std::array::from_fn(|i| (min[i] + max[i]) * 0.5);
        assert!(contains(center, 0.1));
        for axis in 0..3 {
            for bound in [min[axis], max[axis]] {
                let mut point = center;
                point[axis] = bound;
                assert!(!contains(point, 0.1));
            }
        }
        assert!(contains([1.0, min[1] + 0.1, 8.0], 0.1));
        for clearance in [0.0, -1.0, f64::NAN, f64::INFINITY, 10.0] {
            assert!(!contains(center, clearance));
        }
        assert!(!contains([f64::NAN, 1.0, 8.0], 0.1));
    }
    #[test]
    fn only_existing_loose_ship_supported_fragments_are_cargo() {
        use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};
        use salimon_world::{CelestialBodyId, WorldPosition};
        let mut tool = MiningTool::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, 2.0).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            2.0,
        )
        .unwrap();
        tool.session
            .extract(&mut deposit, std::time::Duration::from_secs(1));
        let piece = tool.session.fragments()[0];
        assert!(!stored(&tool, piece));
        tool.ship_fragments.insert(piece.id(), [1.0, 0.4, 8.0]);
        assert!(stored(&tool, piece));
        assert!(tool.session.pick_up(piece.id()));
        assert!(!stored(&tool, piece));
        assert!(tool.session.drop_carried(piece.transform()));
        assert!(stored(&tool, piece));
        tool.ship_fragments.insert(piece.id(), [-5.8, 0.4, 0.0]);
        assert!(!stored(&tool, piece));
        tool.ship_fragments.insert(piece.id(), [1.0, 0.4, 5.1]);
        assert!(!stored(&tool, piece));
        assert_eq!(tool.session.fragments().len(), 1);
        assert_eq!(tool.session.fragments()[0].material(), piece.material());
    }
}
