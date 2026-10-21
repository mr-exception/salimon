use super::*;
use salimon_survival::{Points, Survival, effects::*};
fn observation(values: [f64; 3], ms: u64) -> Observation {
    let mut s = GameplaySession::default();
    let mut v = serde_json::to_value(&s.survival).unwrap();
    for (field, value) in ["health", "hunger", "fatigue"].into_iter().zip(values) {
        v[field] = serde_json::to_value(Points::milli((value * 1000.) as u32).unwrap()).unwrap();
    }
    v["active_time"] = serde_json::to_value(Duration::from_millis(ms)).unwrap();
    s.survival = serde_json::from_value(v).unwrap();
    Observation::capture(&s)
}
#[test]
fn exact_penalties_are_separate_from_meter_values_and_authority_is_unchanged() {
    let o = observation([80., 19., 20.], 0);
    assert_eq!(
        o.penalties,
        [
            "HUNGER SPEED -25%",
            "FATIGUE SPEED -35%",
            "COMBINED SPEED -51.25%",
            "INTERACTION RATE -43.75%"
        ]
    );
    let mut s = GameplaySession::default();
    let before = s.save_bytes(1).unwrap();
    let mut h = Hud::default();
    for _ in 0..20 {
        let f = h.observe(Observation::capture(&s));
        h.image(f, [1280, 720], 1., true).unwrap();
    }
    assert_eq!(s.save_bytes(1).unwrap(), before);
    s.advance_game(Duration::from_millis(16));
    assert!(s.survival.snapshot().active_time > Duration::ZERO);
}
#[test]
fn recovery_tweens_survive_passive_drain_and_feedback_expires_without_fake_initial_gain() {
    let mut h = Hud::default();
    let f = h.observe(observation([50., 20., 100.], 0));
    assert!(f.gains.is_empty());
    let f = h.observe(observation([55., 60., 100.], 16));
    assert_eq!(f.gains, ["+5 HEALTH", "+40 HUNGER"]);
    assert_eq!(f.meters[1].fill, 20);
    let f = h.observe(observation([55., 59.999, 100.], 241));
    assert!((39..=41).contains(&f.meters[1].fill));
    let f = h.observe(observation([55., 59.99, 100.], 1300));
    assert_eq!(f.meters[1].fill, 60);
    assert!(f.gains.is_empty());
    let f = h.observe(observation([10., 59.98, 100.], 1316));
    assert_eq!(f.meters[0].fill, 10); // damage is immediate
    let mut new = observation([100.; 3], 2000);
    new.serial = 2;
    assert!(h.observe(new).gains.is_empty());
}
#[test]
fn critical_colors_warnings_and_hysteresis_follow_exact_boundaries() {
    let mut h = Hud::default();
    assert_eq!(
        h.observe(observation([25., 20., 20.001], 0))
            .meters
            .map(|m| m.tone),
        [Tone::Low, Tone::Low, Tone::Low]
    );
    let f = h.observe(observation([24.999, 19.999, 20.], 16));
    assert_eq!(f.meters.map(|m| m.tone), [Tone::Critical; 3]);
    assert_eq!(f.warning, Some("LOW HEALTH"));
    assert!(
        h.observe(observation([24., 19., 19.], 4000))
            .warning
            .is_none()
    );
    h.observe(observation([26., 19., 19.], 4016));
    assert!(
        h.observe(observation([24., 19., 19.], 4032))
            .warning
            .is_none()
    );
    h.observe(observation([27., 19., 19.], 4048));
    assert_eq!(
        h.observe(observation([24., 19., 19.], 4064)).warning,
        Some("LOW HEALTH")
    );
}
#[test]
fn typed_effect_details_preserve_amount_stacks_duration_removal_and_source() {
    let rule = EffectRule {
        id: EffectId(7),
        behavior: Behavior::Damage {
            source: Source::Burning,
            amount: Points::whole(2).unwrap(),
            wound: true,
        },
        lifetime: Lifetime::Timed(Duration::from_secs(10)),
        interval: Some(Duration::from_secs(1)),
        max_stacks: 2,
        removal: Removal::TreatmentOnly,
    };
    let mut s = GameplaySession::default();
    s.survival = Survival::with_rules(
        Default::default(),
        Rules {
            effects: vec![rule],
            ..Default::default()
        },
    )
    .unwrap();
    s.survival.apply_effect(EffectId(7)).unwrap();
    s.survival.apply_effect(EffectId(7)).unwrap();
    s.survival.advance(Duration::from_millis(500));
    let o = Observation::capture(&s);
    assert!(
        o.statuses[0]
            .description
            .contains("BURNING -4.0 HEALTH / 1.000S + WOUND")
    );
    assert!(o.statuses[0].description.contains("X2"));
    assert_eq!(o.statuses[0].duration, "10S LEFT / TREATMENT ONLY");
    let before = s.save_bytes(1).unwrap();
    assert_eq!(s.survival.effect_snapshots().next().unwrap().stacks, 2);
    assert_eq!(s.save_bytes(1).unwrap(), before);
}
#[test]
fn hiding_for_views_crafting_death_and_diagnostics_closes_details() {
    let mut s = GameplaySession::default();
    assert!(Observation::capture(&s).visible);
    s.crafting_ui.open = true;
    assert!(!Observation::capture(&s).visible);
    s.crafting_ui.open = false;
    s.view_mode = ViewMode::PrecisionTour;
    assert!(!Observation::capture(&s).visible);
    s.view_mode = ViewMode::Gameplay;
    s.survival
        .command(salimon_survival::Command::Damage(Points::FULL))
        .unwrap();
    assert!(!Observation::capture(&s).visible);
    let mut h = Hud::default();
    let f = h.observe(observation([100.; 3], 0));
    h.toggle_details();
    assert!(h.image(f, [1280, 720], 1., false).is_none());
    assert!(!h.details_open);
}
#[test]
fn cache_does_not_upload_an_unchanged_healthy_frame_and_small_surfaces_hide_safely() {
    let mut h = Hud::default();
    let f = h.observe(observation([100.; 3], 0));
    let a = h.image(f, [1280, 720], 1., true).unwrap().revision;
    let f = h.observe(observation([100.; 3], 800));
    let b = h.image(f.clone(), [1280, 720], 1., true).unwrap().revision;
    assert_eq!(a, b);
    assert!(h.image(f, [16, 16], 1., true).is_none());
}
#[test]
fn raster_evidence_and_exact_rgba_layout() {
    let mut h = Hud::default();
    let f = h.observe(observation([100.; 3], 0));
    let image = h.image(f, [1280, 720], 1., true).unwrap();
    assert_eq!(image.rgba8.len(), (image.width * image.height * 4) as usize);
    let save = |name: &str, image: salimon_renderer::OverlayImage<'_>| {
        if let Ok(path) = std::env::var("SALIMON_HUD_EVIDENCE_DIR") {
            let root = std::path::Path::new(&path);
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(root.join(format!("{name}.rgba")), image.rgba8).unwrap();
            std::fs::write(
                root.join(format!("{name}.json")),
                format!("{{\"width\":{},\"height\":{}}}", image.width, image.height),
            )
            .unwrap();
        }
    };
    save("healthy", image);
    h.observe(observation([19., 10., 0.], 1000));
    let mut o = observation([24., 50., 0.], 1016);
    let mut statuses = Vec::new();
    for i in 0..9 {
        statuses.push(Status {
            icon: Icon::Damage,
            description: format!("#{i}, X2: BURNING -4.0 HEALTH / 1.000S + WOUND"),
            duration: "10S LEFT / TREATMENT ONLY".into(),
        });
    }
    o.statuses.extend(statuses);
    let f = h.observe(o);
    h.toggle_details();
    let image = h.image(f, [1280, 720], 1., true).unwrap();
    save("critical-details", image);
}

#[test]
fn real_exact_slot_ration_and_medkit_recovery_drive_feedback_and_full_meter_waste_does_not() {
    use crate::session::consumables::Selection;
    use salimon_fabrication::{ConsumableSlot, JobId, Mass, OutputCategory, OutputItem, RecipeId};
    let mut s = GameplaySession::default();
    let mut vitals = serde_json::to_value(&s.survival).unwrap();
    vitals["hunger"] = serde_json::to_value(Points::whole(10).unwrap()).unwrap();
    s.survival = serde_json::from_value(vitals).unwrap();
    let mut hud = Hud::default();
    hud.observe(Observation::capture(&s));
    let item = |recipe: &str, job, category| OutputItem {
        recipe: RecipeId::new(recipe).unwrap(),
        job: JobId(job),
        unit: 1,
        mass: Mass::from_milligrams(1000).unwrap(),
        category,
    };
    s.consumables
        .insert(item("food-ration", 1, OutputCategory::Food))
        .unwrap();
    s.consume_selection(Selection {
        slot: ConsumableSlot::FoodOne,
        job: JobId(1),
        unit: 1,
    })
    .unwrap();
    assert_eq!(hud.observe(Observation::capture(&s)).gains, ["+40 HUNGER"]);
    assert!(s.consumables.food().iter().all(Option::is_none));
    s.survival
        .command(salimon_survival::Command::Damage(
            Points::whole(40).unwrap(),
        ))
        .unwrap();
    hud.observe(Observation::capture(&s));
    s.consumables
        .insert(item("medkit", 2, OutputCategory::Medical))
        .unwrap();
    s.consume_selection(Selection {
        slot: ConsumableSlot::MedicalOne,
        job: JobId(2),
        unit: 1,
    })
    .unwrap();
    assert!(
        Observation::capture(&s)
            .statuses
            .iter()
            .any(|e| e.icon == Icon::Recovery && e.description.contains("+2.5"))
    );
    s.advance_game(Duration::from_secs(1));
    assert!(
        hud.observe(Observation::capture(&s))
            .gains
            .contains(&"+2.5 HEALTH".into())
    );
    let mut full = GameplaySession::default();
    let mut hud = Hud::default();
    hud.observe(Observation::capture(&full));
    full.consumables
        .insert(item("food-ration", 1, OutputCategory::Food))
        .unwrap();
    full.consume_selection(Selection {
        slot: ConsumableSlot::FoodOne,
        job: JobId(1),
        unit: 1,
    })
    .unwrap();
    assert!(hud.observe(Observation::capture(&full)).gains.is_empty());
}
#[test]
fn status_punctuation_is_visible_and_maximum_typed_descriptions_fit_without_truncation() {
    for ch in "+:#,[]<".chars() {
        assert_ne!(crate::action_bar::glyph_rows(ch), [0; 7]);
    }
    let mut rule = EffectRule {
        id: EffectId(u16::MAX),
        behavior: Behavior::Damage {
            source: Source::Registered(u16::MAX),
            amount: Points::FULL,
            wound: true,
        },
        lifetime: Lifetime::Timed(Duration::from_secs(86400)),
        interval: Some(Duration::from_secs(86400)),
        max_stacks: 100,
        removal: Removal::TreatmentOnly,
    };
    for behavior in [
        rule.behavior.clone(),
        Behavior::Feed {
            amount: Points::FULL,
        },
        Behavior::Heal {
            amount: Points::FULL,
            mode: Healing::Treatment,
        },
        Behavior::Amplify {
            source: Source::Registered(u16::MAX),
            basis_points: 50000,
        },
    ] {
        rule.behavior = behavior;
        rule.interval = if matches!(rule.behavior, Behavior::Amplify { .. }) {
            None
        } else {
            Some(Duration::from_secs(86400))
        };
        let mut rules = Rules {
            effects: vec![rule.clone()],
            ..Default::default()
        };
        rules
            .sources
            .push(SourceRule::unmodified(Source::Registered(u16::MAX), 9));
        let mut s = GameplaySession::default();
        s.survival = Survival::with_rules(Default::default(), rules).unwrap();
        for _ in 0..100 {
            s.survival.apply_effect(rule.id).unwrap();
        }
        let o = Observation::capture(&s);
        assert!(
            o.statuses[0].description.chars().count() <= 84,
            "{}",
            o.statuses[0].description
        );
        let lines = super::raster::description_lines(&o.statuses[0].description);
        assert!(lines.len() <= 2);
        assert!(lines.iter().all(|l| l.len() <= 42));
        assert!(o.statuses[0].duration.chars().count() <= 42);
    }
}
