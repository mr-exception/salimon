//! Tool input and session extraction composition; no general resource inventory.
use std::time::Duration;

use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_renderer::SceneInstance;
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
    pub(crate) session: MiningSession,
}

impl MiningTool {
    pub(crate) fn toggle(&mut self) {
        self.equipped = !self.equipped;
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

    pub(crate) fn visuals(&self, player: CharacterSnapshot) -> Vec<SceneInstance> {
        if player.location != CharacterLocation::Surface {
            return Vec::new();
        }
        let Some(ray) = MiningRay::new(
            position(player.eye_position_meters),
            position(player.look_target_meters),
        ) else {
            return Vec::new();
        };
        let marker = SceneInstance {
            center_meters: std::array::from_fn(|i| {
                player.eye_position_meters[i] + ray.direction[i] * 0.5
            }),
            half_extents_meters: [0.002; 3],
            color: [0.9, 0.9, 0.9, 1.0],
        };
        vec![marker]
    }
}

fn position(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}
