//! Small optional OS playback adapter. No audio/build dependency; failure is explicit.
use super::Cue;
/// Original deterministic PCM synthesis, no third-party samples or media license.
fn wav(cue: Cue) -> Vec<u8> {
    let rate = 22050u32;
    let count = rate as usize;
    let mut bytes = Vec::with_capacity(44 + count * 2);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + count as u32 * 2).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&(count as u32 * 2).to_le_bytes());
    for i in 0..count {
        let t = i as f64 / f64::from(rate);
        let (freq, amp) = match cue {
            Cue::Heart => (
                65.,
                if t < 0.12 {
                    (1. - t / 0.12).powi(2)
                } else if (0.23..0.35).contains(&t) {
                    (1. - (t - 0.23) / 0.12).powi(2)
                } else {
                    0.
                },
            ),
            Cue::Hunger => (
                360.,
                if t < 0.3 {
                    (std::f64::consts::PI * t / 0.3).sin().powi(2)
                } else {
                    0.
                },
            ),
            Cue::Breath => (110., (std::f64::consts::PI * t).sin().powi(2) * 0.35),
        };
        let sample = (1500. * amp * (std::f64::consts::TAU * freq * t).sin()) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    bytes
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_wav_is_bounded_and_valid_pcm() {
        for c in [Cue::Heart, Cue::Hunger, Cue::Breath] {
            let w = wav(c);
            assert_eq!(&w[..4], b"RIFF");
            assert_eq!(w.len(), 44144);
            assert!(
                w[44..]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .all(|p| i16::from_le_bytes([p[0], p[1]]).abs() <= 1500)
            );
        }
    }
}
