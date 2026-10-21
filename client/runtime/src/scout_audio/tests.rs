use super::*;
fn observation() -> Observation {
    Observation {
        active: true,
        interior: true,
        walking: true,
        eye: [0., 1.8, 0.],
        right: [0., 0., -1.],
        powered: true,
        conversion: false,
        fabrication: false,
        thrust: 0.,
        door: 0.,
        low_power: false,
        serial: 1,
        now: Duration::ZERO,
    }
}
fn cues(voices: &[Voice]) -> Vec<Cue> {
    voices.iter().map(|v| v.cue).collect()
}
#[test]
fn maps_machinery_transitions_without_repeating_alerts() {
    let settings = Settings::default();
    let mut m = Mixer::default();
    let mut o = observation();
    assert!(m.observe(o, settings));
    assert_eq!(cues(&m.voices(settings)), [Cue::Core]);
    o.now += Duration::from_millis(16);
    o.conversion = true;
    o.fabrication = true;
    o.thrust = 0.8;
    o.door = 0.1;
    o.low_power = true;
    m.observe(o, settings);
    let sounds = cues(&m.voices(settings));
    for cue in [
        Cue::LowPower,
        Cue::Core,
        Cue::Conversion,
        Cue::Fabricator,
        Cue::Thrust,
        Cue::Ramp,
    ] {
        assert!(sounds.contains(&cue));
    }
    assert!(!cues(&m.voices(settings)).contains(&Cue::LowPower));
    o.now += Duration::from_secs(1);
    o.powered = false;
    o.thrust = 0.;
    m.observe(o, settings);
    assert_eq!(cues(&m.voices(settings)), [Cue::Blackout]);
    assert!(m.voices(settings).is_empty());
    o.powered = true;
    m.observe(o, settings);
    assert!(cues(&m.voices(settings)).contains(&Cue::Restored));
}
#[test]
fn silence_and_resume_clear_queued_events_and_run_changes_do_not_replay() {
    let mut m = Mixer::default();
    let o = observation();
    let settings = Settings::default();
    for mode in 0..5 {
        m.observe(o, settings);
        m.queue(Cue::Blackout);
        let mut s = settings;
        let mut suppressed = o;
        match mode {
            0 => suppressed.active = false,
            1 => suppressed.interior = false,
            2 => s.master_mute = true,
            3 => s.scene_audio = false,
            _ => s.scene_volume_percent = 0,
        }
        assert!(!m.observe(suppressed, s));
        assert!(m.voices(s).is_empty());
        m.observe(o, settings);
        assert_eq!(cues(&m.voices(settings)), [Cue::Core]);
    }
    m.queue(Cue::Blackout);
    let mut fresh = o;
    fresh.serial = 2;
    m.observe(fresh, settings);
    assert_eq!(cues(&m.voices(settings)), [Cue::Core]);
    fresh.now = Duration::from_secs(2);
    m.observe(fresh, settings);
    m.queue(Cue::Blackout);
    fresh.now = Duration::ZERO;
    m.observe(fresh, settings);
    assert_eq!(cues(&m.voices(settings)), [Cue::Core]);
}
#[test]
fn footsteps_follow_local_distance_and_reject_drift_and_teleports() {
    let mut m = Mixer::default();
    let mut o = observation();
    let s = Settings::default();
    m.observe(o, s);
    o.eye[0] += 0.4;
    m.observe(o, s);
    assert!(!cues(&m.voices(s)).contains(&Cue::Footstep));
    o.eye[0] += 0.41;
    m.observe(o, s);
    assert!(cues(&m.voices(s)).contains(&Cue::Footstep));
    m.observe(o, s);
    assert!(!cues(&m.voices(s)).contains(&Cue::Footstep));
    o.walking = false;
    o.eye[0] += 1.;
    m.observe(o, s);
    assert!(!cues(&m.voices(s)).contains(&Cue::Footstep));
    o.walking = true;
    o.eye[0] += 100.;
    m.observe(o, s);
    assert!(!cues(&m.voices(s)).contains(&Cue::Footstep));
}
#[test]
fn spatial_channels_rotate_with_listener_and_attenuate_with_distance() {
    let (_, right) = spatial([0.; 3], [0., 0., -1.], [0., 0., -3.]);
    assert_eq!(right, 1.);
    let (_, left) = spatial([0.; 3], [0., 0., 1.], [0., 0., -3.]);
    assert_eq!(left, -1.);
    assert!(
        spatial([0.; 3], [0., 0., -1.], [2., 0., 0.]).0
            > spatial([0.; 3], [0., 0., -1.], [10., 0., 0.]).0
    );
    assert_eq!(spatial([0.; 3], [0., 0., -1.], [21., 0., 0.]), (0., 0.));
    assert_eq!(
        spatial([f64::NAN, 0., 0.], [0., 0., -1.], [0.; 3]),
        (0., 0.)
    );
}
#[test]
fn pcm_is_bounded_stereo_and_mute_has_no_samples() {
    let v = Voice {
        cue: Cue::Core,
        gain: 0.8,
        pan: 1.,
    };
    let wav = synth::wav(&[v], 0);
    assert_eq!(wav.len(), synth::WAV_BYTES);
    assert_eq!(&wav[0..4], b"RIFF");
    assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 2);
    assert_eq!(wav, synth::wav(&[v], 0));
    let samples: Vec<[i16; 2]> = wav[44..]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| {
            [
                i16::from_le_bytes([p[0], p[1]]),
                i16::from_le_bytes([p[2], p[3]]),
            ]
        })
        .collect();
    assert!(samples.iter().all(|p| p[0] == 0));
    assert!(samples.iter().any(|p| p[1] != 0));
    let loud = synth::wav(
        &[Voice {
            cue: Cue::Thrust,
            gain: 10.,
            pan: 0.,
        }; MAX_VOICES + 4],
        0,
    );
    assert!(
        loud[44..]
            .as_chunks::<2>()
            .0
            .iter()
            .all(|p| i16::from_le_bytes([p[0], p[1]]).abs() <= 30000)
    );
    assert!(synth::wav(&[], 0)[44..].iter().all(|v| *v == 0));
}
#[test]
fn event_queue_and_mix_are_bounded_with_alert_priority() {
    let mut m = Mixer::default();
    let s = Settings::default();
    let o = observation();
    m.observe(o, s);
    for _ in 0..1000 {
        m.queue(Cue::Footstep);
    }
    m.queue(Cue::Blackout);
    assert_eq!(m.pending.len(), MAX_EVENTS);
    assert_eq!(m.pending.front(), Some(&Cue::Blackout));
    let v = m.voices(s);
    assert!(v.len() <= MAX_VOICES);
    assert_eq!(v[0].cue, Cue::Blackout);
}
#[test]
fn production_core_recovery_and_pause_map_without_authority_changes() {
    use crate::e2e::{Config, Scenario, initialize};
    let mut session = GameplaySession::default();
    initialize(
        &mut session,
        Config {
            scenario: Scenario::CoreRestart,
            seed: 0,
            step: Duration::from_millis(100),
            fragment_load: None,
        },
    )
    .unwrap();
    let before = session.ship.snapshot();
    let mut m = Mixer::default();
    let settings = Settings::default();
    let off = Observation::capture(&session, true);
    assert!(!off.powered);
    m.observe(off, settings);
    assert!(m.voices(settings).is_empty());
    assert_eq!(session.ship.snapshot(), before);
    session.interact();
    session.advance_game(Duration::from_millis(100));
    let on = Observation::capture(&session, true);
    assert!(on.powered);
    assert!(on.conversion);
    m.observe(on, settings);
    let sounds = cues(&m.voices(settings));
    assert!(sounds.contains(&Cue::Restored));
    assert!(sounds.contains(&Cue::Conversion));
    session.toggle_pause();
    assert!(!Observation::capture(&session, true).active);
    session.toggle_pause();
    assert!(!Observation::capture(&session, false).active);
}

#[test]
fn narration_free_pcm_synthesis_has_fixed_memory_and_measured_cost() {
    let voice = Voice {
        cue: Cue::Thrust,
        gain: 0.1,
        pan: 0.0,
    };
    let start = Instant::now();
    for sequence in 0..32 {
        let wav = synth::wav(&[voice; MAX_VOICES], sequence);
        assert_eq!(wav.len(), synth::WAV_BYTES);
    }
    eprintln!(
        "Scout 8-voice synthesis: 32 half-second chunks in {} us, {} bytes/chunk",
        start.elapsed().as_micros(),
        synth::WAV_BYTES
    );
}
