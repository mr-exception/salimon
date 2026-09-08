//! Native asynchronous launch. Presentation/admission stay on the main thread;
//! one worker owns candidate reconstruction, catch-up and verified publication.
use crate::{
    checkpoints::Checkpoints,
    launch_progress::{Progress, Stage},
    session::GameplaySession,
};
use salimon_persistence::{SaveError, invalid};
use std::{
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
// Domain sessions deliberately contain thread-local Rc query caches. Transfer
// only the verified source bytes, reconstructing caches on the native thread.
type ResultMessage = Result<(Checkpoints, Vec<u8>), SaveError>;
pub(crate) struct Launch {
    directory: PathBuf,
    progress: Arc<Mutex<Progress>>,
    cancel: Arc<AtomicBool>,
    results: Receiver<ResultMessage>,
    worker: Option<JoinHandle<()>>,
    error: Option<String>,
    user_error: Option<String>,
    ready: bool,
    received: bool,
    fixture_gate: Option<Arc<(Mutex<bool>, Condvar)>>,
}
impl Launch {
    pub(crate) fn start(directory: PathBuf) -> Result<Self, SaveError> {
        Self::begin(directory, None, false)
    }
    /// Explicit controlled-clock E2E fixture; ordinary startup always uses now().
    pub(crate) fn start_at(
        directory: PathBuf,
        utc: u64,
        hold_fixture: bool,
    ) -> Result<Self, SaveError> {
        Self::begin(directory, Some(utc), hold_fixture)
    }
    fn begin(directory: PathBuf, utc: Option<u64>, hold_fixture: bool) -> Result<Self, SaveError> {
        let progress = Arc::new(Mutex::new(Progress::at(Stage::Reading)));
        let cancel = Arc::new(AtomicBool::new(false));
        let gate = hold_fixture.then(|| Arc::new((Mutex::new(false), Condvar::new())));
        let (send, results) = mpsc::channel();
        let state = Arc::clone(&progress);
        let canceled = Arc::clone(&cancel);
        let work_gate = gate.clone();
        let path = directory.clone();
        let worker = std::thread::Builder::new()
            .name("salimon-launch".into())
            .spawn(move || {
                let result = (|| {
                    let utc = match utc {
                        Some(utc) => utc,
                        None => crate::checkpoints::now()?,
                    };
                    let mut session = GameplaySession::new_canonical_run().map_err(invalid)?;
                    let checkpoints =
                        Checkpoints::open_reported_at(path, &mut session, utc, &mut |p| {
                            if canceled.load(Ordering::Acquire) {
                                return Err(invalid("launch canceled before publication"));
                            }
                            *state
                                .lock()
                                .map_err(|_| invalid("launch progress unavailable"))? = p;
                            if p.stage == Stage::Propagating
                                && !p.completed.is_zero()
                                && let Some(gate) = &work_gate
                            {
                                let (lock, changed) = &**gate;
                                let mut released = lock
                                    .lock()
                                    .map_err(|_| invalid("E2E launch gate unavailable"))?;
                                let deadline = Instant::now() + Duration::from_secs(30);
                                while !*released {
                                    if canceled.load(Ordering::Acquire) {
                                        return Err(invalid("launch canceled before publication"));
                                    }
                                    if Instant::now() >= deadline {
                                        return Err(invalid("E2E launch gate timed out"));
                                    }
                                    released = changed
                                        .wait_timeout(released, Duration::from_millis(10))
                                        .map_err(|_| invalid("E2E launch gate unavailable"))?
                                        .0;
                                }
                            }
                            Ok(())
                        })?;
                    Ok((checkpoints, session.save_bytes(utc)?))
                })();
                let _ = send.send(result);
            })
            .map_err(|e| invalid(format!("start launch worker: {e}")))?;
        Ok(Self {
            directory,
            progress,
            cancel,
            results,
            worker: Some(worker),
            error: None,
            user_error: None,
            ready: false,
            received: false,
            fixture_gate: gate,
        })
    }
    pub(crate) fn directory(&self) -> PathBuf {
        self.directory.clone()
    }
    pub(crate) fn is_ready(&self) -> bool {
        self.ready
    }
    pub(crate) fn failed(&self) -> bool {
        self.error.is_some()
    }
    pub(crate) fn progress(&self) -> Progress {
        self.progress
            .lock()
            .map(|p| *p)
            .unwrap_or(Progress::at(Stage::Failed))
    }
    pub(crate) fn menu(&self) -> String {
        self.progress().menu(self.user_error.as_deref())
    }
    pub(crate) fn observation(&self) -> serde_json::Value {
        serde_json::json!({"progress":self.progress(),"percentage":self.progress().percentage(),"ready":self.ready,"error":self.error})
    }
    pub(crate) fn release_fixture(&self) -> bool {
        let Some(gate) = &self.fixture_gate else {
            return false;
        };
        let (lock, changed) = &**gate;
        if let Ok(mut released) = lock.lock() {
            *released = true;
            changed.notify_all();
            true
        } else {
            false
        }
    }
    pub(crate) fn poll(&mut self) -> Option<(Checkpoints, GameplaySession)> {
        if self.received {
            return None;
        }
        let result = match self.results.try_recv() {
            Ok(result) => result,
            Err(mpsc::TryRecvError::Empty) => return None,
            Err(mpsc::TryRecvError::Disconnected) => Err(invalid(
                "launch worker stopped; checkpoint remains recoverable",
            )),
        };
        self.received = true;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        let result = result.and_then(|(checkpoints, bytes)| {
            let mut session = GameplaySession::default();
            session.restore_bytes(&bytes)?;
            Ok((checkpoints, session))
        });
        match result {
            Ok(value) => {
                self.ready = true;
                if let Ok(mut p) = self.progress.lock() {
                    *p = Progress::at(Stage::Ready);
                }
                Some(value)
            }
            Err(error) => {
                self.error = Some(error.to_string());
                self.user_error=Some(match error {
                    SaveError::Invalid(message)|SaveError::IncompatibleCatalog(message)=>message,
                    SaveError::UnsupportedVersion(_)=>"This journey was saved by a different game version. Open it with a compatible version and retry.".into(),
                    SaveError::Encoding(_)=>"The saved journey could not be read. Restore a valid checkpoint and retry.".into(),
                });
                if let Ok(mut p) = self.progress.lock() {
                    p.stage = Stage::Failed;
                }
                None
            }
        }
    }
}
impl Drop for Launch {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(gate) = &self.fixture_gate {
            gate.1.notify_all();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_persistence::storage::{NativeStore, SaveManager};
    use std::sync::atomic::{AtomicU64, Ordering};
    fn directory(tag: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        std::env::temp_dir().join(format!(
            "salimon-launch-{tag}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn seed(path: &std::path::Path) -> Vec<u8> {
        let mut s = GameplaySession::new_canonical_run().unwrap();
        s.bind_scanner_clock(100).unwrap();
        let bytes = s.save_bytes(100).unwrap();
        let mut manager = SaveManager::new(NativeStore::open(path.to_path_buf()).unwrap());
        manager
            .checkpoint(&bytes, |b| GameplaySession::default().restore_bytes(b))
            .unwrap();
        bytes
    }
    fn finish(job: &mut Launch) -> Option<(Checkpoints, GameplaySession)> {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(result) = job.poll() {
                return Some(result);
            }
            if job.failed() {
                return None;
            }
            assert!(Instant::now() < deadline, "launch worker did not settle");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    #[test]
    fn asynchronous_result_is_ready_only_after_verified_durable_replacement() {
        let path = directory("ready");
        seed(&path);
        let mut job = Launch::start_at(path.clone(), 160, false).unwrap();
        let (checkpoints, s) = finish(&mut job).unwrap();
        assert!(job.is_ready());
        assert_eq!(job.progress().stage, Stage::Ready);
        assert_eq!(s.offline.watermark, 160);
        assert_eq!(s.scanner.utc_millis, 160_000);
        assert!(!s.ship.snapshot().engines_enabled);
        let expected = s.save_bytes(160).unwrap();
        drop(checkpoints);
        drop(job);
        let recovered = SaveManager::new(NativeStore::open(path.clone()).unwrap())
            .recover(|b| GameplaySession::default().restore_bytes(b))
            .unwrap()
            .bytes
            .unwrap();
        assert_eq!(recovered, expected);
        let slots = std::fs::read_dir(&path)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "sav"))
            .count();
        assert_eq!(slots, 2);
        assert!(!path.join("checkpoint-0.tmp").exists());
        assert!(!path.join("checkpoint-1.tmp").exists());
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn cancel_during_progress_preserves_original_checkpoint_and_restart_credits_once() {
        let path = directory("cancel");
        let original = seed(&path);
        let job = Launch::start_at(path.clone(), 160, true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while job.progress().completed.is_zero() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(!job.is_ready());
        assert!(job.menu().contains("Resume unavailable"));
        drop(job);
        let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
        assert_eq!(
            manager
                .recover(|b| GameplaySession::default().restore_bytes(b))
                .unwrap()
                .bytes
                .unwrap(),
            original
        );
        drop(manager);
        let mut restarted = Launch::start_at(path.clone(), 160, false).unwrap();
        let (host, s) = finish(&mut restarted).unwrap();
        let once = s.save_bytes(160).unwrap();
        drop(host);
        drop(restarted);
        let mut again = Launch::start_at(path.clone(), 160, false).unwrap();
        let (host, s) = finish(&mut again).unwrap();
        assert_eq!(s.save_bytes(160).unwrap(), once);
        drop(host);
        drop(again);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn backward_or_invalid_clock_blocks_resume_and_keeps_both_slots() {
        let path = directory("clock");
        let original = seed(&path);
        for utc in [99, u64::MAX] {
            let mut job = Launch::start_at(path.clone(), utc, false).unwrap();
            assert!(finish(&mut job).is_none());
            assert!(!job.is_ready());
            assert!(job.failed());
            assert!(job.menu().contains("Resume unavailable"));
            drop(job);
            let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
            assert_eq!(
                manager
                    .recover(|b| GameplaySession::default().restore_bytes(b))
                    .unwrap()
                    .bytes
                    .unwrap(),
                original
            );
        }
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn storage_setup_failure_has_an_actionable_retry_and_no_ready_state() {
        let path = directory("storage");
        std::fs::write(&path, b"blocked directory").unwrap();
        let mut job = Launch::start_at(path.clone(), 100, false).unwrap();
        assert!(finish(&mut job).is_none());
        assert!(job.menu().contains("RETRY"));
        drop(job);
        std::fs::remove_file(&path).unwrap();
        let mut retry = Launch::start_at(path.clone(), 100, false).unwrap();
        let (host, _) = finish(&mut retry).unwrap();
        assert!(retry.is_ready());
        drop(host);
        drop(retry);
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[cfg(test)]
mod real_clock_tests {
    use super::*;
    use salimon_persistence::storage::{NativeStore, SaveManager};
    #[test]
    fn actual_local_clock_absence_is_distinct_from_controlled_elapsed_fixtures() {
        let path =
            std::env::temp_dir().join(format!("salimon-434-real-clock-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        let saved = crate::checkpoints::now().unwrap();
        let mut source = GameplaySession::new_canonical_run().unwrap();
        source.bind_scanner_clock(saved).unwrap();
        let original_epoch = source.motion.as_ref().unwrap().clone();
        let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
        manager
            .checkpoint(&source.save_bytes(saved).unwrap(), |b| {
                GameplaySession::default().restore_bytes(b)
            })
            .unwrap();
        drop(manager);
        std::thread::sleep(Duration::from_millis(1100));
        let mut job = Launch::start(path.clone()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        let (host, restored) = loop {
            if let Some(value) = job.poll() {
                break value;
            }
            assert!(!job.failed());
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(2));
        };
        let elapsed = restored.offline.watermark - saved;
        assert!((1..=10).contains(&elapsed));
        assert_eq!(
            restored.motion.unwrap(),
            original_epoch
                .advanced(Duration::from_secs(elapsed))
                .unwrap()
        );
        assert_eq!(restored.scanner.utc_millis, (saved + elapsed) * 1000);
        drop(host);
        drop(job);
        std::fs::remove_dir_all(path).unwrap();
    }
}

#[cfg(test)]
mod incompatible_tests {
    use super::*;
    use salimon_persistence::storage::{NativeStore, SaveManager};
    #[test]
    fn unknown_motion_or_world_publication_never_bootstraps_over_existing_generations() {
        for pointer in [
            "/state/motion/definition_sha256",
            "/state/physical/canonical_definition/sha256",
        ] {
            let path = std::env::temp_dir().join(format!(
                "salimon-434-incompatible-{}-{}",
                std::process::id(),
                if pointer.contains("motion") { 0 } else { 1 }
            ));
            let _ = std::fs::remove_dir_all(&path);
            let mut source = GameplaySession::new_canonical_run().unwrap();
            source.bind_scanner_clock(100).unwrap();
            let mut value: serde_json::Value =
                serde_json::from_slice(&source.save_bytes(100).unwrap()).unwrap();
            *value.pointer_mut(pointer).unwrap() = "a".repeat(64).into();
            // A future binary can publish integrity-valid bytes for another
            // model; CRC validity never authorizes this binary to replace them.
            let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
            manager
                .checkpoint(&serde_json::to_vec(&value).unwrap(), |_| Ok(()))
                .unwrap();
            manager
                .checkpoint(&serde_json::to_vec(&value).unwrap(), |_| Ok(()))
                .unwrap();
            drop(manager);
            let slots: Vec<_> = (0..2)
                .map(|i| std::fs::read(path.join(format!("checkpoint-{i}.sav"))).unwrap())
                .collect();
            let mut job = Launch::start_at(path.clone(), 160, false).unwrap();
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                assert!(job.poll().is_none());
                if job.failed() {
                    break;
                }
                assert!(Instant::now() < deadline);
                std::thread::sleep(Duration::from_millis(2));
            }
            assert!(!job.is_ready());
            assert!(
                job.user_error
                    .as_deref()
                    .unwrap()
                    .contains("compatible game version")
            );
            assert!(job.menu().contains("RETRY"));
            drop(job);
            for (i, original) in slots.iter().enumerate() {
                assert_eq!(
                    &std::fs::read(path.join(format!("checkpoint-{i}.sav"))).unwrap(),
                    original
                );
            }
            std::fs::remove_dir_all(path).unwrap();
        }
    }
}
