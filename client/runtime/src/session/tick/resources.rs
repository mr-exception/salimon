use crate::session::{GameplaySession, ViewMode, door_passable, frames::character_ship_frame};
use salimon_ship::ShipSnapshot;
use std::time::Duration;

impl GameplaySession {
    pub(super) fn advance_resources_tick(&mut self, delta: Duration, ship: ShipSnapshot) {
        if self.view_mode == ViewMode::Gameplay {
            let frame = character_ship_frame(ship.pose);
            let extracted_before = self.mining.session.extracted_mass_kg();
            if !self.advance_biology(delta) {
                self.mining.advance(
                    &self.equipment,
                    self.mining_delta(delta),
                    self.character.snapshot(frame, self.terrain_frame(ship)),
                    frame,
                    door_passable(ship),
                    self.world_seed(),
                );
            }
            if self.mining.session.extracted_mass_kg() > extracted_before {
                self.surface_age
                    .contact(salimon_ship::surface_age::TOOL, delta);
            }
            self.mining.biological_obstacles = self.organism_field();
            let player = self
                .character
                .snapshot(frame, self.terrain_frame(ship))
                .eye_position_meters;
            let measurements = if self.policy.measures_physics() {
                crate::fragment_physics::advance_measured(
                    &mut self.mining,
                    frame,
                    delta,
                    player,
                    true,
                )
            } else {
                crate::fragment_physics::advance(&mut self.mining, frame, delta, player);
                Default::default()
            };
            self.latest_physics.accumulate(measurements);
        }
    }
}
