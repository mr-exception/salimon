use crate::session::{GameplaySession, ViewMode, door_passable, frames::character_ship_frame};
use salimon_ship::{FlightState, ShipSnapshot};
use std::time::Duration;

impl GameplaySession {
    pub(super) fn advance_character_tick(&mut self, delta: Duration, ship: ShipSnapshot) {
        let wear_walk_start = self.character.local_ship_position();
        if self.view_mode == ViewMode::Gameplay {
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
            self.advance_cargo_door(delta);
            self.advance_habitation(delta);
            self.character
                .set_ramp_moving(ship.door_open_fraction > 0.0 && ship.door_open_fraction < 1.0);
            if self
                .habitation
                .cabin
                .as_ref()
                .is_none_or(|c| c.rest.is_none())
            {
                self.character.advance_with_motion(
                    delta,
                    self.input.movement,
                    (
                        character_ship_frame(ship.pose),
                        ship.velocity_meters_per_second,
                    ),
                    self.terrain_frame(ship),
                    door_passable(ship),
                    matches!(ship.flight_state, FlightState::Landed { .. }),
                );
            }
            self.record_floor_contact(wear_walk_start, delta);
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
        }
    }
}
