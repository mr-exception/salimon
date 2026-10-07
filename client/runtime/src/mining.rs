//! Tool input and session extraction composition; no general resource inventory.
use std::time::Duration;

use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_renderer::HeldItemInstance;
use salimon_world::WorldPosition;
use salimon_world::mining::{MiningRay, MiningSession, MiningTarget, aimed_deposit};
use salimon_world::resource_generation::{GenerationError, SurfaceDeposit};

#[derive(Default)]
pub(crate) struct MiningTool {
    pub(crate) ship_fragments:
        std::collections::HashMap<salimon_world::resources::FragmentId, [f64; 3]>,
    pub(crate) fragment_motion: std::collections::HashMap<
        salimon_world::resources::FragmentId,
        crate::fragment_physics::FragmentMotion,
    >,
    pub(crate) carry_feedback: Option<&'static str>,
    pub(crate) equipped: bool,
    pub(crate) held: bool,
    f_down: bool,
    f_mining: bool,
    mouse_held: bool,
    pub(crate) session: MiningSession,
}

impl MiningTool {
    pub(crate) fn toggle(&mut self) {
        self.equipped = !self.equipped;
        self.clear_input();
    }

    /// Latch the initial F edge; a carrying action consumes this entire press.
    pub(crate) fn press_f(&mut self) -> bool {
        if self.f_down {
            return false;
        }
        self.f_down = true;
        true
    }

    pub(crate) fn start_f_mining(&mut self) {
        self.f_mining = true;
        self.held = true;
    }

    pub(crate) fn release_f(&mut self) {
        self.f_down = false;
        self.f_mining = false;
        self.held = self.mouse_held;
    }

    pub(crate) fn set_mouse_held(&mut self, pressed: bool) {
        self.mouse_held = pressed;
        self.held = self.f_mining || self.mouse_held;
    }

    pub(crate) fn clear_input(&mut self) {
        self.f_down = false;
        self.f_mining = false;
        self.mouse_held = false;
        self.held = false;
    }

    pub(crate) fn nearby(
        &self,
        position: [f64; 3],
        seed: u64,
    ) -> Result<Vec<SurfaceDeposit>, GenerationError> {
        let mut entries = crate::resource_presentation::nearby_deposits(position, seed)?;
        self.session.apply_to(&mut entries);
        Ok(entries)
    }

    pub(crate) fn target(
        &self,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) -> Option<MiningTarget> {
        self.equipped
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
        aimed_deposit(
            ray,
            &self.nearby(player.eye_position_meters, seed).ok()?,
            obstruction,
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
        delta: Duration,
        player: CharacterSnapshot,
        frame: ShipFrame,
        door_open: bool,
        seed: u64,
    ) {
        if !self.held {
            return;
        }
        let Some(target) = self.target(player, frame, door_open, seed) else {
            return;
        };
        let Ok(mut deposits) = self.nearby(player.eye_position_meters, seed) else {
            return;
        };
        let entry = deposits
            .iter_mut()
            .find(|entry| entry.deposit.id() == target.id)
            .expect("target comes from the same deterministic query");
        self.session.extract(&mut entry.deposit, delta);
    }

    /// Equip/location gates and active feedback for the authored visual.
    /// The caller supplies gameplay visibility and the existing target result.
    pub(crate) fn held_item(
        &self,
        player: CharacterSnapshot,
        valid_target: bool,
    ) -> Option<HeldItemInstance> {
        (player.location == CharacterLocation::Surface && self.equipped).then_some(
            HeldItemInstance {
                active: self.held && valid_target,
            },
        )
    }
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
        tool.start_f_mining();
        tool.set_mouse_held(false);
        assert!(tool.held, "mouse release must preserve F mining");
        tool.clear_input();
        assert!(!tool.held);
        assert!(tool.press_f(), "reset clears the F latch");
        tool.start_f_mining();
        tool.toggle();
        assert!(!tool.held);
        assert!(tool.press_f(), "stow clears the F latch too");
    }

    #[test]
    fn authored_tool_visibility_and_feedback_follow_existing_gates() {
        let mut tool = MiningTool::default();
        let mut player = CharacterSnapshot {
            location: CharacterLocation::Surface,
            eye_position_meters: [0.0; 3],
            look_target_meters: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            local_ship_position_meters: None,
            doorway_blend_fraction: None,
        };
        assert!(tool.held_item(player, true).is_none());
        tool.toggle();
        assert_eq!(
            tool.held_item(player, true),
            Some(HeldItemInstance { active: false })
        );
        tool.held = true;
        assert_eq!(
            tool.held_item(player, false),
            Some(HeldItemInstance { active: false })
        );
        assert_eq!(
            tool.held_item(player, true),
            Some(HeldItemInstance { active: true })
        );
        player.location = CharacterLocation::InsideShip;
        assert!(tool.held_item(player, true).is_none());
        player.location = CharacterLocation::Surface;
        tool.toggle();
        assert!(tool.held_item(player, true).is_none());
        assert!(!tool.held);
    }
}
