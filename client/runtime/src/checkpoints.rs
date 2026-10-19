//! Native orchestration: capture on the session thread, durable IO on one worker.
pub(crate) mod control;
use crate::session::GameplaySession;
use control::Control;
use salimon_persistence::{
    SaveError,
    storage::{CheckpointStore, NativeStore, SaveManager},
};
use std::{
    path::PathBuf,
    sync::mpsc::{self, Receiver, SyncSender},
    thread::JoinHandle,
    time::{Duration, SystemTime, UNIX_EPOCH},
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

pub(crate) struct Checkpoints {
    sender: Option<SyncSender<Vec<u8>>>,
    results: Receiver<Result<(), SaveError>>,
    worker: Option<JoinHandle<()>>,
    busy: bool,
    control: Control,
    last_time: Duration,
    last_events: Option<Vec<u8>>,
    retry_at: Duration,
    pub(crate) warning: Option<&'static str>,
}
impl Checkpoints {
    pub(crate) fn open(
        directory: PathBuf,
        session: &mut GameplaySession,
    ) -> Result<Self, SaveError> {
        Self::with_store(NativeStore::open(directory)?, session)
    }
    fn with_store<S: CheckpointStore + Send + 'static>(
        store: S,
        session: &mut GameplaySession,
    ) -> Result<Self, SaveError> {
        let mut manager = SaveManager::new(store);
        let recovery = manager.recover(validate)?;
        let had_save = recovery.bytes.is_some();
        if let Some(bytes) = recovery.bytes {
            session.restore_bytes(&bytes)?;
        }
        let (sender, jobs) = mpsc::sync_channel::<Vec<u8>>(1);
        let (done, results) = mpsc::channel();
        let worker = std::thread::Builder::new()
            .name("salimon-save".into())
            .spawn(move || {
                for bytes in jobs {
                    let result = manager.checkpoint(&bytes, validate);
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
            || (!self.control.pending && session.elapsed < self.retry_at)
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
            .and_then(|now| session.save_bytes(now).map(|bytes| (now, bytes)))
            .and_then(|(now, bytes)| {
                self.sender
                    .as_ref()
                    .ok_or_else(|| salimon_persistence::invalid("save worker closed"))?
                    .try_send(bytes)
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
        } else if self.warning.is_some_and(|w| w.starts_with("SAVE FAILED")) {
            self.warning = None;
        }
    }
    /// Portable menu request admission. Capture/IO remains asynchronous and uses
    /// the same active-run store as auto saves. No game tick or offline grant.
    #[allow(dead_code)] // UI blocked by #381; exercised by IO integration tests.
    pub(crate) fn request(&mut self, request: control::Request, session: &GameplaySession) {
        self.control.request(request);
        self.poll(session);
    }
    #[allow(dead_code)] // Read by the future pause-menu presenter.
    pub(crate) fn status(&self) -> (control::Status, Option<u64>, bool) {
        (
            self.control.status,
            self.control.last_verified_utc,
            self.control.exit_ready,
        )
    }
    /// Drain the in-flight write, then verify the final snapshot before returning.
    pub(crate) fn finish(&mut self, session: &GameplaySession) -> Result<(), SaveError> {
        let bytes = now().and_then(|now| session.save_bytes(now))?;
        if self.busy {
            let _ = self.results.recv();
            self.busy = false;
        }
        let sender = self
            .sender
            .take()
            .ok_or_else(|| salimon_persistence::invalid("checkpoint worker closed"))?;
        sender
            .send(bytes)
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
        assert_eq!(restored.save_bytes(123).unwrap(), expected);
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
        let expected = session.save_bytes(1).unwrap();
        let mut saves = Checkpoints::open(dir.clone(), &mut session).unwrap();
        saves.finish(&session).unwrap();
        drop(saves);
        let mut restored = GameplaySession::default();
        let saves = Checkpoints::open(dir.clone(), &mut restored).unwrap();
        assert_eq!(restored.mining.session.fragments().len(), 500);
        assert_eq!(restored.save_bytes(1).unwrap(), expected);
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
        let expected = session.save_bytes(123).unwrap();
        saves.finish(&session).unwrap();
        drop(saves);
        let mut restored = GameplaySession::default();
        let mut saves = Checkpoints::open(dir.clone(), &mut restored).unwrap();
        assert_eq!(restored.save_bytes(123).unwrap(), expected);
        saves.finish(&restored).unwrap();
        drop(saves);
        // Third generation is slot 0; the second remains independently loadable.
        std::fs::write(dir.join("checkpoint-0.sav"), b"partial").unwrap();
        let mut recovered = GameplaySession::default();
        let saves = Checkpoints::open(dir.clone(), &mut recovered).unwrap();
        assert!(saves.warning.is_some());
        assert_eq!(recovered.save_bytes(123).unwrap(), expected);
        drop(saves);
        std::fs::remove_dir_all(dir).unwrap();
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
        assert_eq!(store.0.lock().unwrap().publishes, 2); // Bounded, not 102.
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
        assert_eq!(loaded.save_bytes(1).unwrap(), before);
        assert_eq!(reopened.status().0, Status::Idle); // No false new-session receipt.
    }
    #[test]
    fn disk_full_exit_waits_retry_works_while_game_time_is_frozen() {
        let store = Store::default();
        let mut session = GameplaySession::default();
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
        assert_eq!(loaded.save_bytes(1).unwrap(), before);
    }
}
