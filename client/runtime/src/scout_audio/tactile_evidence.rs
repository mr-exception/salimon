//! Reproducible original PCM and real CPAL output probes; no listening approval.
use super::*;
use crate::{e2e, session::Command};
fn setup(scenario: e2e::Scenario) -> GameplaySession {
    let mut s = GameplaySession::default();
    e2e::initialize(
        &mut s,
        e2e::Config {
            scenario,
            seed: 583,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )
    .unwrap();
    s
}
fn control(s: &GameplaySession, m: &mut Mixer) -> crate::native_audio::Control {
    let settings = Settings::default();
    let o = Observation::capture(s, true);
    m.observe(o, settings);
    controls(m.voices(settings), s.audio_epoch, true, None, settings)
}
#[test]
#[ignore = "exports actual production mechanism control sequences and continuous PCM"]
fn tactile_audio_evidence_583() {
    use std::path::PathBuf;
    let root = PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    let mut rows = Vec::new();
    for (name, scenario) in [
        ("table", e2e::Scenario::ScoutCabinTable),
        ("locker", e2e::Scenario::ScoutCabinLocker),
        ("bunk", e2e::Scenario::ScoutCabinBunk),
        ("shutter", e2e::Scenario::ScoutLightingShutter),
        ("shutter-blackout", e2e::Scenario::ScoutLightingBlackout),
    ] {
        let mut s = setup(scenario);
        let mut m = Mixer::default();
        let mut sequence = vec![control(&s, &mut m)];
        let mut events = Vec::new();
        s.command(Command::Interact);
        for tick in 0..180 {
            if tick == 22 {
                s.command(Command::Interact);
            } // actual early reversal (debounce admitted)
            if tick == 85 {
                s.command(Command::Interact);
            }
            if tick == 130 {
                let bytes = s.save_bytes(100).unwrap();
                s.restore_bytes(&bytes).unwrap();
            }
            s.advance_automation(Duration::from_millis(16));
            let c = control(&s, &mut m);
            events.push(serde_json::json!({"tick":tick,"fractions":Observation::capture(&s,true).motions,"events":c.events.iter().enumerate().filter_map(|(i,on)|on.then_some(i)).collect::<Vec<_>>(),"loops":(13..20).filter(|i|c.gains[*i]>0.).collect::<Vec<_>>(),"epoch":s.audio_epoch}));
            sequence.push(c);
        }
        let (wav, metrics) = crate::native_audio::sequence_wav(&sequence);
        let (repeat, _) = crate::native_audio::sequence_wav(&sequence);
        assert_eq!(wav, repeat);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        std::fs::write(
            root.join(format!("{name}-events.json")),
            serde_json::to_vec_pretty(&events).unwrap(),
        )
        .unwrap();
        rows.push(serde_json::json!({"case":name,"metrics":metrics}));
    }
    // Matched same-state listeners on either side of physical partitions.
    for (name, eye, door, screen) in [
        ("cargo-door-open", [-6., 1.8, 0.], 1., 1.),
        ("cargo-door-closed", [-6., 1.8, 0.], 0., 1.),
        ("privacy-open", [-0.8, 1.8, -3.5], 1., 1.),
        ("privacy-closed", [-0.8, 1.8, -3.5], 1., 0.),
    ] {
        let mut o = super::tests::observation();
        o.eye = eye;
        o.motions[6] = door;
        o.motions[3] = screen;
        o.fabrication = true;
        o.conversion = true;
        let settings = Settings::default();
        let mut m = Mixer::default();
        m.observe(o, settings);
        let c = controls(m.voices(settings), 0, true, None, settings);
        let (wav, metrics) = crate::native_audio::sequence_wav(&vec![c; 120]);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        rows.push(serde_json::json!({"case":name,"metrics":metrics,"gain":c.gains,"filters":c.filters,"pan":c.pans}));
    }
    // Isolated callback timbre references (synthetic controls, not walked routes).
    for (name, index) in [
        ("seat-timbre", 15),
        ("privacy-screen-timbre", 16),
        ("cargo-door-timbre", 19),
        ("mount-clamp-timbre", 22),
        ("restraint-timbre", 23),
        ("habitation-footstep-timbre", 6),
        ("cargo-footstep-timbre", 24),
    ] {
        let mut c = crate::native_audio::Control {
            enabled: 1 << index,
            ..Default::default()
        };
        c.gains[index] = 0.3;
        c.events[index] = true;
        let mut seq = Vec::new();
        for tick in 0..120 {
            seq.push(c);
            c.events[index] = false;
            if tick == 60 {
                c.enabled = 0;
            }
        }
        let (wav, metrics) = crate::native_audio::sequence_wav(&seq);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        rows.push(serde_json::json!({"case":name,"scope":"isolated callback timbre reference; synthetic controls","metrics":metrics}));
    }
    // Simultaneous worst bounded mix, with explicit controls rather than physical motion claims.
    let mut c = crate::native_audio::Control {
        enabled: (1 << 27) - 1,
        gains: [0.3; 27],
        events: [true; 27],
        ..Default::default()
    };
    let mut seq = Vec::new();
    for _ in 0..180 {
        seq.push(c);
        c.events = [false; 27];
    }
    let (wav, metrics) = crate::native_audio::sequence_wav(&seq);
    std::fs::write(root.join("simultaneous-stress.wav"), wav).unwrap();
    rows.push(serde_json::json!({"case":"simultaneous-stress","metrics":metrics}));
    std::fs::write(
        root.join("metrics.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
#[test]
#[ignore = "opens an explicitly configured audio device and records callback telemetry"]
fn tactile_native_output_583() {
    let root = std::path::PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    let mut s = setup(e2e::Scenario::ScoutLightingShutter);
    let mut m = Mixer::default();
    let mut output = crate::native_audio::Output::default();
    output.update(control(&s, &mut m));
    s.command(Command::Interact);
    for _ in 0..90 {
        s.advance_automation(Duration::from_millis(16));
        output.update(control(&s, &mut m));
        std::thread::sleep(Duration::from_millis(16));
    }
    let active = output.telemetry();
    output.silence(99);
    std::thread::sleep(Duration::from_millis(40));
    let stopped = output.telemetry();
    std::fs::write(root.join("actual-output.json"),serde_json::to_vec_pretty(&serde_json::json!({"active":active,"stopped":stopped,"scope":"actual configured CPAL output sink; virtual ALSA file sink is not physical speaker or microphone evidence"})).unwrap()).unwrap();
    assert!(!output.unavailable());
    assert!(active["callbacks"].as_u64().unwrap() > 2);
}
