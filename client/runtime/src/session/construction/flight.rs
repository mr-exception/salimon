//! Flight admission derives from physical consequences and the authoritative hull.
use super::*;
use salimon_ship::{CockpitMessage, FlightState};

fn requires_ground(p: &Placeholder) -> bool {
    p.blueprint.hull_affecting || !p.blueprint.flight_eligible
}
impl GameplaySession {
    /// Current construction publishes sealed geometry atomically. There are no
    /// half-built hull openings to pretend are sealed. Detect missing/edited
    /// completed faces and attachment loss against their canonical source.
    pub(super) fn construction_hull_ready(&self) -> bool {
        self.building
            .source
            .structure(None)
            .is_ok_and(|expected| expected == self.building.structure)
            && !self
                .building
                .source
                .projects
                .jobs()
                .any(|p| p.work.completed && p.blueprint.hull_affecting && p.work.damaged)
    }
    pub(super) fn construction_can_secure(&self, p: &Placeholder) -> bool {
        !p.work.damaged
            && self.construction_hull_ready()
            && (!is_structural(p) || self.structural_support(p.space))
    }
    pub(in crate::session) fn construction_takeoff_blocker(&self) -> Option<CockpitMessage> {
        if !self.construction_hull_ready() {
            return Some(CockpitMessage::RepairStructuralHull);
        }
        for p in self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| !p.work.completed && requires_ground(p))
        {
            if !p.work.stopped {
                return Some(CockpitMessage::StopExteriorConstruction);
            }
            if !self.construction_can_secure(p) {
                return Some(CockpitMessage::SecureExteriorConstruction);
            }
        }
        None
    }
    pub(crate) fn contextual_cockpit_message(&self) -> Option<CockpitMessage> {
        let ship = self.ship.snapshot();
        if ship.power_available
            && ship.cockpit_control_active
            && matches!(ship.flight_state, FlightState::Landed { .. })
        {
            // Preserve existing actual Core/engine and open-ramp precedence.
            if !ship.engines_enabled {
                return Some(CockpitMessage::EnginesOff);
            }
            if ship.door_open_fraction > 0. || ship.door_state == salimon_ship::DoorState::Open {
                return Some(CockpitMessage::CloseDoorBeforeTakeoff);
            }
            if let Some(reason) = self.construction_takeoff_blocker() {
                return Some(reason);
            }
        }
        self.ship.contextual_cockpit_message()
    }
    pub(super) fn stop_resume_construction(&mut self) {
        let Some(id) = self.aimed_construction() else {
            self.feedback = Some("Aim at a reachable unfinished project");
            return;
        };
        if self.module_stop_resume(id) {
            return;
        }
        let p = self
            .building
            .source
            .projects
            .inspect(id)
            .expect("aimed project");
        if p.work.completed {
            self.feedback = Some("Completed objects cannot be stopped");
            return;
        }
        let resuming = p.work.stopped;
        if resuming && (p.work.damaged || self.construction_safety(p).is_err()) {
            self.feedback = Some("Cannot resume: repair damage and clear the site while landed");
            return;
        }
        let secured = self.construction_can_secure(p);
        self.building
            .source
            .projects
            .funding_mut(id, OWNER)
            .expect("local owner")
            .work
            .stopped = !resuming;
        self.building.running = None;
        self.feedback = Some(if resuming {
            "Construction resumed"
        } else if secured {
            "Construction stopped and secured"
        } else {
            "Construction stopped - unsecured; repair support or cancel"
        });
    }
    pub(super) fn stop_airborne_construction(&mut self) {
        if matches!(
            self.ship.snapshot().flight_state,
            FlightState::Landed { .. }
        ) {
            return;
        }
        let ids: Vec<_> = self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| !p.work.completed && requires_ground(p))
            .map(|p| p.id)
            .collect();
        for id in ids {
            self.building
                .source
                .projects
                .funding_mut(id, OWNER)
                .expect("local owner")
                .work
                .stopped = true;
        }
    }
}

#[cfg(test)]
mod tests;
