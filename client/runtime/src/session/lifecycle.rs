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
            final_location: super::journey::NamedLocation::with_terrain(
                self.snapshot().character.eye_position_meters,
                self.mining.terrain,
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
        let mut fresh = if self.mining.terrain == salimon_world::terrain::TerrainMode::Flat {
            Self::new_run()
        } else {
            Self::new_canonical_run().map_err(invalid)?
        };
        fresh.seed = self.seed;
        fresh.policy = self.policy;
        fresh.run = RunState {
            serial,
            ..Default::default()
        };
        fresh.view_mode = ViewMode::Gameplay;
        Ok(fresh)
    }
    /// Recovery happens only during launch; publication is the caller's transaction.
    pub(crate) fn restore_for_launch(
        &mut self,
        bytes: &[u8],
        launch_utc: u64,
    ) -> Result<(), SaveError> {
        self.restore_for_launch_reported(bytes, launch_utc, &mut |_| Ok(()))
    }
    pub(crate) fn restore_for_launch_reported(
        &mut self,
        bytes: &[u8],
        launch_utc: u64,
        progress: &mut impl FnMut(crate::launch_progress::Progress) -> Result<(), SaveError>,
    ) -> Result<(), SaveError> {
        #[derive(serde::Deserialize)]
        struct Header {
            saved_at_unix_seconds: u64,
            schema_version: u32,
        }
        let mut candidate = GameplaySession {
            policy: self.policy,
            ..Default::default()
        };
        candidate.restore_bytes(bytes)?;
        if candidate.motion.is_some() {
            candidate.ship.shut_down_for_close();
        }
        let header: Header =
            serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
        if candidate.motion.is_some() && launch_utc < candidate.offline.watermark {
            return Err(invalid(
                "System clock precedes the saved journey. Correct the local date/time and retry; the checkpoint was not changed.",
            ));
        }
        if candidate.motion.is_some() {
            salimon_universe::starting_motion::Clock::at(launch_utc)
                .map_err(|e| invalid(format!("invalid local clock: {e}")))?;
        }
        if header.schema_version < 30 && candidate.motion.is_some() {
            // Legacy sub-tick debt belongs to paid construction work, before
            // the new motion epoch exists. Settle it without inventing earlier
            // trajectory/scanner time or changing the migration's saved pose.
            let clock = candidate.motion.take();
            candidate.finish_offline_remainder();
            candidate.motion = clock;
        }
        if header.schema_version >= 22 {
            let elapsed = launch_utc.saturating_sub(candidate.offline.watermark);
            let total = Duration::from_secs(elapsed);
            candidate.reconcile_closed_reported(total, &[], &mut |done| {
                progress(crate::launch_progress::Progress::propagating(done, total))
            })?;
            if candidate.motion.is_some() {
                candidate.finish_offline_remainder();
            }
            if let Some(error) = &candidate.motion_error {
                return Err(invalid(format!("offline motion/scanner: {error}")));
            }
            progress(crate::launch_progress::Progress::propagating(total, total))?;
        }
        candidate.offline.watermark = candidate.offline.watermark.max(launch_utc);
        candidate
            .survival
            .recover_offline(header.saved_at_unix_seconds, launch_utc);
        candidate.enforce_survival_restrictions();
        progress(crate::launch_progress::Progress::at(
            crate::launch_progress::Stage::Verifying,
        ))?;
        let mut verified = GameplaySession::default();
        verified.restore_bytes(&candidate.save_bytes(launch_utc)?)?;
        *self = candidate;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
