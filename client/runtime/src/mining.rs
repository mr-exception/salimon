//! Tool input and session extraction composition; no general resource inventory.
use std::time::Duration;

use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_renderer::SceneInstance;
use salimon_world::WorldPosition;
use salimon_world::mining::{MiningRay, MiningSession, MiningTarget, aimed_deposit};
use salimon_world::resource_generation::{GenerationError, SurfaceDeposit};

#[derive(Default)]
pub(crate) struct MiningTool {
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
        if !self.equipped || player.location != CharacterLocation::Surface {
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

    pub(crate) fn visuals(&self, player: CharacterSnapshot, active: bool) -> Vec<SceneInstance> {
        if !self.equipped || player.location != CharacterLocation::Surface {
            return Vec::new();
        }
        let Some(ray) = MiningRay::new(
            position(player.eye_position_meters),
            position(player.look_target_meters),
        ) else {
            return Vec::new();
        };
        let up = player.up.map(f64::from);
        let right = [
            ray.direction[1] * up[2] - ray.direction[2] * up[1],
            ray.direction[2] * up[0] - ray.direction[0] * up[2],
            ray.direction[0] * up[1] - ray.direction[1] * up[0],
        ];
        let center = std::array::from_fn(|i| {
            player.eye_position_meters[i] + ray.direction[i] * 0.55 + right[i] * 0.20 - up[i] * 0.20
        });
        vec![
            SceneInstance {
                center_meters: center,
                half_extents_meters: [0.06, 0.06, 0.06],
                color: [0.24, 0.27, 0.30, 1.0],
            },
            SceneInstance {
                center_meters: std::array::from_fn(|i| {
                    center[i] - ray.direction[i] * 0.07 + up[i] * 0.04
                }),
                half_extents_meters: [0.035, 0.035, 0.035],
                color: if active {
                    [0.2, 1.0, 0.8, 1.0]
                } else {
                    [0.9, 0.6, 0.15, 1.0]
                },
            },
            SceneInstance {
                center_meters: std::array::from_fn(|i| center[i] - up[i] * 0.08),
                half_extents_meters: [0.03, 0.05, 0.03],
                color: [0.15, 0.17, 0.20, 1.0],
            },
            // A small aim marker on the camera ray, independent of world scale.
            SceneInstance {
                center_meters: std::array::from_fn(|i| {
                    player.eye_position_meters[i] + ray.direction[i] * 0.5
                }),
                half_extents_meters: [0.002; 3],
                color: [0.9, 0.9, 0.9, 1.0],
            },
        ]
    }
}

fn position(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}
