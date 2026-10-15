//! Tool input and session extraction composition; no general resource inventory.
use std::time::Duration;

use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_renderer::HeldItemInstance;
use salimon_world::WorldPosition;
use salimon_world::mining::{MiningRay, MiningSession, MiningTarget, aimed_deposit_with_terrain};
use salimon_world::resource_generation::{GenerationError, SurfaceDeposit};

#[derive(Default)]
pub(crate) struct MiningTool {
    pub(crate) ship_gravity_disabled: bool,
    pub(crate) ship_mass: salimon_ship::ShipMass,
    pub(crate) terrain: salimon_world::terrain::TerrainMode,
    placements: std::cell::RefCell<crate::resource_presentation::PlacementCache>,
    pub(crate) fragment_previous: std::collections::HashMap<
        salimon_world::resources::FragmentId,
        salimon_physics::ObjectState,
    >,
    pub(crate) fragment_sleep: salimon_physics::SleepTracker<salimon_world::resources::FragmentId>,
    pub(crate) ship_fragments:
        std::collections::HashMap<salimon_world::resources::FragmentId, [f64; 3]>,
    pub(crate) fragment_motion: std::collections::HashMap<
        salimon_world::resources::FragmentId,
        crate::fragment_physics::FragmentMotion,
    >,
    pub(crate) carry_feedback: Option<&'static str>,
    pub(crate) held: bool,
    pub(crate) config: salimon_world::spherical_mining::MiningConfig,
    f_down: bool,
    pub(crate) session: MiningSession,
}

impl MiningTool {
    /// Latch the initial F edge; a carrying action consumes this entire press.
    pub(crate) fn press_f(&mut self) -> bool {
        if self.f_down {
            return false;
        }
        self.f_down = true;
        true
    }

    pub(crate) fn release_f(&mut self) {
        self.f_down = false;
    }

    pub(crate) fn set_mouse_held(&mut self, pressed: bool) {
        self.held = pressed;
        if !pressed {
            self.session.interrupt_spherical();
        }
    }

    /// Cancel extraction while preserving a contextual F press consumed by pickup.
    pub(crate) fn stop_mining(&mut self) {
        self.held = false;
        self.session.interrupt_spherical();
    }

    pub(crate) fn clear_input(&mut self) {
        self.f_down = false;
        self.stop_mining();
    }

    pub(crate) fn nearby(
        &self,
        position: [f64; 3],
        seed: u64,
    ) -> Result<Vec<SurfaceDeposit>, GenerationError> {
        let mut entries = crate::resource_presentation::nearby_deposits_cached(
            position,
            seed,
            self.terrain,
            &self.placements,
        )?;
        self.session.apply_to(&mut entries);
        Ok(entries)
    }

    pub(crate) fn target(
        &self,
        equipment: &crate::equipment::EquipmentToolbar,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) -> Option<MiningTarget> {
        equipment
            .mining_equipped()
            .then(|| self.inspect_target(player, frame, door_open, seed))
            .flatten()
    }

    /// Inspect a reachable deposit even when the handheld tool is stowed.
    pub(crate) fn inspect_target(
        &self,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) -> Option<MiningTarget> {
        if player.location != CharacterLocation::Surface {
            return None;
        }
        let origin = position(player.eye_position_meters);
        let ray = MiningRay::new(origin, position(player.look_target_meters))?;
        let obstruction = salimon_character::ship_sight_obstruction(
            frame.world_to_local(player.eye_position_meters),
            frame.world_to_local(player.look_target_meters),
            door_open,
        );
        aimed_deposit_with_terrain(
            ray,
            &self.nearby(player.eye_position_meters, seed).ok()?,
            obstruction,
            self.terrain,
        )
    }

    pub(crate) fn nearby_fragments(
        &self,
        player: [f64; 3],
    ) -> impl Iterator<Item = salimon_world::resources::ResourceFragment> + '_ {
        self.session
            .fragments()
            .iter()
            .copied()
            .filter(move |piece| {
                let offset = piece.transform().position().offset_from(position(player));
                offset.iter().map(|v| v * v).sum::<f64>()
                    <= crate::resource_presentation::ACTIVE_RADIUS_METERS.powi(2)
            })
    }

    pub(crate) fn advance(
        &mut self,
        equipment: &crate::equipment::EquipmentToolbar,
        delta: Duration,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) {
        let Some(pose) = attachment_pose(player) else {
            self.session.interrupt_spherical();
            return;
        };
        let Some(target) = self
            .held
            .then(|| self.target(equipment, player, frame, door_open, seed))
            .flatten()
        else {
            self.session
                .tick_spherical(self.held, None, delta, self.config, pose);
            return;
        };
        let Ok(mut deposits) = self.nearby(player.eye_position_meters, seed) else {
            self.session.interrupt_spherical();
            return;
        };
        let target = deposits
            .iter_mut()
            .find(|entry| entry.deposit.id() == target.id);
        self.session.tick_spherical(
            self.held,
            target.map(|entry| &mut entry.deposit),
            delta,
            self.config,
            pose,
        );
    }

    /// Equip/location gates and active feedback for the authored visual.
    /// The caller supplies gameplay visibility and the existing target result.
    pub(crate) fn held_item(
        &self,
        equipment: &crate::equipment::EquipmentToolbar,
        player: CharacterSnapshot,
        valid_target: bool,
    ) -> Option<HeldItemInstance> {
        (player.location == CharacterLocation::Surface && equipment.mining_equipped()).then_some(
            HeldItemInstance {
                active: self.held && valid_target,
            },
        )
    }
}

/// Current camera/tool front-left in absolute f64 meters. Renderer grip is
/// camera-right 0.20, down 0.20, forward 0.55; this point is left of its muzzle.
fn attachment_pose(
    player: CharacterSnapshot,
) -> Option<salimon_world::resources::ResourceTransform> {
    use salimon_math::{cross, length, sub};
    let direction = sub(player.look_target_meters, player.eye_position_meters);
    let magnitude = length(direction);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return None;
    }
    let forward = direction.map(|v| v / magnitude);
    let right = cross(forward, player.up.map(f64::from));
    let magnitude = length(right);
    if !magnitude.is_finite() || magnitude <= 1e-12 {
        return None;
    }
    let right = right.map(|v| v / magnitude);
    let up = cross(right, forward);
    let point = std::array::from_fn(|i| {
        player.eye_position_meters[i] + forward[i] * 0.85 - right[i] * 0.15 - up[i] * 0.20
    });
    salimon_world::resources::ResourceTransform::new(position(point), [0.0, 0.0, 0.0, 1.0]).ok()
}

fn position(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrying_press_is_latched_and_mouse_mining_is_independent() {
        let mut tool = MiningTool::default();
        assert!(tool.press_f());
        assert!(
            !tool.held,
            "carrying consumes F without starting extraction"
        );
        assert!(!tool.press_f(), "repeated press cannot change the action");
        tool.set_mouse_held(true);
        tool.release_f();
        assert!(tool.held, "F release must preserve left mouse mining");
        assert!(tool.press_f());
        tool.set_mouse_held(false);
        assert!(
            !tool.held,
            "held F cannot sustain mining after mouse release"
        );
        tool.set_mouse_held(true);
        tool.stop_mining();
        assert!(!tool.held);
        assert!(!tool.press_f(), "pickup cancellation preserves the F latch");
        tool.clear_input();
        assert!(tool.press_f(), "input reset clears the F latch");
        assert!(!tool.held, "F after reset cannot start mining");
    }

    #[test]
    fn authored_tool_visibility_and_feedback_follow_existing_gates() {
        let mut tool = MiningTool::default();
        let mut equipment = crate::equipment::EquipmentToolbar::default();
        let mut player = CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: [0.0; 3],
            look_target_meters: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        };
        assert!(tool.held_item(&equipment, player, true).is_none());
        equipment.select(crate::equipment::ToolbarSlot::One, false);
        assert_eq!(
            tool.held_item(&equipment, player, true),
            Some(HeldItemInstance { active: false })
        );
        tool.held = true;
        assert_eq!(
            tool.held_item(&equipment, player, false),
            Some(HeldItemInstance { active: false })
        );
        assert_eq!(
            tool.held_item(&equipment, player, true),
            Some(HeldItemInstance { active: true })
        );
        player.location = CharacterLocation::InsideShip;
        assert!(tool.held_item(&equipment, player, true).is_none());
        player.location = CharacterLocation::Surface;
        equipment.sync_carrying(true);
        tool.clear_input();
        assert!(tool.held_item(&equipment, player, true).is_none());
        assert!(!tool.held);
    }
}

#[cfg(test)]
mod spherical_tests {
    use super::*;
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};

    fn player() -> CharacterSnapshot {
        CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: [0.0, 2.0, 0.0],
            look_target_meters: [1.0, 2.0, 0.0],
            up: [0.0, 1.0, 0.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        }
    }

    #[test]
    fn attached_piece_follows_tool_and_is_excluded_from_loose_physics() {
        let mut tool = MiningTool::default();
        let pose = attachment_pose(player()).unwrap();
        let mut source = ResourceDeposit::new(
            DepositId {
                body: salimon_world::CelestialBodyId::Earth,
                local: 99,
            },
            RawMaterial::new(ResourceId::IronOre, 25.0).unwrap(),
            WorldPosition::new(2.0, 0.0, 0.0),
            25.0,
        )
        .unwrap();
        tool.session.tick_spherical(
            true,
            Some(&mut source),
            Duration::from_millis(16),
            tool.config,
            pose,
        );
        let id = tool.session.attached_sphere_id().unwrap();
        let frame = ShipFrame {
            origin_meters: [0.0; 3],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        assert!(crate::fragment_physics::snapshots(&tool, frame, [0.0; 3]).is_empty());
        assert!(!tool.session.pick_up(id));
        tool.set_mouse_held(false);
        assert_eq!(tool.session.attached_sphere_id(), None);
        assert_eq!(
            crate::fragment_physics::snapshots(&tool, frame, [0.0; 3]).len(),
            1
        );
        assert_eq!(tool.session.fragments()[0].id(), id);
        assert!(tool.session.pick_up(id));
    }

    #[test]
    fn tool_attachment_uses_current_camera_basis_and_absolute_f64_pose() {
        let original = attachment_pose(player()).unwrap().position().meters();
        assert_eq!(original, [0.85, 1.8, -0.15]);
        let mut moved = player();
        moved.eye_position_meters = [1e12, -1e12, 1e12];
        moved.look_target_meters = [1e12, -1e12, 1e12 + 1.0];
        let offset = attachment_pose(moved)
            .unwrap()
            .position()
            .offset_from(position(moved.eye_position_meters));
        for (actual, expected) in offset.into_iter().zip([0.15, -0.20, 0.85]) {
            assert!((actual - expected).abs() < 0.0002);
        }
    }
}
