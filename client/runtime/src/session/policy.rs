//! Host execution policy is copied across restore/restart, never serialized.
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Policy {
    automation_step: Option<Duration>,
    interpolate: bool,
    measure_physics: bool,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            automation_step: None,
            interpolate: true,
            measure_physics: false,
        }
    }
}

impl Policy {
    #[cfg(test)]
    pub(crate) fn with_automation_step(step: Option<Duration>) -> Self {
        Self {
            automation_step: step,
            ..Self::default()
        }
    }
    pub(crate) fn automation_step(self) -> Option<Duration> {
        self.automation_step
    }
    pub(crate) fn interpolates(self) -> bool {
        self.interpolate
    }
    pub(crate) fn measures_physics(self) -> bool {
        self.measure_physics
    }
    pub(crate) fn set_automation_step(&mut self, step: Option<Duration>) {
        self.automation_step = step;
    }
    pub(crate) fn set_interpolation(&mut self, enabled: bool) {
        self.interpolate = enabled;
    }
    pub(crate) fn set_physics_measurement(&mut self, enabled: bool) {
        self.measure_physics = enabled;
    }
}
