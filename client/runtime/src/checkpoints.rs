//! Native orchestration: capture on the session thread, durable IO on one worker.
pub(crate) mod control;
use crate::session::{GameplaySession, lifecycle::ARCHIVE_VERSION};
use control::Control;
use salimon_persistence::{
    SaveError,
    storage::{CheckpointStore, NativeStore, SaveManager},
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, SyncSender},
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn validate(bytes: &[u8]) -> Result<(), SaveError> {
    GameplaySession::default().restore_bytes(bytes)
}
fn now() -> Result<u64, SaveError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| salimon_persistence::invalid(format!("system clock: {e}")))
}
pub(crate) fn default_directory() -> Result<PathBuf, SaveError> {
    if let Some(path) = std::env::var_os("SALIMON_SAVE_DIR") {
        return Ok(PathBuf::from(path));
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    base.map(|p| p.join("salimon/active-run"))
        .ok_or_else(|| salimon_persistence::invalid("no user save directory; set SALIMON_SAVE_DIR"))
}

enum SaveJob {
    Checkpoint {
        bytes: Vec<u8>,
        archive: Option<(u64, Vec<u8>)>,
    },
    Restart {
        serial: u64,
        report: Vec<u8>,
        terminal: Vec<u8>,
        fresh: Vec<u8>,
    },
}
impl SaveJob {
    fn checkpoint(session: &GameplaySession, utc: u64) -> Result<Self, SaveError> {
        Ok(Self::Checkpoint {
            bytes: session.save_bytes(utc)?,
            archive: session
                .survival
                .snapshot()
                .terminal
                .then(|| {
                    session
                        .ended_run_bytes()
                        .map(|report| (session.run.serial, report))
                })
                .transpose()?,
        })
    }
}
pub(crate) struct Checkpoints {
    sender: Option<SyncSender<SaveJob>>,
    results: Receiver<Result<(), SaveError>>,
    worker: Option<JoinHandle<()>>,
    busy: bool,
    control: Control,
    last_time: Duration,
    last_events: Option<Vec<u8>>,
    retry_at: Duration,
    retry_wall: Option<Instant>,
    pub(crate) warning: Option<&'static str>,
}
impl Checkpoints {
    pub(crate) fn open(
        directory: PathBuf,
        session: &mut GameplaySession,
    ) -> Result<Self, SaveError> {
        Self::open_at(directory, session, now()?)
    }
    pub(crate) fn open_at(
        directory: PathBuf,
        session: &mut GameplaySession,
        launch_utc: u64,
    ) -> Result<Self, SaveError> {
        Self::with_store_at(NativeStore::open(directory)?, session, launch_utc)
    }
    #[cfg(test)]
    fn with_store<S: CheckpointStore + Send + 'static>(
        store: S,
        session: &mut GameplaySession,
    ) -> Result<Self, SaveError> {
        Self::with_store_at(store, session, 1)
    }
    fn with_store_at<S: CheckpointStore + Send + 'static>(
        store: S,
        session: &mut GameplaySession,
        launch_utc: u64,
    ) -> Result<Self, SaveError> {
        let mut manager = SaveManager::new(store);
        let recovery = manager.recover(validate)?;
        let had_save = recovery.bytes.is_some();
        if let Some(bytes) = recovery.bytes {
            let mut candidate = GameplaySession::default();
            candidate.restore_for_launch(&bytes, launch_utc)?;
            #[derive(serde::Deserialize)]
            struct Version {
                schema_version: u32,
            }
            let version: Version =
                serde_json::from_slice(&bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
            if version.schema_version == 1 {
                candidate.run.serial = manager.next_run_serial()?;
            }
            let credited = candidate.save_bytes(launch_utc)?;
            if candidate.survival.snapshot().terminal {
                manager.checkpoint_terminal(
                    ARCHIVE_VERSION,
                    candidate.run.serial,
                    &candidate.ended_run_bytes()?,
                    &credited,
                    validate,
                )?;
            } else {
                manager.checkpoint(&credited, validate)?;
            }
            *session = candidate;
        } else {
            // Corruption of both slots must not reuse an archived run identity.
            session.run.serial = manager.next_run_serial()?;
            // Bootstrap a verified checkpoint before the new run becomes playable.
            let initial = session.save_bytes(launch_utc)?;
            manager.checkpoint(&initial, validate)?;
        }
        let (sender, jobs) = mpsc::sync_channel::<SaveJob>(1);
        let (done, results) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("salimon-save".into())
            .spawn(move || {
                for job in jobs {
                    let result = match job {
                        SaveJob::Checkpoint { bytes, archive } => match archive {
                            Some((serial, report)) => manager.checkpoint_terminal(
                                ARCHIVE_VERSION,
                                serial,
                                &report,
                                &bytes,
                                validate,
                            ),
                            None => manager.checkpoint(&bytes, validate),
                        },
                        SaveJob::Restart {
                            serial,
                            report,
                            terminal,
                            fresh,
                        } => manager.replace_terminal_run(
                            ARCHIVE_VERSION,
                            serial,
                            &report,
                            &terminal,
                            &fresh,
                            validate,
                        ),
                    };
                    if done.send(result).is_err() {
                        break;
                    }
                }
            })
            .map_err(|e| salimon_persistence::invalid(format!("start checkpoint worker: {e}")))?;
        Ok(Self {
            sender: Some(sender),
            results,
            worker: Some(worker),
            busy: false,
            control: Control::default(),
            last_time: session.elapsed,
            last_events: None,
            retry_at: Duration::ZERO,
            retry_wall: None,
            warning: recovery.recovered.then_some(if had_save {
                "RECOVERED PREVIOUS CHECKPOINT - latest save damaged"
            } else {
                "NO VALID CHECKPOINT - started a new run"
            }),
        })
    }
    pub(crate) fn poll(&mut self, session: &GameplaySession) {
        if self.busy {
            match self.results.try_recv() {
                Ok(result) => self.completed(result, session),
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.busy = false;
                    self.control.completed(false);
                    self.warning = Some("SAVE FAILED - checkpoint worker stopped");
                    return;
                }
            }
        }
        if self.control.exit_ready
            || (self.control.exit_requested && self.control.status == control::Status::Failed)
            || (!self.control.pending
                && ((!session.survival.snapshot().terminal && session.elapsed < self.retry_at)
                    || (session.survival.snapshot().terminal
                        && self.retry_wall.is_some_and(|at| Instant::now() < at))))
        {
            return;
        }
        let events = session.checkpoint_events();
        if !self.control.pending
            && self.last_events.as_ref() == Some(&events)
            && session.elapsed.saturating_sub(self.last_time) < Duration::from_secs(60)
        {
            return;
        }
        match now()
            .and_then(|now| SaveJob::checkpoint(session, now).map(|job| (now, job)))
            .and_then(|(now, job)| {
                self.sender
                    .as_ref()
                    .ok_or_else(|| salimon_persistence::invalid("save worker closed"))?
                    .try_send(job)
                    .map_err(|e| salimon_persistence::invalid(format!("queue checkpoint: {e}")))
                    .map(|()| now)
            }) {
            Ok(now) => {
                self.control.started(now);
                self.busy = true;
                self.last_time = session.elapsed;
                self.last_events = Some(events);
            }
            Err(error) => {
                self.control.completed(false);
                log::error!("capture checkpoint failed: {error}");
                self.warning = Some("SAVE FAILED - session could not be captured");
                self.retry_at = session.elapsed.saturating_add(Duration::from_secs(5));
                self.retry_wall = Some(Instant::now() + Duration::from_secs(5));
            }
        }
    }
    fn completed(&mut self, result: Result<(), SaveError>, session: &GameplaySession) {
        self.busy = false;
        self.control.completed(result.is_ok());
        if let Err(error) = result {
            log::error!("checkpoint failed: {error}");
            self.warning = Some("SAVE FAILED - check storage and permissions");
            self.last_events = None;
            self.retry_at = session.elapsed.saturating_add(Duration::from_secs(5));
            self.retry_wall = Some(Instant::now() + Duration::from_secs(5));
        } else if self.warning.is_some_and(|w| w.starts_with("SAVE FAILED")) {
            self.warning = None;
        }
    }
    /// Portable menu request admission. Capture/IO remains asynchronous and uses
    /// the same active-run store as auto saves. No game tick or offline grant.
    pub(crate) fn request(&mut self, request: control::Request, session: &GameplaySession) {
        self.control.request(request);
        self.poll(session);
    }
    pub(crate) fn status(&self) -> (control::Status, Option<u64>, bool) {
        (
            self.control.status,
            self.control.last_verified_utc,
            self.control.exit_ready,
        )
    }
    pub(crate) fn exit_requested(&self) -> bool {
        self.control.exit_requested
    }
    pub(crate) fn restart(&mut self, session: &mut GameplaySession) -> Result<(), SaveError> {
        if !session.survival.snapshot().terminal {
            return Err(salimon_persistence::invalid("restart requires death"));
        }
        let fresh = session.fresh_after_death()?;
        let utc = now()?;
        let job = SaveJob::Restart {
            serial: session.run.serial,
            report: session.ended_run_bytes()?,
            terminal: session.save_bytes(utc)?,
            fresh: fresh.save_bytes(utc)?,
        };
        if self.busy {
            let result = self
                .results
                .recv()
                .map_err(|e| salimon_persistence::invalid(format!("drain checkpoint: {e}")));
            self.busy = false;
            result??;
        }
        self.sender
            .as_ref()
            .ok_or_else(|| salimon_persistence::invalid("save worker closed"))?
            .send(job)
            .map_err(|e| salimon_persistence::invalid(format!("queue restart: {e}")))?;
        self.results
            .recv()
            .map_err(|e| salimon_persistence::invalid(format!("verify restart: {e}")))??;
        *session = fresh;
        self.last_time = session.elapsed;
        self.last_events = Some(session.checkpoint_events());
        self.retry_at = Duration::ZERO;
        self.retry_wall = None;
        self.control = Control::default();
        self.warning = None;
        Ok(())
    }
    /// Drain the in-flight write, then verify the final snapshot before returning.
    pub(crate) fn finish(&mut self, session: &GameplaySession) -> Result<(), SaveError> {
        let job = now().and_then(|now| SaveJob::checkpoint(session, now))?;
        if self.busy {
            let _ = self.results.recv();
            self.busy = false;
        }
        let sender = self
            .sender
            .take()
            .ok_or_else(|| salimon_persistence::invalid("checkpoint worker closed"))?;
        sender
            .send(job)
            .map_err(|e| salimon_persistence::invalid(format!("queue exit checkpoint: {e}")))?;
        drop(sender);
        let result = self
            .results
            .recv()
            .map_err(|e| salimon_persistence::invalid(format!("verify exit checkpoint: {e}")))?;
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .map_err(|_| salimon_persistence::invalid("checkpoint worker panicked"))?;
        }
        result
    }
}
impl Drop for Checkpoints {
    fn drop(&mut self) {
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn logical(bytes: Vec<u8>) -> serde_json::Value {
        let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        value["state"]["survival"]["vitals"]
            .as_object_mut()
            .unwrap()
            .remove("offline_watermark");
        value
    }
    #[test]
    fn durable_restore_closes_q_wheel_and_clears_primary_without_consumption() {
        use crate::session::{Command, consumable_wheel::WheelAction};
        let dir = std::env::temp_dir().join(format!("salimon-wheel-save-{}", std::process::id()));
        let mut session = GameplaySession::default();
        let expected = session.save_bytes(123).unwrap();
        let events = session.checkpoint_events();
        session.command(Command::Wheel(WheelAction::Hold(true)));
        session.command(Command::Wheel(WheelAction::Hover(Some(
            salimon_fabrication::ConsumableSlot::ALL[0],
        ))));
        session.command(Command::Wheel(WheelAction::Primary(true)));
        assert!(session.wheel.open && session.wheel.primary_down);
        assert_eq!(session.checkpoint_events(), events);
        let mut saves = Checkpoints::open(dir.clone(), &mut session).unwrap();
        saves.finish(&session).unwrap();
        drop(saves);
        let mut restored = GameplaySession::default();
        let saves = Checkpoints::open(dir.clone(), &mut restored).unwrap();
        assert!(!restored.wheel.open && !restored.wheel.held && !restored.wheel.primary_down);
        assert!(restored.wheel.hovered.is_none());
        assert_eq!(
            logical(restored.save_bytes(123).unwrap()),
            logical(expected)
        );
        drop(saves);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn periodic_boundary_and_discrete_events_schedule_without_waiting_for_render_time() {
        let dir = std::env::temp_dir().join(format!("salimon-timer-save-{}", std::process::id()));
        let mut session = GameplaySession::default();
        let mut saves = Checkpoints::open(dir.clone(), &mut session).unwrap();
        saves.poll(&session);
        saves.results.recv().unwrap().unwrap();
        saves.busy = false;
        session.elapsed = Duration::from_millis(59_999);
        saves.poll(&session);
        assert!(!saves.busy);
        session.elapsed = Duration::from_secs(60);
        saves.poll(&session);
        assert!(saves.busy);
        saves.results.recv().unwrap().unwrap();
        saves.busy = false;
        session
            .equipment
            .select(crate::equipment::ToolbarSlot::Three, false);
        saves.poll(&session);
        assert!(saves.busy);
        saves.finish(&session).unwrap();
        drop(saves);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn five_hundred_fragment_native_checkpoint_preserves_ids_mass_and_clocks() {
        let dir = std::env::temp_dir().join(format!("salimon-500-save-{}", std::process::id()));
        let mut session = GameplaySession::default();
        crate::benchmark_fixture::initialize(
            &mut session,
            500,
            crate::benchmark_fixture::Layout::Dense,
            366,
            false,
        )
        .unwrap();
        session.update_clock.accumulate(Duration::from_millis(7));
        let expected = logical(session.save_bytes(1).unwrap());
        let launch = now().unwrap();
        let mut saves = Checkpoints::open(dir.clone(), &mut session).unwrap();
        saves.finish(&session).unwrap();
        drop(saves);
        let mut restored = GameplaySession::default();
        let saves = Checkpoints::open_at(dir.clone(), &mut restored, launch).unwrap();
        assert_eq!(restored.mining.session.fragments().len(), 500);
        assert_eq!(logical(restored.save_bytes(1).unwrap()), expected);
        drop(saves);
        std::fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn startup_exit_roundtrip_and_corrupted_latest_fallback() {
        let dir = std::env::temp_dir().join(format!("salimon-runtime-save-{}", std::process::id()));
        let mut session = GameplaySession::default();
        let mut saves = Checkpoints::open(dir.clone(), &mut session).unwrap();
        saves.poll(&session);
        session.advance_automation(Duration::from_secs(1));
        let expected = logical(session.save_bytes(123).unwrap());
        let launch = now().unwrap();
        saves.finish(&session).unwrap();
        drop(saves);
        let mut restored = GameplaySession::default();
        let mut saves = Checkpoints::open_at(dir.clone(), &mut restored, launch).unwrap();
        assert_eq!(logical(restored.save_bytes(123).unwrap()), expected);
        saves.finish(&restored).unwrap();
        drop(saves);
        // Latest generation is slot 0 after bootstrap, launch-credit and exit writes.
        std::fs::write(dir.join("checkpoint-0.sav"), b"partial").unwrap();
        let mut recovered = GameplaySession::default();
        let saves = Checkpoints::open_at(dir.clone(), &mut recovered, launch).unwrap();
        assert!(saves.warning.is_some());
        assert_eq!(logical(recovered.save_bytes(123).unwrap()), expected);
        drop(saves);
        std::fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use salimon_persistence::storage::CheckpointStore;
    use salimon_survival::{Command, Points};
    fn dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("salimon-371-{name}-{}", std::process::id()))
    }
    fn reduced() -> GameplaySession {
        let mut s = GameplaySession::default();
        let mut v = serde_json::to_value(&s.survival).unwrap();
        v["fatigue"] = serde_json::to_value(Points::whole(20).unwrap()).unwrap();
        s.survival = serde_json::from_value(v).unwrap();
        s.survival
            .command(Command::Damage(Points::whole(40).unwrap()))
            .unwrap();
        s.survival
            .apply_consumable(s.consumable_effects.get("medkit").unwrap())
            .unwrap();
        s.advance_automation(Duration::from_millis(1375));
        s
    }
    #[test]
    fn launch_commits_only_fatigue_credit_once_and_preserves_world_healing_and_rollback_watermark()
    {
        let directory = dir("offline");
        let _ = std::fs::remove_dir_all(&directory);
        let original = reduced();
        let mut manager = SaveManager::new(NativeStore::open(directory.clone()).unwrap());
        manager
            .checkpoint(&original.save_bytes(100).unwrap(), validate)
            .unwrap();
        drop(manager);
        let mut restored = GameplaySession::default();
        let saves = Checkpoints::open_at(directory.clone(), &mut restored, 7300).unwrap();
        assert_eq!(
            restored.survival.snapshot().health,
            original.survival.snapshot().health
        );
        assert_eq!(
            restored.survival.snapshot().hunger,
            original.survival.snapshot().hunger
        );
        assert_eq!(
            restored.survival.active_effects().collect::<Vec<_>>(),
            original.survival.active_effects().collect::<Vec<_>>()
        );
        assert_eq!(restored.snapshot().ship, original.snapshot().ship);
        assert!(restored.survival.snapshot().fatigue.value() > 53.0);
        let credited = restored.survival.snapshot().fatigue;
        drop(saves);
        let saves = Checkpoints::open_at(directory.clone(), &mut restored, 7300).unwrap();
        assert_eq!(restored.survival.snapshot().fatigue, credited);
        drop(saves);
        let saves = Checkpoints::open_at(directory.clone(), &mut restored, 1).unwrap();
        assert_eq!(restored.survival.snapshot().fatigue, credited);
        drop(saves);
        let saves = Checkpoints::open_at(directory.clone(), &mut restored, 2).unwrap();
        assert_eq!(restored.survival.snapshot().fatigue, credited);
        drop(saves);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn terminal_native_restart_archives_once_and_corrupt_latest_cannot_restore_old_run() {
        let directory = dir("restart");
        let _ = std::fs::remove_dir_all(&directory);
        let mut s = reduced();
        let mut saves = Checkpoints::open_at(directory.clone(), &mut s, 100).unwrap();
        s.survival.command(Command::Damage(Points::FULL)).unwrap();
        s.advance_automation(Duration::from_secs(1));
        let serial = s.run.serial;
        let report = s.ended_run_bytes().unwrap();
        saves.restart(&mut s).unwrap();
        assert_eq!(s.run.serial, serial + 1);
        assert_eq!(s.survival.snapshot().health, Points::FULL);
        assert_eq!(
            std::fs::read(directory.join(format!("journeys/v2/run-{serial}.json"))).unwrap(),
            report
        );
        drop(saves);
        // Both slots contain the fresh run, regardless of which was the final generation.
        for slot in 0..2 {
            let copy = dir(&format!("fallback-{slot}"));
            let _ = std::fs::remove_dir_all(&copy);
            std::fs::create_dir_all(&copy).unwrap();
            for i in 0..2 {
                std::fs::copy(
                    directory.join(format!("checkpoint-{i}.sav")),
                    copy.join(format!("checkpoint-{i}.sav")),
                )
                .unwrap();
            }
            std::fs::write(copy.join(format!("checkpoint-{slot}.sav")), b"bad").unwrap();
            let mut candidate = GameplaySession::default();
            let reopened = Checkpoints::open_at(copy.clone(), &mut candidate, 100).unwrap();
            assert_eq!(candidate.run.serial, serial + 1);
            assert!(!candidate.survival.snapshot().terminal);
            drop(reopened);
            std::fs::remove_dir_all(copy).unwrap();
        }
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn death_checkpoint_archives_before_restart_and_launch_repairs_missing_report() {
        let directory = dir("death-archive");
        let _ = std::fs::remove_dir_all(&directory);
        let mut s = reduced();
        let mut saves = Checkpoints::open_at(directory.clone(), &mut s, 100).unwrap();
        s.survival.command(Command::Damage(Points::FULL)).unwrap();
        let report = s.ended_run_bytes().unwrap();
        saves.poll(&s);
        saves.results.recv().unwrap().unwrap();
        saves.busy = false;
        let path = directory.join(format!("journeys/v2/run-{}.json", s.run.serial));
        assert_eq!(std::fs::read(&path).unwrap(), report);
        assert!(s.survival.snapshot().terminal);
        // Model interruption after both terminal writes but before archive rename.
        std::fs::remove_file(&path).unwrap();
        drop(saves);
        let mut recovered = GameplaySession::default();
        let saves = Checkpoints::open_at(directory.clone(), &mut recovered, 100).unwrap();
        assert!(recovered.survival.snapshot().terminal);
        assert_eq!(std::fs::read(&path).unwrap(), report);
        drop(saves);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn report_versions_are_immutable_and_both_namespaces_reserve_run_ids() {
        let directory = dir("archive-versions");
        let _ = std::fs::remove_dir_all(&directory);
        let mut store = NativeStore::open(directory.clone()).unwrap();
        store.archive_run(1, 7, b"legacy").unwrap();
        store.archive_run(2, 7, b"current").unwrap();
        store.archive_run(2, 9, b"latest").unwrap();
        assert_eq!(
            std::fs::read(directory.join("journeys/run-7.json")).unwrap(),
            b"legacy"
        );
        assert_eq!(
            std::fs::read(directory.join("journeys/v2/run-7.json")).unwrap(),
            b"current"
        );
        assert!(store.archive_run(2, 7, b"conflict").is_err());
        assert_eq!(store.next_run_serial().unwrap(), 10);
        drop(store);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn archive_is_idempotent_conflicting_report_rejects_and_new_run_serial_avoids_history() {
        let directory = dir("archive");
        let _ = std::fs::remove_dir_all(&directory);
        let mut store = NativeStore::open(directory.clone()).unwrap();
        store.archive_run(1, 7, b"report").unwrap();
        store.archive_run(1, 7, b"report").unwrap();
        assert!(store.archive_run(1, 7, b"different").is_err());
        assert_eq!(store.next_run_serial().unwrap(), 8);
        drop(store);
        let mut s = GameplaySession::default();
        let saves = Checkpoints::open_at(directory.clone(), &mut s, 100).unwrap();
        assert_eq!(s.run.serial, 8);
        drop(saves);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod publication_failure_tests {
    use super::*;
    #[test]
    fn failed_offline_credit_publication_does_not_expose_candidate_or_destroy_prior_checkpoint() {
        let directory =
            std::env::temp_dir().join(format!("salimon-371-credit-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let mut original = GameplaySession::default();
        let mut vitals = serde_json::to_value(&original.survival).unwrap();
        vitals["fatigue"] =
            serde_json::to_value(salimon_survival::Points::whole(20).unwrap()).unwrap();
        original.survival = serde_json::from_value(vitals).unwrap();
        let mut manager = SaveManager::new(NativeStore::open(directory.clone()).unwrap());
        manager
            .checkpoint(&original.save_bytes(100).unwrap(), validate)
            .unwrap();
        drop(manager);
        std::fs::create_dir(directory.join("checkpoint-1.tmp")).unwrap();
        let mut visible = GameplaySession::default();
        let before = visible.save_bytes(100).unwrap();
        assert!(Checkpoints::open_at(directory.clone(), &mut visible, 1000).is_err());
        assert!(visible.save_bytes(100).unwrap() == before);
        let mut manager = SaveManager::new(NativeStore::open(directory.clone()).unwrap());
        assert_eq!(
            manager.recover(validate).unwrap().bytes.unwrap(),
            original.save_bytes(100).unwrap()
        );
        drop(manager);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn terminal_checkpoint_retry_uses_host_time_while_gameplay_clock_is_frozen() {
        let directory =
            std::env::temp_dir().join(format!("salimon-371-retry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let mut s = GameplaySession::default();
        let mut saves = Checkpoints::open_at(directory.clone(), &mut s, 100).unwrap();
        s.survival
            .command(salimon_survival::Command::Damage(
                salimon_survival::Points::FULL,
            ))
            .unwrap();
        saves.retry_at = Duration::from_secs(5);
        saves.retry_wall = Some(Instant::now() + Duration::from_secs(5));
        saves.poll(&s);
        assert!(!saves.busy);
        saves.retry_wall = Some(Instant::now() - Duration::from_secs(1));
        saves.poll(&s);
        assert!(saves.busy);
        saves.results.recv().unwrap().unwrap();
        saves.busy = false;
        drop(saves);
        std::fs::remove_dir_all(directory).unwrap();
    }
}

#[cfg(test)]
mod manual_tests {
    use super::*;
    use control::{Request, Status};
    use std::sync::{Arc, Mutex};
    #[derive(Default)]
    struct Disk {
        slots: [Option<Vec<u8>>; 2],
        full: bool,
        corrupt: bool,
        publishes: usize,
    }
    #[derive(Clone, Default)]
    struct Store(Arc<Mutex<Disk>>);
    impl CheckpointStore for Store {
        fn read(&self, slot: usize) -> Result<Option<Vec<u8>>, SaveError> {
            Ok(self.0.lock().unwrap().slots[slot].clone())
        }
        fn publish(&mut self, slot: usize, bytes: &[u8]) -> Result<(), SaveError> {
            let mut disk = self.0.lock().unwrap();
            if disk.full {
                return Err(salimon_persistence::invalid("disk full (injected)"));
            }
            disk.publishes += 1;
            disk.slots[slot] = Some(if disk.corrupt {
                b"partial".to_vec()
            } else {
                bytes.to_vec()
            });
            Ok(())
        }
    }
    fn drain(saves: &mut Checkpoints, session: &GameplaySession) {
        // Block only in tests, with a finite deadline; production poll never waits.
        while saves.busy {
            let result = saves.results.recv_timeout(Duration::from_secs(5)).unwrap();
            saves.completed(result, session);
            saves.poll(session);
        }
    }
    #[test]
    fn stationary_save_and_exit_coalesces_and_saves_latest_state_without_a_tick() {
        let store = Store::default();
        let mut session = GameplaySession::default();
        session.toggle_pause();
        let mut saves = Checkpoints::with_store(store.clone(), &mut session).unwrap();
        saves.poll(&session); // Start an older auto checkpoint.
        session
            .equipment
            .select(crate::equipment::ToolbarSlot::Three, false);
        for _ in 0..100 {
            saves.control.request(Request::SaveNow);
        }
        saves.control.request(Request::SaveAndExit);
        let before = session.save_bytes(1).unwrap();
        drain(&mut saves, &session);
        assert_eq!(session.save_bytes(1).unwrap(), before);
        assert!(saves.status().2);
        assert_eq!(store.0.lock().unwrap().publishes, 3); // Bounded, not 102.
        let verified_utc = saves.status().1.unwrap();
        assert_eq!(
            saves.status().0,
            Status::Saved {
                checkpoint_utc: verified_utc
            }
        );
        drop(saves);
        let mut loaded = GameplaySession::default();
        let reopened = Checkpoints::with_store(store, &mut loaded).unwrap();
        assert_eq!(
            super::tests::logical(loaded.save_bytes(1).unwrap()),
            super::tests::logical(before)
        );
        assert_eq!(reopened.status().0, Status::Idle); // No false new-session receipt.
    }
    #[test]
    fn disk_full_exit_waits_retry_works_while_game_time_is_frozen() {
        let store = Store::default();
        let mut session = GameplaySession::default();
        session.toggle_pause();
        let mut saves = Checkpoints::with_store(store.clone(), &mut session).unwrap();
        saves.request(Request::SaveNow, &session);
        drain(&mut saves, &session);
        let verified = saves.status().1;
        store.0.lock().unwrap().full = true;
        saves.request(Request::SaveAndExit, &session);
        drain(&mut saves, &session);
        assert_eq!(saves.status(), (Status::Failed, verified, false));
        assert!(saves.warning.is_some());
        for _ in 0..100 {
            saves.poll(&session);
            assert!(!saves.busy);
        }
        store.0.lock().unwrap().full = false;
        saves.request(Request::Retry, &session);
        drain(&mut saves, &session);
        assert!(saves.status().2);
        assert!(saves.warning.is_none());
        assert_eq!(session.elapsed, Duration::ZERO);
    }
    #[test]
    fn corrupt_write_keeps_previous_generation_and_explicit_risk_exit_is_not_saved() {
        let store = Store::default();
        let mut session = GameplaySession::default();
        session.toggle_pause();
        let mut saves = Checkpoints::with_store(store.clone(), &mut session).unwrap();
        saves.request(Request::SaveNow, &session);
        drain(&mut saves, &session);
        let before = session.save_bytes(1).unwrap();
        let verified = saves.status().1;
        session
            .equipment
            .select(crate::equipment::ToolbarSlot::Four, false);
        store.0.lock().unwrap().corrupt = true;
        saves.request(Request::SaveAndExit, &session);
        drain(&mut saves, &session);
        assert_eq!(saves.status(), (Status::Failed, verified, false));
        saves.request(Request::ExitWithRisk, &session);
        assert_eq!(saves.status(), (Status::Failed, verified, true));
        drop(saves);
        store.0.lock().unwrap().corrupt = false;
        let mut loaded = GameplaySession::default();
        let recovered = Checkpoints::with_store(store, &mut loaded).unwrap();
        assert!(recovered.warning.is_some());
        assert_eq!(
            super::tests::logical(loaded.save_bytes(1).unwrap()),
            super::tests::logical(before)
        );
    }
}
