//! Fixed-memory callback state. No locks, allocation, logging or platform calls.
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::SeqCst};
pub(super) const VOICES: usize = 13;
#[derive(Clone, Copy)]
pub(crate) struct Control {
    pub epoch: u64,
    pub enabled: u32,
    pub gains: [f32; VOICES],
    pub pans: [f32; VOICES],
    pub events: [bool; VOICES],
}
impl Default for Control {
    fn default() -> Self {
        Self {
            epoch: 0,
            enabled: 0,
            gains: [0.; VOICES],
            pans: [0.; VOICES],
            events: [false; VOICES],
        }
    }
}
pub(super) struct Mailbox {
    revision: AtomicU64,
    epoch: AtomicU64,
    enabled: AtomicU32,
    stamp_ms: AtomicU64,
    gains: [AtomicU32; VOICES],
    pans: [AtomicU32; VOICES],
    events: [AtomicU64; VOICES],
}
#[derive(Clone, Copy, Default)]
struct Snapshot {
    epoch: u64,
    enabled: u32,
    stamp_ms: u64,
    gains: [f32; VOICES],
    pans: [f32; VOICES],
    events: [u64; VOICES],
}
impl Default for Mailbox {
    fn default() -> Self {
        Self {
            revision: AtomicU64::new(0),
            epoch: AtomicU64::new(0),
            enabled: AtomicU32::new(0),
            stamp_ms: AtomicU64::new(0),
            gains: std::array::from_fn(|_| AtomicU32::new(0)),
            pans: std::array::from_fn(|_| AtomicU32::new(0)),
            events: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}
impl Mailbox {
    // One producer. SeqCst makes the bounded coherent-copy protocol explicit.
    pub(super) fn publish(&self, c: Control, stamp_ms: u64) {
        self.revision.fetch_add(1, SeqCst);
        self.epoch.store(c.epoch, SeqCst);
        self.enabled.store(c.enabled, SeqCst);
        self.stamp_ms.store(stamp_ms, SeqCst);
        for i in 0..VOICES {
            self.gains[i].store(c.gains[i].clamp(0., 1.).to_bits(), SeqCst);
            self.pans[i].store(c.pans[i].clamp(-1., 1.).to_bits(), SeqCst);
            if c.events[i] {
                self.events[i].fetch_add(1, SeqCst);
            }
        }
        self.revision.fetch_add(1, SeqCst);
    }
    fn read(&self) -> Option<Snapshot> {
        for _ in 0..3 {
            let rev = self.revision.load(SeqCst);
            if rev & 1 != 0 {
                continue;
            }
            let s = Snapshot {
                epoch: self.epoch.load(SeqCst),
                enabled: self.enabled.load(SeqCst),
                stamp_ms: self.stamp_ms.load(SeqCst),
                gains: std::array::from_fn(|i| f32::from_bits(self.gains[i].load(SeqCst))),
                pans: std::array::from_fn(|i| f32::from_bits(self.pans[i].load(SeqCst))),
                events: std::array::from_fn(|i| self.events[i].load(SeqCst)),
            };
            if rev == self.revision.load(SeqCst) {
                return Some(s);
            }
        }
        None
    }
}
#[derive(Clone, Copy)]
struct Voice {
    gain: f64,
    pan: f64,
    target_pan: f64,
    level: f64,
    age: f64,
    phase: f64,
    seen: u64,
    noise: f64,
}
impl Default for Voice {
    fn default() -> Self {
        Self {
            gain: 0.,
            pan: 0.,
            target_pan: 0.,
            level: 0.,
            age: 2.,
            phase: 0.,
            seen: 0,
            noise: 0.,
        }
    }
}
pub(super) struct Mix {
    voices: [Voice; VOICES],
    snapshot: Snapshot,
    seed: u32,
    rate: f64,
}
impl Mix {
    pub(super) fn new(rate: u32) -> Self {
        Self {
            voices: [Voice::default(); VOICES],
            snapshot: Snapshot::default(),
            seed: 1,
            rate: f64::from(rate),
        }
    }
    pub(super) fn begin(&mut self, mailbox: &Mailbox, now_ms: u64) {
        if let Some(s) = mailbox.read() {
            if s.epoch != self.snapshot.epoch {
                for (i, v) in self.voices.iter_mut().enumerate() {
                    v.level = 0.;
                    v.seen = s.events[i];
                }
            }
            self.snapshot = s;
        }
        let stale = now_ms.saturating_sub(self.snapshot.stamp_ms) > 250;
        for (i, v) in self.voices.iter_mut().enumerate() {
            if stale || self.snapshot.enabled & (1 << i) == 0 {
                v.seen = self.snapshot.events[i];
                v.level = 0.;
            } else if i < 6 {
                v.level = f64::from(self.snapshot.gains[i]);
                v.target_pan = f64::from(self.snapshot.pans[i]);
            } else if v.seen != self.snapshot.events[i] {
                v.seen = self.snapshot.events[i];
                v.age = 0.;
                v.level = f64::from(self.snapshot.gains[i]);
                v.target_pan = f64::from(self.snapshot.pans[i]);
            }
        }
    }
    pub(super) fn sample(&mut self) -> [f32; 2] {
        let dt = 1. / self.rate;
        let smooth = 1. - (-dt / 0.01).exp();
        let mut out = [0f64; 2];
        for (i, v) in self.voices.iter_mut().enumerate() {
            self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let noise = f64::from(self.seed >> 8) / 8388608. - 1.;
            v.noise += 0.035 * (noise - v.noise);
            let hz = [
                67., 47., 180., 220., 43., 120., 90., 180., 140., 440., 65., 360., 110.,
            ][i];
            v.phase = (v.phase + dt).rem_euclid(64.);
            let t = v.phase;
            let a = v.age;
            let tone = |f: f64| (std::f64::consts::TAU * f * t).sin();
            let signal = match i {
                0 => tone(hz) * 0.6 + tone(134.) * 0.2,
                1 => v.noise * 0.5,
                2 => tone(hz) * (0.6 + 0.2 * tone(7.)),
                3 => tone(hz) * 0.5 + v.noise * 0.12,
                4 => tone(hz) * 0.5 + v.noise * 0.3,
                5 => tone(hz) * 0.4 + v.noise * 0.2,
                6 => (tone(hz) * 0.4 + v.noise * 0.4) * (-a * 30.).exp(),
                7 => (std::f64::consts::TAU * (180. * a - 100. * a * a)).sin() * (-a * 8.).exp(),
                8 => (std::f64::consts::TAU * (140. * a + 160. * a * a)).sin() * (-a * 8.).exp(),
                9 => tone(hz) * (-a * 12.).exp(),
                10 => {
                    tone(hz)
                        * if a < 0.12 {
                            (1. - a / 0.12).powi(2)
                        } else if (0.23..0.35).contains(&a) {
                            (1. - (a - 0.23) / 0.12).powi(2)
                        } else {
                            0.
                        }
                }
                11 => {
                    tone(hz)
                        * if a < 0.3 {
                            (std::f64::consts::PI * a / 0.3).sin().powi(2)
                        } else {
                            0.
                        }
                }
                _ => tone(hz) * (std::f64::consts::PI * a.min(1.)).sin().powi(2) * 0.35,
            };
            let level = if i >= 6 && a >= 1. { 0. } else { v.level };
            v.gain += (level - v.gain) * smooth;
            v.pan += (v.target_pan - v.pan) * smooth;
            let sample = signal * v.gain * 0.36;
            out[0] += sample * ((1. - v.pan) * 0.5).sqrt();
            out[1] += sample * ((1. + v.pan) * 0.5).sqrt();
            v.age = (a + dt).min(2.);
        }
        // Smooth saturation preserves headroom without hard sample clipping.
        out.map(|v| v.tanh() as f32)
    }
}
#[cfg(test)]
mod tests;
