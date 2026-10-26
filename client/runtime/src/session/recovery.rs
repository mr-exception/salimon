//! Portable Archive machine source. Core and Fabricator retain energy/item authority.
use super::GameplaySession;
use salimon_fabrication::{OutputCategory, OutputItem};
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryConfig {
    watts: f64,
    record_cost_joules: [f64; 2],
    mobility_reserve_joules: f64,
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Recovery {
    version: u32,
    config: RecoveryConfig,
    pub(super) module: Option<OutputItem>,
    spent_joules: [f64; 2],
    active: Option<usize>,
}
impl Default for Recovery {
    fn default() -> Self {
        let config = serde_json::from_str(include_str!("../../config/data-recovery.json"))
            .expect("checked-in recovery config JSON");
        Self {
            version: 1,
            config,
            module: None,
            spent_joules: [0.0; 2],
            active: None,
        }
    }
}
impl Validate for Recovery {
    fn validate(&self) -> Result<(), SaveError> {
        if self.version != 1
            || !self.config.watts.is_finite()
            || self.config.watts <= 0.0
            || self.config.watts > 1e12
            || !self.config.mobility_reserve_joules.is_finite()
            || self.config.mobility_reserve_joules < 0.0
            || self
                .config
                .record_cost_joules
                .iter()
                .any(|v| !v.is_finite() || *v <= 0.0)
            || self
                .spent_joules
                .iter()
                .zip(self.config.record_cost_joules)
                .any(|(s, c)| !s.is_finite() || *s < 0.0 || *s > c)
            || self.active.is_some_and(|i| {
                i >= 2 || self.spent_joules[i] >= self.config.record_cost_joules[i]
            })
            || (self.module.is_none() && (self.spent_joules != [0.0; 2] || self.active.is_some()))
        {
            return Err(invalid("recovery configuration/progress"));
        }
        if self.module.as_ref().is_some_and(|m| {
            m.recipe.key() != "data-recovery-module"
                || m.category != OutputCategory::Component
                || m.mass.milligrams() == 0
                || m.job.0 == 0
                || m.unit == 0
        }) {
            return Err(invalid("installed recovery identity"));
        }
        Ok(())
    }
}
impl Recovery {
    pub(super) fn checkpoint_events(&self) -> (Option<(u64, u32)>, Option<usize>, [bool; 2]) {
        (
            self.module.as_ref().map(|m| (m.job.0, m.unit)),
            self.active,
            std::array::from_fn(|i| self.complete(i)),
        )
    }
    fn complete(&self, i: usize) -> bool {
        self.spent_joules[i] >= self.config.record_cost_joules[i]
    }
    pub(super) fn record_text(&self, id: &str) -> Option<String> {
        let i = match id {
            "received.damaged-01" => 0,
            "received.damaged-02" => 1,
            _ => return None,
        };
        let status = if self.complete(i) {
            if i == 0 {
                "RECOVERED FRAGMENT / UNVERIFIED"
            } else {
                "IRRECOVERABLE SEGMENTS / PARTIAL"
            }
        } else if self.spent_joules[i] > 0.0 {
            "PARTIALLY RECONSTRUCTED / UNVERIFIED"
        } else {
            "DAMAGED / CONTENT UNKNOWN"
        };
        let content = if self.spent_joules[i] == 0.0 {
            "No message claim has been recovered."
        } else if i == 0 {
            "Received distress fragment: [signal incomplete].\nSender/time/location remain unverified.\nMissing data cannot be invented."
        } else {
            "Warning packet fragment: [content incomplete].\nMissing segments are permanently unrecoverable.\nNo origin or historical event is confirmed."
        };
        Some(format!("{status}\n{content}"))
    }
    pub(super) fn panel(&self, reserve: f64, stored: u64, multiplier: f64) -> String {
        let hardware = if self.module.is_some() {
            "INSTALLED"
        } else {
            "CRAFT data-recovery-module"
        };
        let active = if self.active.is_some() && stored as f64 <= reserve {
            "WAITING / RESERVE PROTECTED"
        } else if self.active.is_some() {
            "WORKING"
        } else {
            "STOPPED"
        };
        format!(
            "RECOVERY {hardware}\n{active}  {:.1} MW / reserve {:.2} GJ\nRecord progress: {:.1}% / {:.1}%\nWork left at requested power: {:.0}s / {:.0}s\nR: start/stop selected damaged record (no refund)",
            self.config.watts * multiplier / 1e6,
            reserve / 1e9,
            self.spent_joules[0] / self.config.record_cost_joules[0] * 100.0,
            self.spent_joules[1] / self.config.record_cost_joules[1] * 100.0,
            (self.config.record_cost_joules[0] - self.spent_joules[0])
                / (self.config.watts * multiplier),
            (self.config.record_cost_joules[1] - self.spent_joules[1])
                / (self.config.watts * multiplier)
        )
    }
}
impl GameplaySession {
    /// Immutable source evidence; never advances work or transfers identity.
    pub(crate) fn recovery_observation(&self) -> serde_json::Value {
        serde_json::json!(self.recovery)
    }

    pub(super) fn recovery_reserve(&self) -> f64 {
        let c = self.ship.energy_config();
        self.recovery.config.mobility_reserve_joules.max(
            (12.0 * (c.forward_watts + c.gravity_idle_watts + c.cockpit_idle_watts)).min(f64::MAX),
        )
    }
    pub(super) fn toggle_recovery(&mut self, record: &str) {
        let i = match record {
            "received.damaged-01" => 0,
            "received.damaged-02" => 1,
            _ => return,
        };
        if self.recovery.active == Some(i) {
            self.recovery.active = None;
            return;
        }
        if self.recovery.module.is_some() && !self.recovery.complete(i) {
            self.recovery.active = Some(i);
        }
    }
    pub(super) fn advance_recovery(&mut self, delta: Duration) {
        let Some(i) = self.recovery.active else {
            return;
        };
        let remaining = self.recovery.config.record_cost_joules[i] - self.recovery.spent_joules[i];
        let request =
            (self.recovery.config.watts * self.utility_recovery_multiplier() * delta.as_secs_f64())
                .min(remaining);
        let funded = self
            .ship
            .fund_auxiliary_work(request, self.recovery_reserve(), delta)
            .expect("validated finite recovery draw and reserve");
        self.recovery.spent_joules[i] += funded;
        if self.recovery.complete(i) {
            self.recovery.active = None;
        }
    }
    pub(super) fn install_recovery_module(&mut self, item: OutputItem) {
        if self.recovery.module.is_some() {
            self.feedback = Some("Recovery Module is already installed; item stays on tray");
            return;
        }
        // Preparation has no mutation: collection/installation commit under the
        // exclusive session borrow; the same identity moves tray -> installed.
        self.execute_fabricator(salimon_fabrication::FabricationCommand::Collect)
            .expect("validated occupied component tray");
        self.recovery.module = Some(item);
        self.sync_fabricator_mass();
        self.feedback =
            Some("Recovery Module installed at Archive; select damaged record and press R");
    }
}
#[cfg(test)]
mod tests;
