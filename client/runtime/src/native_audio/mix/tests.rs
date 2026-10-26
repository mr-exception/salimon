use super::*;
fn control() -> Control {
    Control {
        enabled: (1 << VOICES) - 1,
        gains: [0.15; VOICES],
        ..Default::default()
    }
}
#[test]
fn samples_are_continuous_across_arbitrary_callback_boundaries_and_no_chunk_gaps() {
    let box_ = Mailbox::default();
    box_.publish(control(), 0);
    let mut a = Mix::new(48000);
    let mut b = Mix::new(48000);
    a.begin(&box_, 0);
    b.begin(&box_, 0);
    let expected: Vec<_> = (0..48000).map(|_| a.sample()).collect();
    let mut actual = Vec::new();
    for n in [127, 512, 143, 11025, 36193] {
        b.begin(&box_, 10);
        actual.extend((0..n).map(|_| b.sample()));
    }
    assert_eq!(actual, expected);
    assert!(
        actual
            .chunks(480)
            .all(|c| c.iter().any(|p| p[0].abs() > 1e-5))
    );
    assert!(actual.windows(2).all(|p| (p[1][0] - p[0][0]).abs() < 0.08));
}
#[test]
fn reset_mute_exterior_and_stale_control_retire_events_without_replay() {
    let mb = Mailbox::default();
    let mut c = control();
    c.events[6] = true;
    mb.publish(c, 0);
    let mut mix = Mix::new(48000);
    mix.begin(&mb, 0);
    assert!((0..480).any(|_| mix.sample()[0] != 0.));
    c.epoch = 1;
    c.enabled = 0;
    c.events = [false; VOICES];
    mb.publish(c, 1);
    mix.begin(&mb, 1);
    for _ in 0..48000 {
        mix.sample();
    }
    assert!(mix.sample().iter().all(|v| v.abs() < 1e-8));
    c.enabled = 1 << 6;
    mb.publish(c, 2);
    mix.begin(&mb, 2);
    assert!(mix.sample().iter().all(|v| v.abs() < 1e-8));
    c.events[6] = true;
    mb.publish(c, 3);
    mix.begin(&mb, 1000);
    for _ in 0..48000 {
        mix.sample();
    }
    assert!(mix.sample().iter().all(|v| v.abs() < 1e-8));
    c.events[6] = false;
    mb.publish(c, 1001);
    mix.begin(&mb, 1001);
    assert!(mix.sample().iter().all(|v| v.abs() < 1e-8));
}
#[test]
fn worst_mix_has_finite_headroom_bounded_state_and_immediate_transient_onset() {
    let mb = Mailbox::default();
    let mut c = control();
    c.gains = [1.; VOICES];
    c.events = [true; VOICES];
    mb.publish(c, 0);
    let mut mix = Mix::new(48000);
    mix.begin(&mb, 0);
    let pcm: Vec<_> = (0..48000).map(|_| mix.sample()).collect();
    assert!(pcm.iter().flatten().all(|v| v.is_finite() && v.abs() < 1.));
    assert!(pcm[..48].iter().flatten().any(|v| v.abs() > 1e-5));
    assert!(std::mem::size_of::<Mix>() < 4096);
    assert!(std::mem::size_of::<Mailbox>() < 1024);
}
#[test]
#[ignore = "writes original continuous listening fixtures and real-rate CPU measurements"]
fn continuous_audio_evidence_582() {
    use std::{path::PathBuf, time::Instant};
    let root = PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    let mut timing = String::from(
        "case,frames,sample_rate,cpu_ns,peak,first_nonzero_frame,mailbox_bytes,mixer_bytes\n",
    );
    for (name, enabled, events) in [
        ("quiet-idle", 3, 0),
        ("fabrication", 11, 0),
        ("thrust", 19, 0),
        ("survival", 0x1c00, 0x1c00),
        ("blackout", 1 << 7, 1 << 7),
        ("stress", (1 << VOICES) - 1, (1 << VOICES) - 1),
        ("muted", 0, 0),
    ] {
        let mb = Mailbox::default();
        let mut c = control();
        c.enabled = enabled;
        c.events = std::array::from_fn(|i| events & (1 << i) != 0);
        if name == "quiet-idle" {
            c.gains[0] = 0.03;
            c.gains[1] = 0.02;
        }
        let mut mix = Mix::new(48000);
        let mut pcm = Vec::new();
        let start = Instant::now();
        let mut peak = 0f32;
        let mut first = None;
        for block in 0..600 {
            c.events = std::array::from_fn(|i| block == 0 && events & (1 << i) != 0);
            mb.publish(c, block * 1000 / 60);
            mix.begin(&mb, block * 1000 / 60);
            for _ in 0..800 {
                let s = mix.sample();
                if first.is_none() && s.iter().any(|v| v.abs() > 1e-5) {
                    first = Some(pcm.len() / 4);
                }
                for v in s {
                    peak = peak.max(v.abs());
                    pcm.extend_from_slice(&((v * 32767.) as i16).to_le_bytes());
                }
            }
        }
        let elapsed = start.elapsed().as_nanos();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&48000u32.to_le_bytes());
        wav.extend_from_slice(&192000u32.to_le_bytes());
        wav.extend_from_slice(&4u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
        wav.extend(pcm);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        timing.push_str(&format!(
            "{name},480000,48000,{elapsed},{peak},{},{},{}\n",
            first.map_or("silent".into(), |v| v.to_string()),
            std::mem::size_of::<Mailbox>(),
            std::mem::size_of::<Mix>()
        ));
    }
    std::fs::write(root.join("timings.csv"), timing).unwrap();
}
