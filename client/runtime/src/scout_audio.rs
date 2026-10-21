//! Observational ship soundscape. No simulation, save or power authority.
mod synth;
#[cfg(test)]
mod tests;
use crate::{
    condition_feedback::Settings,
    session::{GameplaySession, ViewMode},
};
use salimon_character::CharacterLocation;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    process::Child,
    time::{Duration, Instant},
};

const MAX_EVENTS: usize = 4;
const MAX_VOICES: usize = 8;
const CORE: [f64; 3] = [-1.0, 1.2, 0.0];
const FABRICATOR: [f64; 3] = [-3.0, 1.65, 3.55];
const RAMP: [f64; 3] = [-7.1, 1.34, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cue {
    Core,
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
            thrust: if state.ship.power_available {
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
            add(Cue::Core, CORE, 0.12);
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
    child: Option<(Child, Instant)>,
    file: Option<PathBuf>,
    failed: bool,
    sequence: u64,
}
impl Presenter {
    pub(crate) fn unavailable(&self) -> bool {
        self.failed
    }
    pub(crate) fn stop(&mut self) {
        if let Some((mut child, _)) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.mixer = Mixer::default();
        self.failed = false;
    }
    pub(crate) fn update(&mut self, o: Observation, settings: Settings, path: Option<&Path>) {
        if !self.mixer.observe(o, settings) || path.is_none() {
            self.stop();
            return;
        }
        if self.failed {
            return;
        }
        if let Some((child, started)) = &mut self.child {
            match child.try_wait() {
                Ok(Some(status)) if status.success() => self.child = None,
                Ok(None) if started.elapsed() < Duration::from_secs(3) => return,
                result => {
                    log::warn!("Scout audio stopped: {result:?}");
                    self.stop();
                    self.failed = true;
                    return;
                }
            }
        }
        let voices = self.mixer.voices(settings);
        if voices.is_empty() {
            return;
        }
        let file = path
            .expect("checked settings path")
            .with_file_name(format!("scout-audio-{}.wav", std::process::id()));
        self.file = Some(file.clone());
        let result = (|| {
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file, synth::wav(&voices, self.sequence))?;
            crate::native_audio::spawn(&file)
        })();
        self.sequence = self.sequence.wrapping_add(1);
        match result {
            Ok(child) => self.child = Some((child, Instant::now())),
            Err(e) => {
                log::warn!("Scout audio unavailable: {e}");
                self.failed = true;
            }
        }
    }
}
impl Drop for Presenter {
    fn drop(&mut self) {
        self.stop();
        if let Some(file) = &self.file {
            let _ = std::fs::remove_file(file);
        }
    }
}
