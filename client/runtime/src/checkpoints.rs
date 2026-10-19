//! Native orchestration: capture on the session thread, durable IO on one worker.
use crate::session::GameplaySession;
use salimon_persistence::{
    SaveError,
    storage::{NativeStore, SaveManager},
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
        let mut manager = SaveManager::new(NativeStore::open(directory)?);
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
                Ok(result) => {
                    self.busy = false;
                    if let Err(error) = result {
                        log::error!("checkpoint failed: {error}");
                        self.warning = Some("SAVE FAILED - check storage and permissions");
                        self.last_events = None;
                        self.retry_at = session.elapsed.saturating_add(Duration::from_secs(5));
                    }
                }
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.warning = Some("SAVE FAILED - checkpoint worker stopped");
                    return;
                }
            }
        }
        if session.elapsed < self.retry_at {
            return;
        }
        let events = session.checkpoint_events();
        if self.last_events.as_ref() == Some(&events)
            && session.elapsed.saturating_sub(self.last_time) < Duration::from_secs(60)
        {
            return;
        }
        match now()
            .and_then(|now| session.save_bytes(now))
            .and_then(|bytes| {
                self.sender
                    .as_ref()
                    .ok_or_else(|| salimon_persistence::invalid("save worker closed"))?
                    .try_send(bytes)
                    .map_err(|e| salimon_persistence::invalid(format!("queue checkpoint: {e}")))
            }) {
            Ok(()) => {
                self.busy = true;
                self.last_time = session.elapsed;
                self.last_events = Some(events);
            }
            Err(error) => {
                log::error!("capture checkpoint failed: {error}");
                self.warning = Some("SAVE FAILED - session could not be captured");
                self.retry_at = session.elapsed.saturating_add(Duration::from_secs(5));
            }
        }
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
