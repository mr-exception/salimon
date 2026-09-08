//! Original stereo PCM, no sample downloads, music or media licensing.
use super::{Cue, MAX_VOICES, Voice};
pub(super) const RATE: u32 = 22050;
pub(super) const FRAMES: usize = 11025; // half-second bounded playback chunk
pub(super) const WAV_BYTES: usize = 44 + FRAMES * 4;

pub(super) fn wav(voices: &[Voice], sequence: u64) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(WAV_BYTES);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(WAV_BYTES as u32 - 8).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&RATE.to_le_bytes());
    bytes.extend_from_slice(&(RATE * 4).to_le_bytes());
    bytes.extend_from_slice(&4u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(FRAMES as u32 * 4).to_le_bytes());
    for i in 0..FRAMES {
        let t = i as f64 / f64::from(RATE);
        let global = t + (sequence % 4096) as f64 * 0.5;
        let mut stereo = [0.0; 2];
        for voice in voices.iter().take(MAX_VOICES) {
            if !voice.gain.is_finite() || !voice.pan.is_finite() {
                continue;
            }
            let tone = |hz: f64| (std::f64::consts::TAU * hz * global).sin();
            let noise = ((i as u32).wrapping_mul(1664525).wrapping_add(1013904223) >> 8) as f64
                / 8388608.0
                - 1.0;
            let sample = match voice.cue {
                Cue::Ventilation => noise * 0.3,
                Cue::Core => tone(67.0) * 0.6 + tone(134.0) * 0.2,
                Cue::Conversion => tone(180.0) * (0.6 + 0.2 * tone(7.0)),
                Cue::Fabricator => tone(220.0) * 0.5 + noise * 0.12,
                Cue::Thrust => tone(43.0) * 0.5 + noise * 0.3,
                Cue::Ramp => tone(120.0) * 0.4 + noise * 0.2,
                Cue::Footstep => (tone(90.0) * 0.4 + noise * 0.4) * (-t * 30.0).exp(),
                Cue::Blackout => {
                    (std::f64::consts::TAU * (180.0 * t - 100.0 * t * t)).sin() * (-t * 8.0).exp()
                }
                Cue::Restored => {
                    (std::f64::consts::TAU * (140.0 * t + 160.0 * t * t)).sin() * (-t * 8.0).exp()
                }
                Cue::LowPower => tone(440.0) * (-t * 12.0).exp(),
                _ => noise * 0.2 * (-t * 25.).exp(),
            };
            let pan = voice.pan.clamp(-1.0, 1.0);
            let gain = voice.gain.clamp(0.0, 1.0);
            stereo[0] += sample * gain * ((1.0 - pan) * 0.5).sqrt();
            stereo[1] += sample * gain * ((1.0 + pan) * 0.5).sqrt();
        }
        // Short edge fades prevent clicks when OS chunks start/stop. Gaps remain
        // an explicit limitation of the minimal subprocess playback boundary.
        let fade = (t / 0.015).min(1.0) * ((0.5 - t) / 0.015).min(1.0);
        for channel in stereo {
            let value = (channel * fade * 12000.0).clamp(-30000.0, 30000.0) as i16;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    bytes
}
