//! Observable launch stages. Progress never grants gameplay or save authority.
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) enum Stage {
    Reading,
    Propagating,
    Verifying,
    Publishing,
    Region,
    Ready,
    Failed,
}
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub(crate) struct Progress {
    pub(crate) stage: Stage,
    pub(crate) completed: Duration,
    pub(crate) total: Duration,
}
impl Progress {
    pub(crate) fn at(stage: Stage) -> Self {
        Self {
            stage,
            completed: Duration::ZERO,
            total: Duration::ZERO,
        }
    }
    pub(crate) fn propagating(completed: Duration, total: Duration) -> Self {
        Self {
            stage: Stage::Propagating,
            completed,
            total,
        }
    }
    pub(crate) fn percentage(self) -> Option<f64> {
        Some(match self.stage {
            Stage::Reading
            | Stage::Failed
            | Stage::Verifying
            | Stage::Publishing
            | Stage::Region => return None,
            Stage::Propagating => {
                100. * if self.total.is_zero() {
                    1.
                } else {
                    (self.completed.as_secs_f64() / self.total.as_secs_f64()).clamp(0., 1.)
                }
            }
            Stage::Ready => 100.,
        })
    }
    pub(crate) fn menu(self, error: Option<&str>) -> String {
        let stage = match self.stage {
            Stage::Reading => "Reading your journey",
            Stage::Propagating => "Catching up ship and systems",
            Stage::Verifying => "Checking the recovered journey",
            Stage::Publishing => "Saving the verified result",
            Stage::Region => "Preparing the current region",
            Stage::Ready => "Journey ready",
            Stage::Failed => "Journey could not be verified",
        };
        let mut text = format!("SALIMON\n\n{stage}\n");
        if let Some(p) = self.percentage() {
            let filled = (p / 5.).floor() as usize;
            text.push_str(&format!(
                "\nCatch-up {:5.1}%  [{}{}]\n",
                p,
                "#".repeat(filled),
                "-".repeat(20 - filled)
            ));
        }
        if self.stage == Stage::Propagating {
            text.push_str(&format!(
                "\nElapsed work: {:.1} / {:.1} seconds\n",
                self.completed.as_secs_f64(),
                self.total.as_secs_f64()
            ));
        }
        if self.stage == Stage::Ready {
            text.push_str("\nENTER   CONTINUE\nX       QUIT");
        } else if self.stage == Stage::Failed {
            text.push_str("\nResume unavailable\nR   RETRY     X   QUIT\n\n");
            if let Some(e) = error {
                for (i, c) in e.chars().take(320).enumerate() {
                    if i > 0 && i % 45 == 0 {
                        text.push('\n');
                    }
                    text.push(c);
                }
            }
        } else {
            text.push_str("\nResume unavailable until verification completes\nX   QUIT");
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_measured_elapsed_progress_has_a_percentage_and_no_early_resume_label() {
        let p = Progress::propagating(Duration::from_secs(25), Duration::from_secs(100));
        assert_eq!(p.percentage(), Some(25.));
        assert!(p.menu(None).contains("25.0%"));
        assert!(!p.menu(None).contains("ENTER"));
        for stage in [
            Stage::Reading,
            Stage::Verifying,
            Stage::Publishing,
            Stage::Region,
            Stage::Failed,
        ] {
            let p = Progress::at(stage);
            assert_eq!(p.percentage(), None);
            assert!(!p.menu(None).contains("ENTER"));
        }
        assert!(Progress::at(Stage::Ready).menu(None).contains("ENTER"));
    }
}
