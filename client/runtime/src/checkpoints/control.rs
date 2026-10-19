//! Portable request/status policy for the #382 pause-menu consumer.
//! UI wiring is blocked by #381; IO completion is the only success authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Status {
    #[default]
    Idle,
    Saving,
    Saved {
        checkpoint_utc: u64,
    },
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)] // Pause-menu admission follows #381; tests exercise the policy now.
pub(crate) enum Request {
    SaveNow,
    SaveAndExit,
    Retry,
    CancelExit,
    ExitWithRisk,
}
#[derive(Debug, Default)]
pub(crate) struct Control {
    pub(crate) status: Status,
    pub(crate) last_verified_utc: Option<u64>,
    pub(crate) pending: bool,
    pub(crate) exit_requested: bool,
    pub(crate) exit_ready: bool,
    captured_utc: Option<u64>,
}
impl Control {
    #[allow(dead_code)] // Called by the future #381/#382 native menu adapter.
    pub(crate) fn request(&mut self, request: Request) {
        match request {
            Request::SaveNow | Request::SaveAndExit | Request::Retry => {
                self.pending = true;
                self.exit_ready = false;
                self.exit_requested |= request == Request::SaveAndExit;
                self.status = Status::Saving;
            }
            Request::CancelExit => {
                self.exit_requested = false;
                self.exit_ready = false;
                // A pending SaveNow may still be useful; cancel only exit intent.
            }
            Request::ExitWithRisk => {
                if self.exit_requested && self.status == Status::Failed {
                    self.exit_ready = true;
                    self.pending = false;
                }
            }
        }
    }
    pub(crate) fn started(&mut self, utc: u64) {
        self.pending = false;
        self.captured_utc = Some(utc);
        self.status = Status::Saving;
    }
    pub(crate) fn completed(&mut self, success: bool) {
        if success {
            if let Some(utc) = self.captured_utc.take() {
                self.last_verified_utc = Some(utc);
                self.status = if self.pending {
                    Status::Saving
                } else {
                    Status::Saved {
                        checkpoint_utc: utc,
                    }
                };
                // A request made during an older write must capture once more.
                self.exit_ready = self.exit_requested && !self.pending;
            }
        } else {
            self.captured_utc = None;
            self.pending = false;
            self.exit_ready = false;
            self.status = Status::Failed;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn timestamp_requires_verification_and_failed_writes_never_report_success() {
        let mut c = Control::default();
        c.started(10);
        assert_eq!(c.last_verified_utc, None);
        c.completed(true);
        assert_eq!(c.status, Status::Saved { checkpoint_utc: 10 });
        c.started(20);
        c.completed(false);
        assert_eq!(c.status, Status::Failed);
        assert_eq!(c.last_verified_utc, Some(10));
        c.completed(true);
        assert_eq!(c.status, Status::Failed); // No outstanding receipt.
    }
    #[test]
    fn old_inflight_save_cannot_satisfy_a_new_exit_request() {
        let mut c = Control::default();
        c.started(10);
        for _ in 0..1000 {
            c.request(Request::SaveNow);
        }
        c.request(Request::SaveAndExit);
        c.completed(true);
        assert!(c.pending);
        assert!(!c.exit_ready);
        c.started(11);
        c.completed(true);
        assert!(c.exit_ready);
        assert_eq!(c.last_verified_utc, Some(11));
    }
    #[test]
    fn failure_waits_for_retry_cancel_or_explicit_risk_choice() {
        let mut c = Control::default();
        c.request(Request::SaveAndExit);
        c.started(1);
        c.completed(false);
        assert!(!c.exit_ready);
        assert!(!c.pending);
        c.request(Request::Retry);
        c.started(2);
        c.completed(true);
        assert!(c.exit_ready);
        let mut c = Control::default();
        c.request(Request::SaveAndExit);
        c.started(3);
        c.completed(false);
        c.request(Request::CancelExit);
        c.request(Request::ExitWithRisk);
        assert!(!c.exit_ready);
        c.request(Request::SaveAndExit);
        c.started(4);
        c.completed(false);
        c.request(Request::ExitWithRisk);
        assert!(c.exit_ready);
        assert_eq!(c.last_verified_utc, None);
        assert_eq!(c.status, Status::Failed);
    }
    #[test]
    fn cancel_while_busy_never_exits_and_rapid_pending_requests_stay_bounded() {
        let mut c = Control::default();
        c.request(Request::SaveAndExit);
        c.started(1);
        for _ in 0..10000 {
            c.request(Request::SaveNow);
        }
        c.request(Request::CancelExit);
        c.completed(true);
        assert!(!c.exit_ready);
        c.started(2);
        c.completed(true);
        assert!(!c.exit_ready);
        assert!(!c.pending);
    }
}
