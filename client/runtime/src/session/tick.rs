//! Explicit deterministic scheduler. Owners contain stage details, never a registry.
use super::GameplaySession;
use salimon_character::CharacterSnapshot;
use salimon_ship::ShipSnapshot;
use std::time::Duration;
mod character;
mod power;
mod resources;
#[cfg(test)]
mod tests;

impl GameplaySession {
    pub(crate) fn advance_automation(&mut self, elapsed: Duration) {
        if self.paused {
            return;
        }
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
            return;
        }
        self.update_clock.accumulate(elapsed);
        while let Some(tick) = self.update_clock.take_tick() {
            self.advance_game(tick);
        }
    }
    pub(crate) fn advance_game(&mut self, requested: Duration) {
        if !self.sync_organism_collisions() {
            return;
        }
        let Some(delta) = self.admit_tick(requested) else {
            return;
        };
        self.begin_tick_presentation();
        let journey_ship_start = self.ship.snapshot().pose.position_meters;
        let ship = self.advance_powered_domains(delta);
        // Both character endpoints use the completed ship frame, so cabin
        // transport cannot become extra walking distance.
        let journey_character_start = self.snapshot().character;
        self.advance_character_tick(delta, ship);
        self.advance_resources_tick(delta, ship);
        self.finish_tick(delta, ship, journey_ship_start, journey_character_start);
    }
    fn admit_tick(&mut self, delta: Duration) -> Option<Duration> {
        if self.paused {
            return None;
        }
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
            return None;
        }
        self.finish_offline_remainder();
        let before = self.survival.snapshot().active_time;
        self.survival.advance(delta);
        let delta = self.survival.snapshot().active_time - before;
        self.enforce_survival_restrictions();
        self.elapsed += delta;
        Some(delta)
    }
    fn finish_tick(
        &mut self,
        delta: Duration,
        ship: ShipSnapshot,
        journey_ship_start: [f64; 3],
        journey_character_start: CharacterSnapshot,
    ) {
        let journey_character_end = self.snapshot().character;
        self.run.journey.record(
            journey_ship_start,
            ship.pose.position_meters,
            journey_character_start,
            journey_character_end,
            !delta.is_zero(),
        );
        self.update_field_tick();
        self.sync_carried();
        self.validate_crafting_focus();
        self.validate_archive_focus();
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
        }
        self.complete_tick_presentation();
    }
}
