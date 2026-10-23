//! Observational ship soundscape. No simulation, save or power authority.
#[cfg(test)]
mod synth;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod verification;
use crate::{
    condition_feedback::Settings,
    session::{GameplaySession, ViewMode},
};
use salimon_character::CharacterLocation;
use std::{collections::VecDeque, path::Path, time::Duration};
#[cfg(test)]
use std::{path::PathBuf, time::Instant};

const MAX_EVENTS: usize = 4;
const MAX_VOICES: usize = 8;
const CORE: [f64; 3] = [-1.0, 1.2, 0.0];
const FABRICATOR: [f64; 3] = [-3.0, 1.65, 3.55];
const RAMP: [f64; 3] = [-7.1, 1.34, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cue {
    Core,
    Ventilation,
    Conversion,
    Fabricator,
    Thrust,
    Ramp,
    Footstep,
    Blackout,
    Restored,
    LowPower,
}
#[derive(Clone, Copy, Debug)]
struct Voice {
    cue: Cue,
    gain: f64,
    pan: f64,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct Observation {
    active: bool,
    interior: bool,
    walking: bool,
    eye: [f64; 3],
    right: [f64; 3],
    powered: bool,
    conversion: bool,
    fabrication: bool,
    thrust: f64,
    door: f64,
    low_power: bool,
    serial: u64,
    now: Duration,
}
impl Observation {
    pub(crate) fn capture(s: &GameplaySession, active: bool) -> Self {
        let state = s.snapshot();
        let frame = crate::session::frames::character_ship_frame(state.ship.pose);
        let eye = frame.world_to_local(state.character.eye_position_meters);
        let look = frame.world_to_local(state.character.look_target_meters);
        let direction = salimon_math::sub(look, eye);
        // Ship +Y up, +Z port: forward +X yields right -Z.
        let right = salimon_math::cross(direction, [0.0, 1.0, 0.0]);
        let length = salimon_math::length(right);
        let right = if length > 1e-6 {
            right.map(|v| v / length)
        } else {
            [0., 0., -1.]
        };
        Self {
            active: active
                && !s.is_paused()
                && !s.survival.snapshot().terminal
                && s.view_mode == ViewMode::Gameplay,
            interior: matches!(
                state.character.location,
                CharacterLocation::InsideShip
                    | CharacterLocation::Cockpit
                    | CharacterLocation::DoorwayBlend
            ),
            walking: state.character.location == CharacterLocation::InsideShip
                && state.ship.power_available
                && (s.movement_input.forward
                    || s.movement_input.backward
                    || s.movement_input.left
                    || s.movement_input.right)
                && (eye[1]
                    - salimon_character::SHIP_FLOOR_HEIGHT_METERS
                    - salimon_character::PLAYER_EYE_HEIGHT_METERS)
                    .abs()
                    < 0.15,
            eye,
            right,
            powered: state.ship.power_available,
            conversion: state.ship.core_status.conversion_watts > 0.0,
            fabrication: state.ship.core_status.fabricator_watts > 0.0,
            thrust: if state.ship.power_available
                && state.ship.engines_enabled
                && state.ship.flight_state == salimon_ship::FlightState::Flying
            {
                f64::from(state.ship.thruster_percentage).abs() / 100.0
            } else {
                0.0
            },
            door: state.ship.door_open_fraction,
            low_power: state.ship.core_status.low_energy,
            serial: s.run.serial,
            now: s.elapsed,
        }
    }
}

#[derive(Default)]
struct Mixer {
    last: Option<Observation>,
    pending: VecDeque<Cue>,
    walking_meters: f64,
    ramp_until: Duration,
}
impl Mixer {
    fn queue(&mut self, cue: Cue) {
        if matches!(cue, Cue::Blackout | Cue::Restored | Cue::LowPower) {
            if self.pending.len() == MAX_EVENTS {
                self.pending.pop_back();
            }
            self.pending.push_front(cue);
        } else if self.pending.len() < MAX_EVENTS {
            self.pending.push_back(cue);
        }
    }
    fn observe(&mut self, o: Observation, settings: Settings) -> bool {
        let enabled = o.active
            && o.interior
            && settings.scene_audio
            && !settings.master_mute
            && settings.scene_volume_percent > 0;
        let old = self.last.replace(o);
        if !enabled {
            self.pending.clear();
            self.walking_meters = 0.0;
            self.ramp_until = Duration::ZERO;
            self.last = None;
            return false;
        }
        if let Some(old) = old.filter(|old| old.serial == o.serial && o.now >= old.now) {
            if old.powered && !o.powered {
                self.queue(Cue::Blackout);
            }
            if !old.powered && o.powered {
                self.queue(Cue::Restored);
            }
            if o.powered && o.low_power && !old.low_power {
                self.queue(Cue::LowPower);
            }
            if (old.door - o.door).abs() > 1e-5 {
                self.ramp_until = o.now + Duration::from_millis(500);
            }
            let delta = salimon_math::sub(o.eye, old.eye);
            let distance = delta[0].hypot(delta[2]);
            // Ignore resets/teleports and airborne drift. Ship translation does
            // not produce footsteps because these are local coordinates.
            if old.walking && o.walking && distance < 2.0 {
                self.walking_meters += distance;
                if self.walking_meters >= 0.8 {
                    self.walking_meters %= 0.8;
                    self.queue(Cue::Footstep);
                }
            } else {
                self.walking_meters = 0.0;
            }
        } else {
            self.pending.clear();
            self.walking_meters = 0.0;
            self.ramp_until = Duration::ZERO;
        }
        true
    }
    fn voices(&mut self, settings: Settings) -> Vec<Voice> {
        let Some(o) = self.last else {
            return Vec::new();
        };
        let volume = f64::from(settings.scene_volume_percent.min(100)) / 100.0;
        let mut voices = Vec::with_capacity(MAX_VOICES);
        let mut add = |cue, source, gain| {
            if voices.len() < MAX_VOICES {
                let (attenuation, pan) = spatial(o.eye, o.right, source);
                let gain = gain * volume * attenuation;
                if gain > 0.001 {
                    voices.push(Voice { cue, gain, pan });
                }
            }
        };
        for cue in self.pending.drain(..) {
            let source = if cue == Cue::Footstep { o.eye } else { CORE };
            add(cue, source, 0.35);
        }
        if o.powered {
            add(Cue::Core, CORE, 0.065);
            add(Cue::Ventilation, [1., 2.3, 0.], 0.035);
            if o.conversion {
                add(Cue::Conversion, CORE, 0.18);
            }
            if o.fabrication {
                add(Cue::Fabricator, FABRICATOR, 0.25);
            }
            if o.thrust > 0.0 {
                add(Cue::Thrust, [-6.4, 1.1, 0.0], 0.3 * o.thrust);
            }
        }
        if o.now < self.ramp_until {
            add(Cue::Ramp, RAMP, 0.24);
        }
        voices
    }
}

fn spatial(eye: [f64; 3], right: [f64; 3], source: [f64; 3]) -> (f64, f64) {
    let delta = salimon_math::sub(source, eye);
    let distance = salimon_math::length(delta);
    if !distance.is_finite() || !right.iter().all(|v| v.is_finite()) || distance > 20.0 {
        return (0.0, 0.0);
    }
    let pan = if distance > 1e-6 {
        salimon_math::dot(delta, right) / distance
    } else {
        0.0
    };
    (
        1.0 / (1.0 + distance * distance / 9.0),
        pan.clamp(-1.0, 1.0),
    )
}

#[derive(Default)]
pub(crate) struct Presenter {
    mixer: Mixer,
    output: crate::native_audio::Output,
    epoch: u64,
    last_run: Option<(u64, Duration)>,
}
impl Presenter {
    pub(crate) fn telemetry(&self) -> serde_json::Value {
        self.output.telemetry()
    }
    pub(crate) fn unavailable(&self) -> bool {
        self.output.unavailable()
    }
    pub(crate) fn stop(&mut self) {
        self.mixer = Mixer::default();
        self.last_run = None;
        self.epoch = self.epoch.wrapping_add(1);
        self.output.silence(self.epoch);
    }
    pub(crate) fn update(
        &mut self,
        o: Observation,
        settings: Settings,
        condition: Option<crate::condition_feedback::Cue>,
    ) {
        let reset = self
            .last_run
            .is_some_and(|(serial, now)| serial != o.serial || o.now < now);
        if reset {
            self.stop();
        }
        self.last_run = Some((o.serial, o.now));
        let active = o.active && !settings.master_mute;
        let scene = self.mixer.observe(o, settings);
        if !active {
            self.stop();
            return;
        }
        let voices = if scene {
            self.mixer.voices(settings)
        } else {
            Vec::new()
        };
        self.output
            .update(controls(voices, self.epoch, scene, condition, settings));
    }
}

fn controls(
    voices: Vec<Voice>,
    epoch: u64,
    scene: bool,
    condition: Option<crate::condition_feedback::Cue>,
    settings: Settings,
) -> crate::native_audio::Control {
    let mut control = crate::native_audio::Control {
        epoch,
        enabled: if scene { (1 << 10) - 1 } else { 0 },
        ..Default::default()
    };
    if settings.condition_audio {
        control.enabled |= 7 << 10;
    }
    for v in voices {
        let index = match v.cue {
            Cue::Core => 0,
            Cue::Ventilation => 1,
            Cue::Conversion => 2,
            Cue::Fabricator => 3,
            Cue::Thrust => 4,
            Cue::Ramp => 5,
            Cue::Footstep => 6,
            Cue::Blackout => 7,
            Cue::Restored => 8,
            Cue::LowPower => 9,
        };
        control.gains[index] = v.gain as f32;
        control.pans[index] = v.pan as f32;
        control.events[index] = index >= 6;
    }
    if settings.condition_audio
        && let Some(cue) = condition
    {
        let index = match cue {
            crate::condition_feedback::Cue::Heart => 10,
            crate::condition_feedback::Cue::Hunger => 11,
            crate::condition_feedback::Cue::Breath => 12,
        };
        control.gains[index] = 0.125;
        control.events[index] = true;
    }
    control
}
