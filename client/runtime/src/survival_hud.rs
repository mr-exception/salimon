//! Observational survival presentation. No commands, IO or saved state.
mod raster;
use crate::session::{GameplaySession, ViewMode};
use salimon_survival::effects::{Behavior, Lifetime, Removal, Source};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) enum Icon {
    Health,
    Hunger,
    Fatigue,
    Damage,
    Recovery,
    Amplify,
    Wound,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    Normal,
    Low,
    Critical,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct Status {
    pub(crate) icon: Icon,
    pub(crate) description: String,
    pub(crate) duration: String,
}
#[derive(Clone, Debug)]
pub(crate) struct Observation {
    serial: u64,
    now: Duration,
    values: [f64; 3], // health, hunger, fatigue
    pub(crate) statuses: Vec<Status>,
    pub(crate) penalties: [String; 4],
    pub(crate) visible: bool,
}
impl Observation {
    pub(crate) fn capture(session: &GameplaySession) -> Self {
        let v = session.survival.snapshot();
        let r = session.restrictions();
        let mut statuses = Vec::new();
        if v.hunger == salimon_survival::Points::ZERO {
            statuses.push(Status {
                icon: Icon::Hunger,
                description: "STARVATION: HEALTH DAMAGE WHILE EMPTY".into(),
                duration: "UNTIL FED".into(),
            });
        }
        if r.exhausted {
            statuses.push(Status {
                icon: Icon::Fatigue,
                description: "EXHAUSTED: PILOT/MINE/CRAFT/CARRY/CORE BLOCKED".into(),
                duration: "FOOD, DOORS, MENUS STILL AVAILABLE".into(),
            });
        }
        if v.wounds != salimon_survival::Points::ZERO {
            statuses.push(Status {
                icon: Icon::Wound,
                description: format!("WOUNDS {:.1}: REQUIRES TREATMENT", v.wounds.value()),
                duration: "NEEDS TREATMENT".into(),
            });
        }
        for effect in session.survival.effect_snapshots() {
            let period = effect
                .rule
                .interval
                .map_or(String::new(), |d| format!(" / {:.3}S", d.as_secs_f64()));
            let stacks = f64::from(effect.stacks);
            let (icon, description) = match effect.rule.behavior {
                Behavior::Feed { amount } => (
                    Icon::Hunger,
                    format!("HUNGER +{:.1}{period}", amount.value() * stacks),
                ),
                Behavior::Heal { amount, mode } => (
                    Icon::Recovery,
                    format!("{mode:?} HEALTH +{:.1}{period}", amount.value() * stacks),
                ),
                Behavior::Damage {
                    source,
                    amount,
                    wound,
                } => (
                    Icon::Damage,
                    format!(
                        "{} -{:.1} HEALTH{period}{}",
                        source_name(source),
                        amount.value() * stacks,
                        if wound { " + WOUND" } else { "" }
                    ),
                ),
                Behavior::Amplify {
                    source,
                    basis_points,
                } => (
                    Icon::Amplify,
                    format!(
                        "{} DAMAGE +{}%",
                        source_name(source),
                        percent(u32::from(basis_points) * u32::from(effect.stacks))
                    ),
                ),
            };
            let lifetime = effect.remaining.map_or_else(
                || match effect.rule.lifetime {
                    Lifetime::Exposure { .. } => "EXPOSED".into(),
                    _ => "PERSISTENT".into(),
                },
                |d| format!("{}S LEFT", d.as_secs() + u64::from(d.subsec_nanos() > 0)),
            );
            statuses.push(Status {
                icon,
                description: format!(
                    "#{}, X{}: {}",
                    effect.rule.id.0,
                    effect.stacks,
                    description.to_ascii_uppercase()
                ),
                duration: format!(
                    "{lifetime}{}",
                    if effect.rule.removal == Removal::TreatmentOnly {
                        " / TREATMENT ONLY"
                    } else {
                        ""
                    }
                ),
            });
        }
        Self {
            serial: session.run.serial,
            now: v.active_time,
            values: [v.health.value(), v.hunger.value(), v.fatigue.value()],
            statuses,
            penalties: [
                format!("HUNGER SPEED -{}%", percent(10000 - r.hunger_movement_bps)),
                format!(
                    "FATIGUE SPEED -{}%",
                    percent(10000 - r.fatigue_movement_bps)
                ),
                format!("COMBINED SPEED -{}%", percent(10000 - r.movement_bps)),
                format!("INTERACTION RATE -{}%", percent(10000 - r.interaction_bps)),
            ],
            visible: session.view_mode == ViewMode::Gameplay
                && !session.is_paused()
                && !session.crafting_ui.open
                && !v.terminal,
        }
    }
}
fn source_name(source: Source) -> String {
    match source {
        Source::Registered(id) => format!("SOURCE {id}"),
        _ => format!("{source:?}").to_ascii_uppercase(),
    }
}
fn percent(bps: u32) -> String {
    let value = format!("{:.2}", f64::from(bps) / 100.0);
    value.trim_end_matches('0').trim_end_matches('.').to_owned()
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Meter {
    pub(crate) icon: Icon,
    pub(crate) fill: u8,
    pub(crate) tone: Tone,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Frame {
    pub(crate) meters: [Meter; 3],
    pub(crate) statuses: Vec<Status>,
    pub(crate) penalties: [String; 4],
    pub(crate) pulse: bool,
    pub(crate) warning: Option<&'static str>,
    pub(crate) gains: Vec<String>,
    pub(crate) rise: u32,
}
#[derive(Clone, Copy, Debug)]
struct Tween {
    from: f64,
    target: f64,
    at: Duration,
}
impl Tween {
    fn value(self, now: Duration) -> f64 {
        let t = (now.saturating_sub(self.at).as_secs_f64() / 0.45).min(1.0);
        self.from + (self.target - self.from) * t
    }
}
#[derive(Default)]
pub(crate) struct Hud {
    previous: Option<Observation>,
    tweens: Option<[Tween; 3]>,
    gains: [Option<(f64, Duration)>; 2],
    warning: Option<(&'static str, Duration)>,
    armed: [bool; 3],
    pub(crate) details_open: bool,
    page: usize,
    cached: Option<(Frame, raster::Layout)>,
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    revision: u64,
}
impl Hud {
    pub(crate) fn close_details(&mut self) {
        self.details_open = false;
        self.page = 0;
    }
    pub(crate) fn toggle_details(&mut self) {
        self.details_open = !self.details_open;
        self.page = 0;
    }
    pub(crate) fn next_page(&mut self, forward: bool) {
        self.page = if forward {
            self.page.saturating_add(1)
        } else {
            self.page.saturating_sub(1)
        };
    }
    pub(crate) fn observe(&mut self, o: Observation) -> Frame {
        let reset = self
            .previous
            .as_ref()
            .is_none_or(|p| p.serial != o.serial || p.now > o.now);
        if reset {
            self.tweens = Some(o.values.map(|v| Tween {
                from: v,
                target: v,
                at: o.now,
            }));
            self.gains = [None; 2];
            self.warning = None;
            self.armed = [true; 3];
            self.close_details();
        }
        let tweens = self
            .tweens
            .as_mut()
            .expect("initialized above on first observation");
        if let Some(old) = &self.previous
            && !reset
        {
            for (i, tween) in tweens.iter_mut().enumerate() {
                let gain = o.values[i] - old.values[i];
                if gain > 0.000001 && i < 2 {
                    *tween = Tween {
                        from: tween.value(o.now),
                        target: o.values[i],
                        at: o.now,
                    };
                    self.gains[i] = Some((gain, o.now));
                } else if i == 1
                    && o.values[i] < old.values[i]
                    && o.now.saturating_sub(tween.at) < Duration::from_millis(450)
                    && tween.target > tween.from
                {
                    // Passive hunger drain must not cancel a just-consumed
                    // ration's visual recovery. It still updates the target.
                    tween.target = o.values[i];
                } else if o.values[i] < old.values[i] || i == 2 {
                    *tween = Tween {
                        from: o.values[i],
                        target: o.values[i],
                        at: o.now,
                    };
                }
            }
        }
        let thresholds = [25.0, 20.0, 20.0];
        for i in 0..3 {
            let critical = if i == 2 {
                o.values[i] <= thresholds[i]
            } else {
                o.values[i] < thresholds[i]
            };
            if o.values[i] >= thresholds[i] + 2.0 {
                self.armed[i] = true;
            }
            if critical && self.armed[i] {
                self.armed[i] = false;
                if self
                    .warning
                    .is_none_or(|(_, at)| o.now.saturating_sub(at) >= Duration::from_secs(3))
                {
                    self.warning = Some((["LOW HEALTH", "LOW HUNGER", "LOW FATIGUE"][i], o.now));
                }
            }
        }
        let mut gains = Vec::new();
        let mut rise = 0;
        for (i, gain) in self.gains.iter().enumerate() {
            if let Some((amount, at)) = gain
                && o.now.saturating_sub(*at) < Duration::from_millis(1200)
            {
                gains.push(format!(
                    "+{} {}",
                    format_amount(*amount),
                    ["HEALTH", "HUNGER"][i]
                ));
                rise = (o.now.saturating_sub(*at).as_millis() / 100) as u32;
            }
        }
        let frame = Frame {
            meters: std::array::from_fn(|i| Meter {
                icon: [Icon::Health, Icon::Hunger, Icon::Fatigue][i],
                fill: tweens[i].value(o.now).round().clamp(0., 100.) as u8,
                tone: if (i == 2 && o.values[i] <= 20.) || o.values[i] < thresholds[i] {
                    Tone::Critical
                } else if o.values[i] < 50. {
                    Tone::Low
                } else {
                    Tone::Normal
                },
            }),
            statuses: o.statuses.clone(),
            penalties: o.penalties.clone(),
            pulse: o.values.iter().enumerate().any(|(i, v)| {
                if i == 2 {
                    *v <= 20.
                } else {
                    *v < thresholds[i]
                }
            }) && (o.now.as_millis() / 400).is_multiple_of(2),
            warning: self
                .warning
                .filter(|(_, at)| o.now.saturating_sub(*at) < Duration::from_secs(3))
                .map(|(s, _)| s),
            gains,
            rise,
        };
        if !o.visible {
            self.close_details();
        }
        self.previous = Some(o);
        frame
    }
    pub(crate) fn image(
        &mut self,
        frame: Frame,
        viewport: [u32; 2],
        dpi: f64,
        visible: bool,
    ) -> Option<salimon_renderer::OverlayImage<'_>> {
        if !visible {
            self.close_details();
            return None;
        }
        let base_height = 58
            + if frame.statuses.is_empty() { 0 } else { 14 }
            + frame
                .penalties
                .iter()
                .take(2)
                .filter(|s| !s.ends_with(" -0%"))
                .count() as u32
                * 10
            + if frame.warning.is_some() { 12 } else { 0 };
        let layout = raster::Layout::new(
            viewport,
            dpi,
            self.details_open,
            self.page,
            frame.statuses.len(),
            base_height,
        )?;
        self.page = layout.page;
        if self.cached.as_ref() != Some(&(frame.clone(), layout)) {
            (self.width, self.height, self.pixels) = raster::draw(&frame, layout);
            self.cached = Some((frame, layout));
            self.revision = self.revision.wrapping_add(1);
        }
        Some(salimon_renderer::OverlayImage {
            width: self.width,
            height: self.height,
            rgba8: &self.pixels,
            revision: self.revision,
            placement: salimon_renderer::OverlayPlacement::TopLeft,
        })
    }
}
fn format_amount(amount: f64) -> String {
    if amount < 0.05 {
        return "<0.1".into();
    }
    let value = format!("{amount:.1}");
    value.trim_end_matches('0').trim_end_matches('.').to_owned()
}
#[cfg(test)]
mod tests;
