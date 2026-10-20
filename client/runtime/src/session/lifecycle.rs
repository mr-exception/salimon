//! Portable lifecycle source state. Disk publication and native input stay in the host.
use super::{GameplaySession, ViewMode};
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;

pub(crate) use super::journey::JourneyState;
pub(crate) const ARCHIVE_VERSION: u32 = 2;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunState {
    pub(crate) serial: u64,
    pub(crate) journey: JourneyState,
}
impl Default for RunState {
    fn default() -> Self {
        Self {
            serial: 1,
            journey: Default::default(),
        }
    }
}
impl Validate for RunState {
    fn validate(&self) -> Result<(), SaveError> {
        if self.serial == 0
            || !self.journey.distance_meters.is_finite()
            || self.journey.distance_meters < 0.0
        {
            return Err(invalid("invalid run identity/journey counters"));
        }
        self.journey.validate()
    }
}
#[derive(serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EndedRun {
    schema_version: u32,
    serial: u64,
    seed: u64,
    pub(crate) active_duration: Duration,
    pub(crate) cause: salimon_survival::effects::Source,
    position_meters: [f64; 3],
    pub(crate) journey: JourneyState,
    pub(crate) final_location: super::journey::NamedLocation,
}
impl GameplaySession {
    pub(super) fn freeze_terminal(&mut self) {
        self.clear_held_controls();
        self.ship.revoke_pilot_control();
        self.update_clock.reset();
        self.reset_presentation();
    }
    pub(crate) fn ended_run_bytes(&self) -> Result<Vec<u8>, SaveError> {
        serde_json::to_vec(&self.ended_run()?).map_err(|e| SaveError::Encoding(e.to_string()))
    }
    /// One typed report is shared by native presentation and durable archives.
    pub(crate) fn ended_run(&self) -> Result<EndedRun, SaveError> {
        self.run.validate()?;
        let snapshot = self.survival.snapshot();
        if !snapshot.terminal {
            return Err(invalid("cannot archive a living run"));
        }
        Ok(EndedRun {
            schema_version: ARCHIVE_VERSION,
            serial: self.run.serial,
            seed: self.seed,
            active_duration: snapshot.active_time,
            cause: snapshot
                .death_cause
                .ok_or_else(|| invalid("terminal cause missing"))?,
            position_meters: self.snapshot().character.eye_position_meters,
            journey: self.run.journey.clone(),
            final_location: super::journey::NamedLocation::at(
                self.snapshot().character.eye_position_meters,
            ),
        })
    }
    /// Build without publishing. The host archives/checkpoints before replacing self.
    pub(crate) fn fresh_after_death(&self) -> Result<Self, SaveError> {
        if !self.survival.snapshot().terminal {
            return Err(invalid("restart requires terminal run"));
        }
        let serial = self
            .run
            .serial
            .checked_add(1)
            .ok_or_else(|| invalid("run identity exhausted"))?;
        let mut fresh = Self {
            seed: self.seed,
            automation_step: self.automation_step,
            interpolate_presentation: self.interpolate_presentation,
            measure_physics: self.measure_physics,
            run: RunState {
                serial,
                ..Default::default()
            },
            ..Self::new_run()
        };
        if self.mining.terrain == salimon_world::terrain::TerrainMode::Representative {
            let earth = *salimon_world::CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
                .expect("Earth catalog");
            fresh
                .enable_terrain(earth, [0.0, 1.0, 0.0])
                .map_err(invalid)?;
        }
        fresh.view_mode = ViewMode::Gameplay;
        Ok(fresh)
    }
    /// Recovery happens only during launch; publication is the caller's transaction.
    pub(crate) fn restore_for_launch(
        &mut self,
        bytes: &[u8],
        launch_utc: u64,
    ) -> Result<(), SaveError> {
        #[derive(serde::Deserialize)]
        struct Header {
            saved_at_unix_seconds: u64,
        }
        self.restore_bytes(bytes)?;
        let header: Header =
            serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
        self.survival
            .recover_offline(header.saved_at_unix_seconds, launch_utc);
        self.enforce_survival_restrictions();
        Ok(())
    }
}

#[cfg(test)]
mod tests;
