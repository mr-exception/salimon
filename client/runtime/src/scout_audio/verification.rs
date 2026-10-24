//! #485: production observations, exported listening fixtures and real-child lifecycle.
use super::*;
use crate::{
    e2e,
    session::{Command, ControlKey},
};
use salimon_fabrication::{FabricationCommand, RecipeId};
use serde_json::json;

fn fixture(scenario: e2e::Scenario) -> GameplaySession {
    let mut s = GameplaySession::default();
    e2e::initialize(
        &mut s,
        e2e::Config {
            scenario,
            seed: 0,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )
    .unwrap();
    s
}
fn observe(s: &GameplaySession, m: &mut Mixer, expected: Option<Cue>) -> Vec<Voice> {
    let before = s.save_bytes(100).unwrap();
    let o = Observation::capture(s, true);
    m.observe(o, Settings::default());
    let voices = m.voices(Settings::default());
    if let Some(cue) = expected {
        assert!(
            voices.iter().any(|v| v.cue == cue),
            "missing {cue:?}: {voices:?}"
        );
    }
    assert_eq!(
        s.save_bytes(100).unwrap(),
        before,
        "audio changed authority"
    );
    voices
}
fn samples() -> Vec<(&'static str, Vec<Voice>)> {
    let mut result = Vec::new();
    let mut s = fixture(e2e::Scenario::LandedEarth);
    let mut m = Mixer::default();
    result.push(("interior", observe(&s, &mut m, Some(Cue::Core))));
    s.command(Command::Control(ControlKey::Forward, true));
    let mut steps = Vec::new();
    for _ in 0..80 {
        s.advance_game(Duration::from_millis(16));
        let voices = observe(&s, &mut m, None);
        if voices.iter().any(|v| v.cue == Cue::Footstep) {
            steps = voices;
            break;
        }
    }
    assert!(
        !steps.is_empty(),
        "production walking did not emit a footstep"
    );
    result.push(("movement", steps));
    s.command(Command::Control(ControlKey::Forward, false));
    s.command(Command::Control(ControlKey::Jump, true));
    s.advance_game(Duration::from_millis(16));
    let jump = observe(&s, &mut m, None);
    assert!(!jump.iter().any(|v| v.cue == Cue::Footstep));
    result.push(("jump", jump));

    let mut s = fixture(e2e::Scenario::CoreRestart);
    let mut m = Mixer::default();
    assert!(observe(&s, &mut m, None).is_empty());
    let held = s.mining.session.carried_id();
    assert!(held.is_some());
    s.command(Command::Interact);
    s.advance_game(Duration::from_millis(100));
    assert!(s.mining.session.carried_id().is_none());
    result.push(("restoration", observe(&s, &mut m, Some(Cue::Restored))));
    result.push((
        "core-conversion",
        observe(&s, &mut m, Some(Cue::Conversion)),
    ));
    s.ship.set_stored_core_energy_joules(0); // explicit power-loss fixture
    result.push(("blackout", observe(&s, &mut m, Some(Cue::Blackout))));
    assert!(observe(&s, &mut m, None).is_empty());
    result.push(("blackout-settled-silence", Vec::new()));

    let mut s = GameplaySession::new_run();
    let mut m = Mixer::default();
    s.execute_fabricator(FabricationCommand::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_game(Duration::from_secs(1));
    result.push((
        "fabricator-working",
        observe(&s, &mut m, Some(Cue::Fabricator)),
    ));
    s.ship.set_stored_core_energy_joules(0);
    observe(&s, &mut m, Some(Cue::Blackout));
    assert!(observe(&s, &mut m, None).is_empty());
    result.push(("fabricator-no-power-silence", Vec::new()));
    s.ship.set_stored_core_energy_joules(10_000_000_000);
    s.toggle_pause();
    assert!(observe(&s, &mut m, None).is_empty());
    result.push(("paused-silence", Vec::new()));
    s.toggle_pause();
    s.advance_game(Duration::from_secs(10));
    let blocked = observe(&s, &mut m, None);
    assert!(!blocked.iter().any(|v| v.cue == Cue::Fabricator));
    result.push(("fabricator-blocked", blocked));

    let mut s = fixture(e2e::Scenario::FlightRegression);
    let mut m = Mixer::default();
    result.push(("cockpit-coasting", observe(&s, &mut m, Some(Cue::Core))));
    s.ship.adjust_thruster(50);
    s.advance_game(Duration::from_millis(16));
    result.push(("engine-thrust", observe(&s, &mut m, Some(Cue::Thrust))));
    s.command(Command::ToggleEngines);
    assert!(
        !observe(&s, &mut m, None)
            .iter()
            .any(|v| v.cue == Cue::Thrust)
    );

    let mut s = fixture(e2e::Scenario::LandedEarth);
    let mut m = Mixer::default();
    observe(&s, &mut m, None);
    s.ship.toggle_door(); // production controller with an initially clear doorway
    s.advance_game(Duration::from_millis(100));
    result.push(("ramp", observe(&s, &mut m, Some(Cue::Ramp))));
    for (scenario, route, name) in [
        (
            e2e::Scenario::CoastingSpace,
            include_str!("../../../../scenarios/moving-eva.json"),
            "eva-silence",
        ),
        (
            e2e::Scenario::EvaApproach,
            include_str!("../../../../scenarios/nearby-eva.json"),
            "nearby-eva-silence",
        ),
        (
            e2e::Scenario::FragmentPile,
            include_str!("../../../../scenarios/fragment-pile-surface.json"),
            "surface-silence",
        ),
    ] {
        let route: serde_json::Value = serde_json::from_str(route).unwrap();
        let mut s = GameplaySession::default();
        e2e::initialize(
            &mut s,
            e2e::Config {
                scenario,
                seed: route["setup"]["seed"].as_u64().unwrap(),
                step: Duration::from_millis(route["setup"]["step_ms"].as_u64().unwrap()),
                fragment_load: None,
            },
        )
        .unwrap();
        let mut m = Mixer::default();
        let mut exterior_seen = false;
        for (i, step) in route["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
                let mut cmd = action.clone();
                cmd["id"] = json!(i);
                cmd["protocol"] = json!(1);
                let response = crate::automation::execute_session(&mut s, &cmd.to_string());
                assert_eq!(response["ok"], true, "{name} {response}");
                if !Observation::capture(&s, true).interior {
                    exterior_seen = true;
                    assert!(
                        observe(&s, &mut m, None).is_empty(),
                        "ship audio escaped cabin: {name}"
                    );
                } else {
                    observe(&s, &mut m, None);
                }
            }
        }
        assert!(exterior_seen, "route never reached outside: {name}");
        result.push((name, Vec::new()));
    }
    result
}
#[test]
fn production_loop_routes_audio_without_changing_saved_authority() {
    samples();
}

#[test]
fn terminal_restart_restore_and_focus_gates_discard_transients() {
    let mut s = GameplaySession::new_run();
    let mut m = Mixer::default();
    observe(&s, &mut m, Some(Cue::Core));
    m.queue(Cue::Blackout);
    assert!(!m.observe(Observation::capture(&s, false), Settings::default()));
    assert!(m.voices(Settings::default()).is_empty());
    observe(&s, &mut m, Some(Cue::Core));
    s.survival
        .command(salimon_survival::Command::Damage(
            salimon_survival::Points::FULL,
        ))
        .unwrap();
    assert!(observe(&s, &mut m, None).is_empty());
    let fresh = s.fresh_after_death().unwrap();
    let voices = observe(&fresh, &mut m, Some(Cue::Core));
    assert!(
        !voices
            .iter()
            .any(|v| matches!(v.cue, Cue::Blackout | Cue::Restored))
    );
    let bytes = fresh.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(
        Observation::capture(&fresh, true).powered,
        Observation::capture(&restored, true).powered
    );
    let voices = observe(&restored, &mut m, Some(Cue::Core));
    assert!(
        !voices
            .iter()
            .any(|v| matches!(v.cue, Cue::Blackout | Cue::Restored))
    );
}

#[test]
#[ignore = "exports production-observation WAVs and matched synthesis diagnostics"]
fn cloud_audio_evidence_485() {
    let root =
        PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").expect("set fresh evidence path"));
    std::fs::create_dir(&root).unwrap();
    let mut manifest = Vec::new();
    for (name, voices) in samples() {
        let wav = synth::wav(&voices, 0);
        std::fs::write(root.join(format!("{name}.wav")), &wav).unwrap();
        let channels: Vec<_> = wav[44..]
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
        let peak = channels
            .iter()
            .flatten()
            .map(|v| i32::from(*v).abs())
            .max()
            .unwrap();
        assert!(peak <= 30000);
        manifest.push(json!({"name":name,"voices":format!("{voices:?}"),"frames":channels.len(),"sample_rate":synth::RATE,"channels":2,"peak":peak,"left_energy":channels.iter().map(|v|f64::from(v[0]).powi(2)).sum::<f64>(),"right_energy":channels.iter().map(|v|f64::from(v[1]).powi(2)).sum::<f64>()}));
    }
    // Explicitly marked spatial/mute controls complement production loop samples.
    for (name, pan) in [("channel-left", -1.), ("channel-right", 1.)] {
        std::fs::write(
            root.join(format!("{name}.wav")),
            synth::wav(
                &[Voice {
                    filter: 1.,
                    cue: Cue::Core,
                    gain: 0.2,
                    pan,
                }],
                0,
            ),
        )
        .unwrap();
    }
    let mut timings = Vec::new();
    let s = GameplaySession::new_run();
    for trial in 0..5 {
        for enabled in if trial % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let mut m = Mixer::default();
            let start = Instant::now();
            for seq in 0..32 {
                let settings = Settings {
                    master_mute: !enabled,
                    ..Default::default()
                };
                m.observe(Observation::capture(&s, true), settings);
                let voices = m.voices(settings);
                if !voices.is_empty() {
                    std::hint::black_box(synth::wav(&voices, seq));
                }
            }
            timings.push(json!({"trial":trial,"enabled":enabled,"updates":32,"elapsed_us":start.elapsed().as_micros()}));
        }
    }
    std::fs::write(root.join("manifest.json"),serde_json::to_vec_pretty(&json!({"scope":"production session observations plus labeled channel controls; PCM exports, not a device recording","initialization":"seeded e2e/new-run fixtures; explicit power-loss fixture, production commands and controller adapters; not physical travel","samples":manifest,"matched_synthesis":timings,"hardware_audio":"unavailable; listening/OS latency/continuous output not verified"})).unwrap()).unwrap();
}

#[test]
#[ignore = "exports current continuous PCM driven by actual production scenario observations"]
fn continuous_production_audio_582() {
    let root = PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    for (name, voices) in samples() {
        let control = super::controls(voices, 0, true, None, Settings::default());
        std::fs::write(
            root.join(format!("{name}.wav")),
            crate::native_audio::fixture_wav(control),
        )
        .unwrap();
    }
}
