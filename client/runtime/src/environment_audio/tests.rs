use super::*;
use salimon_world::{
    CELESTIAL_BODIES, CelestialBodyId,
    environment::{
        activity::ActivityKind,
        climate::{Chemistry, ClimateInputs},
        ecology::{BiosphereDecision, BiosphereDefinition, BodyNature},
    },
    terrain::{
        TerrainDefinition,
        geology::{GeologyDefinition, GeologyProfile},
    },
};
fn fixture(vacuum: bool, active: bool) -> (ClimateDefinition, ActivityDefinition, Population) {
    let body = *CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Moon)
        .unwrap();
    let terrain = TerrainDefinition::new(body, 1, 0, vec![], vec![]).unwrap();
    let geology = GeologyDefinition::new(terrain, 1, 0, GeologyProfile::DryRock).unwrap();
    let inputs = if vacuum {
        ClimateInputs::vacuum(86400., 31557600., 1361.)
    } else {
        ClimateInputs {
            chemistry: Chemistry::Nitrogen,
            condensable: Condensable::Water,
            pressure_pascal: 101325.,
            scale_height_meters: 8000.,
            rotation_seconds: 86400.,
            year_seconds: 31557600.,
            axial_tilt_radians: 0.4,
            stellar_flux_w_m2: 1361.,
            albedo: 0.3,
            greenhouse_kelvin: 30.,
            condensable_fraction: 0.8,
        }
    };
    let climate = ClimateDefinition::new(geology, 1, 37, inputs).unwrap();
    let activity = ActivityDefinition::new(
        climate.clone(),
        1,
        83,
        if active {
            ActivityInputs {
                kind: ActivityKind::Volcanic,
                candidate_count: 32,
                heat_flux_w_m2: 0.2,
                site_radius_meters: 5.,
                cycle_seconds: 100.,
                active_fraction: 0.5,
            }
        } else {
            ActivityInputs::dormant()
        },
    )
    .unwrap();
    let b = BiosphereDefinition::new(
        salimon_universe::CatalogId::from_source("fictional-fixture", "audio-world").unwrap(),
        BodyNature::Fictional,
        if vacuum {
            BiosphereDecision::Sterile
        } else {
            BiosphereDecision::Developed
        },
        climate.clone(),
        1,
        17,
    )
    .unwrap();
    let population = Population::publish(&b, &[]).unwrap();
    (climate, activity, population)
}
fn epoch(seconds: i64) -> Epoch {
    crate::session::field_marks::epoch(std::time::Duration::from_secs(seconds as u64)).unwrap()
}
fn listener(eye: [f64; 3]) -> Listener {
    Listener {
        eye,
        right: [1., 0., 0.],
        contact: false,
        transmission: 0.45,
        filter: 0.35,
    }
}
fn evaluate(
    f: &(ClimateDefinition, ActivityDefinition, Population),
    l: Listener,
    seconds: i64,
) -> Frame {
    sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: None,
        },
        epoch(seconds),
        l,
        0.,
        |_| false,
    )
}
#[test]
fn vacuum_is_quiet_and_ground_activity_requires_actual_contact_and_range() {
    let f = fixture(true, true);
    let site = &f.1.sites()[0];
    let l = listener(site.body_local_position_meters);
    let seconds = (0..100)
        .find(|&s| f.1.sample(&site.id, epoch(s)).unwrap().intensity > 0.5)
        .unwrap();
    assert!(evaluate(&f, l, seconds).bands.iter().all(|b| b.gain == 0.));
    let contact = evaluate(&f, Listener { contact: true, ..l }, seconds);
    assert!(contact.bands[2].gain > 0.);
    assert!(contact.bands[2].filter <= 0.015);
    assert!(contact.bands[..2].iter().all(|b| b.gain == 0.));
    let far = listener(site.body_local_position_meters.map(|v| -v));
    assert_eq!(
        evaluate(
            &f,
            Listener {
                contact: true,
                ..far
            },
            seconds
        )
        .bands[2]
            .gain,
        0.
    );
}
#[test]
fn local_weather_drives_wind_alignment_precipitation_and_no_remote_storm() {
    let f = fixture(false, false);
    let n = f.0.geology().terrain().radius_meters();
    let l = listener([0.8 * n, 0.6 * n, 0.]);
    let a = evaluate(&f, l, 250);
    let b = evaluate(
        &f,
        Listener {
            right: [-1., 0., 0.],
            ..l
        },
        250,
    );
    assert!(a.bands[0].gain > 0.);
    assert_eq!(a.bands[0].gain, b.bands[0].gain);
    assert_eq!(a.bands[0].pan, -b.bands[0].pan);
    let weather = f.0.sample([0.8, 0.6, 0.], epoch(250)).unwrap();
    assert!(
        (a.bands[1].gain - (weather.precipitation_kg_m2_s * 30.).min(1.) as f32 * 0.18 * 0.45)
            .abs()
            < 1e-6
    );
    let no_dust = sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: None,
        },
        epoch(250),
        l,
        0.,
        |_| false,
    );
    let dust = sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: None,
        },
        epoch(250),
        l,
        0.001,
        |_| false,
    );
    assert!(dust.bands[1].gain >= no_dust.bands[1].gain);
    let space = evaluate(&f, listener([0., n + 1e6, 0.]), 250);
    assert!(space.bands[0].gain < 1e-20);
    // A remote weather sample cannot enter: changing direction changes only the local query.
    let far = evaluate(&f, listener([0., -n, 0.]), 250);
    let w = f.0.sample([0., -1., 0.], epoch(250)).unwrap();
    assert_eq!(
        far.warning,
        (w.storm_fraction > 0.7).then_some("LOCAL STORM / REDUCED VISIBILITY")
    );
}
#[test]
fn biological_rustle_uses_physical_population_and_occlusion_not_visual_density() {
    let f = fixture(false, false);
    let o =
        f.2.objects()
            .iter()
            .find(|o| matches!(o.specimen.form, Form::Frond | Form::Filament))
            .unwrap();
    let l = listener(o.position_meters);
    let a = evaluate(&f, l, 50);
    let blocked = sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: None,
        },
        epoch(50),
        l,
        0.,
        |_| true,
    );
    assert!(a.bands[3].gain > 0.);
    assert!(blocked.bands[3].gain < a.bands[3].gain);
    let before = f.2.binding();
    let again = evaluate(&f, l, 50);
    assert_eq!(again.bands[3].gain, a.bands[3].gain);
    assert_eq!(before, f.2.binding());
}
#[test]
fn mix_settings_material_filters_and_lifecycle_remain_observational() {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    let before = s.save_bytes(100).unwrap();
    let o = crate::scout_audio::Observation::capture(&s, true);
    let mut p = crate::scout_audio::Presenter::default();
    p.preview(o, Settings::default());
    assert!(
        p.telemetry()["scene_gains"].as_array().unwrap()[27..]
            .iter()
            .all(|v| v.as_f64() == Some(0.))
    );
    assert_eq!(before, s.save_bytes(100).unwrap());
    let settings = Settings {
        master_mute: true,
        ..Default::default()
    };
    p.preview(o, settings);
    assert!(
        p.telemetry()["scene_gains"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_f64() == Some(0.))
    );
    s.toggle_pause();
    p.preview(
        crate::scout_audio::Observation::capture(&s, true),
        Settings::default(),
    );
    assert!(
        p.telemetry()["scene_gains"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v.as_f64() == Some(0.))
    );
    let mut frame = Frame::default();
    frame.bands[4] = harvest(salimon_world::resources::ResourceId::OrganicBiomass);
    assert!(frame.bands[4].filter < harvest(salimon_world::resources::ResourceId::IronOre).filter);
    for settings in [
        Settings {
            environment_audio: false,
            ..Default::default()
        },
        Settings {
            environment_volume_percent: 0,
            ..Default::default()
        },
    ] {
        let mut c = Default::default();
        apply(frame, &mut c, settings);
        assert_eq!(c.gains[31], 0.);
    }
}
#[test]
fn scout_actual_partition_geometry_and_airborne_geology_attenuate_sources() {
    let open = shelter([0., 1.8, 0.], 1., 1., 1.);
    let closed = shelter([0., 1.8, 0.], 0., 0., 1.);
    let half = shelter([0., 1.8, 0.], 0.5, 1., 1.);
    assert_eq!(open.0, 1.);
    assert!((open.1 - 0.6).abs() < 1e-15);
    assert!(closed.0 < half.0 && half.0 < open.0);
    assert!(closed.1 < half.1 && half.1 < open.1);
    let f = fixture(false, true);
    let site = &f.1.sites()[0];
    let l = listener(site.body_local_position_meters);
    let seconds = (0..100)
        .find(|&s| f.1.sample(&site.id, epoch(s)).unwrap().intensity > 0.5)
        .unwrap();
    let a = evaluate(&f, l, seconds);
    let blocked = sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: None,
        },
        epoch(seconds),
        l,
        0.,
        |_| true,
    );
    assert!(a.bands[2].gain > 0.);
    assert!(blocked.bands[2].gain < a.bands[2].gain);
}
#[test]
#[ignore = "writes production callback WAV evidence"]
fn export_environment_audio_627() {
    let root = std::path::PathBuf::from(std::env::var_os("SALIMON_AUDIO_EVIDENCE").unwrap());
    std::fs::create_dir_all(&root).unwrap();
    let f = fixture(false, false);
    // Choose an actual strongest local weather sample; never amplify/invent storm state.
    let mut best = (0., [1., 0., 0.], 0);
    for latitude in -7..8 {
        let phi = f64::from(latitude) * 0.19;
        for longitude in 0..16 {
            let theta = f64::from(longitude) * std::f64::consts::TAU / 16.;
            let d = [phi.cos() * theta.cos(), phi.sin(), phi.cos() * theta.sin()];
            for seconds in [0, 1000, 10000, 100000] {
                let w = f.0.sample(d, epoch(seconds)).unwrap();
                let score = length(w.wind_mps) + w.precipitation_kg_m2_s * 1000.;
                if score > best.0 {
                    best = (score, d, seconds);
                }
            }
        }
    }
    let point =
        f.0.geology()
            .terrain()
            .sample(best.1)
            .unwrap()
            .body_local_position_meters;
    let l = listener(point);
    std::fs::write(root.join("source-context.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "fixture":"fictional-fixture/audio-world; seed37 climate, seed17 population",
        "climate_identity":f.0.identity(), "population_identity":f.2.identity(),
        "body_fixed_listener_meters":point,"shared_tt_seconds":best.2,"weather":format!("{:?}",f.0.sample(best.1,epoch(best.2)).unwrap()),
        "synthesis":"production native_audio::Mix, 125 controls at 16ms, not OS playback"
    })).unwrap()).unwrap();
    let mut metrics = serde_json::Map::new();
    for (name, transmission, filter, material) in [
        ("suit-local-weather", 0.45, 0.35, None),
        ("scout-ramp-open", 1., 0.6, None),
        ("scout-ramp-closed", 0.025, 0.04, None),
        (
            "organic-contact-harvest",
            0.,
            0.,
            Some(salimon_world::resources::ResourceId::OrganicBiomass),
        ),
        (
            "mineral-contact-harvest",
            0.,
            0.,
            Some(salimon_world::resources::ResourceId::IronOre),
        ),
    ] {
        let mut frame = evaluate(
            &f,
            Listener {
                transmission,
                filter,
                ..l
            },
            best.2,
        );
        if let Some(m) = material {
            frame.bands[4] = harvest(m);
        }
        let mut control = Default::default();
        apply(frame, &mut control, Settings::default());
        let controls = vec![control; 125];
        let (wav, metric) = crate::native_audio::sequence_wav(&controls);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        metrics.insert(name.into(), metric);
    }
    let geological = fixture(true, true);
    let site = &geological.1.sites()[0];
    let seconds = (0..100)
        .find(|&s| geological.1.sample(&site.id, epoch(s)).unwrap().intensity > 0.5)
        .unwrap();
    let organism =
        f.2.objects()
            .iter()
            .find(|o| matches!(o.specimen.form, Form::Frond | Form::Filament))
            .unwrap();
    for (name, frame) in [
        (
            "vacuum-contact-geology",
            evaluate(
                &geological,
                Listener {
                    contact: true,
                    ..listener(site.body_local_position_meters)
                },
                seconds,
            ),
        ),
        (
            "physical-flexible-biology",
            evaluate(&f, listener(organism.position_meters), 50),
        ),
    ] {
        let mut control = Default::default();
        apply(frame, &mut control, Settings::default());
        let (wav, metric) = crate::native_audio::sequence_wav(&vec![control; 125]);
        std::fs::write(root.join(format!("{name}.wav")), wav).unwrap();
        metrics.insert(name.into(), metric);
    }
    let (wav, metric) = crate::native_audio::sequence_wav(&vec![Default::default(); 125]);
    std::fs::write(root.join("vacuum-quiet.wav"), wav).unwrap();
    metrics.insert("vacuum-quiet".into(), metric);
    std::fs::write(
        root.join("metrics.json"),
        serde_json::to_vec_pretty(&metrics).unwrap(),
    )
    .unwrap();
}

#[test]
fn removed_physical_anatomy_cannot_keep_emitting_rustle() {
    let f = fixture(false, false);
    let o =
        f.2.objects()
            .iter()
            .find(|o| matches!(o.specimen.form, Form::Frond | Form::Filament))
            .unwrap();
    let l = listener(o.position_meters);
    let gone = |_: &str, _: u16| 0.;
    let intact = evaluate(&f, l, 50);
    let removed = sample(
        &f.0,
        &f.1,
        Anatomy {
            population: &f.2,
            remaining: Some(&gone),
        },
        epoch(50),
        l,
        0.,
        |_| false,
    );
    assert!(intact.bands[3].gain > 0.);
    assert_eq!(removed.bands[3].gain, 0.);
}
