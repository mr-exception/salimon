use super::*;
use salimon_fabrication::{FabricationCommand as C, RecipeId};
use std::time::Duration;
fn recovered() -> GameplaySession {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    for resource in [
        salimon_world::resources::ResourceId::IronOre,
        salimon_world::resources::ResourceId::SilicateRock,
    ] {
        s.execute_fabricator(C::Insert(
            salimon_world::resources::RawMaterial::new(resource, 2.).unwrap(),
        ))
        .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("data-recovery-module").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_fabrication(Duration::from_secs(60));
    s.collect_fabricator_output();
    for id in RECORDS {
        s.toggle_recovery(id);
        // Same production funding adapter, isolated from movement/survival for
        // this composition fixture. No progress or Core balance is assigned.
        for _ in 0..100 {
            s.advance_recovery(Duration::from_secs(10));
        }
    }
    s
}
fn at_site(s: &mut GameplaySession, i: usize) {
    let p = site_position(i);
    let frame = s.terrain_frame(s.ship.snapshot());
    let hint = add(p, [1.8, 0., 0.]);
    s.character = salimon_character::CharacterController::on_surface(frame, hint).unwrap();
    let ship_frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
    let c = s.character.snapshot(ship_frame, frame);
    let v = sub(p, c.eye_position_meters);
    let yaw = dot(v, ship_frame.axes[2]).atan2(dot(v, ship_frame.axes[0]));
    let pitch = (dot(v, c.up.map(f64::from)) / length(v)).asin();
    s.character.apply_mouse_delta(yaw / 0.0022, -pitch / 0.0022);
}
#[test]
fn two_paid_recoveries_supply_distinct_uncertain_routes_without_earth_or_transit_claims() {
    let mut s = recovered();
    assert_eq!(s.chart.entries().count(), 2);
    assert!(length(sub(site_position(0), site_position(1))) > 14000.);
    assert!(s.chart.routes().is_empty());
    for (i, record) in RECORDS.iter().enumerate() {
        let e = s.chart.get(&site_id(i)).unwrap();
        assert!(e.observed().is_none());
        assert_eq!(e.leads(), &[lead(i, 0)]);
        assert!(
            s.recovery
                .record_text(record)
                .unwrap()
                .contains("CLAIM")
                .then_some(())
                .is_some()
                || i == 1
        );
        s.character = Default::default();
        s.character.enter_cockpit();
        s.ship.set_cockpit_control(true);
        s.scanner.chart_mode = true;
        s.scanner.selected = s.chart.entries().position(|e| e.id == site_id(i)).unwrap();
        s.chart_route_selected();
        assert_eq!(s.chart.routes().len(), i + 1);
        assert!(s.chart_hint().contains(site_name(&site_id(i)).unwrap()));
        assert!(s.scanner.chart_note.unwrap().contains("UNVERIFIED ROUTE"));
    }
    s.publish_narrative_leads();
    assert_eq!(s.chart.entries().count(), 2);
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
    assert!(
        s.chart
            .entries()
            .all(|e| e.id.as_str().starts_with("salimon-site:"))
    );
    assert!(s.chart.entries().all(|e| e.observed().is_none()));
}
#[test]
fn physical_aim_range_pause_revisit_and_saved_inspection_are_independent_of_recovery() {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    assert!(!s.inspect_narrative_site());
    for (i, record) in RECORDS.iter().enumerate() {
        at_site(&mut s, i);
        assert_eq!(s.narrative_target(), Some(i));
        s.command(super::super::Command::TogglePause);
        assert!(!s.inspect_narrative_site());
        s.command(super::super::Command::TogglePause);
        assert!(s.inspect_narrative_site());
        assert!(s.narrative_text().unwrap().contains("UNVERIFIED CLAIM"));
        assert!(s.chart.entries().next().is_none());
        assert!(s.inspect_narrative_site());
        assert!(s.narrative_text().is_none());
        let bytes = s.save_bytes(100).unwrap();
        s.restore_bytes(&bytes).unwrap();
        assert!(
            s.narrative_archive_clue(record)
                .contains("FIELD INSPECTION")
        );
        assert!(s.narrative.reader.is_none());
        at_site(&mut s, i);
        assert!(s.inspect_narrative_site());
        s.narrative.reader = None;
    }
}
#[test]
fn incomplete_or_mutated_packet_coordinates_and_future_inspection_reject_atomically() {
    let mut s = recovered();
    let bytes = s.save_bytes(100).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    for (path, replacement) in [
        (
            "/state/chart/entries/0/leads/0/position/meters/0",
            serde_json::json!(123.),
        ),
        ("/state/recovery/spent_joules/0", serde_json::json!(0.)),
        (
            "/state/narrative/inspected_utc_millis/0",
            serde_json::json!(1),
        ),
    ] {
        let mut bad = value.clone();
        *bad.pointer_mut(path).unwrap() = replacement;
        assert!(
            s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{path}"
        );
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
    }
    let mut legacy = value.clone();
    legacy["schema_version"] = 30.into();
    legacy["state"].as_object_mut().unwrap().remove("narrative");
    s.restore_bytes(&serde_json::to_vec(&legacy).unwrap())
        .unwrap();
    let mut missing = value;
    missing["state"]
        .as_object_mut()
        .unwrap()
        .remove("narrative");
    assert!(
        s.restore_bytes(&serde_json::to_vec(&missing).unwrap())
            .is_err()
    );
}

#[test]
fn first_hour_can_ignore_every_clue_without_objectives_or_unearned_destinations() {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    let initial = s.tutorial_text().unwrap();
    assert!(initial.contains("Earth is home") && initial.contains("missing"));
    s.command(super::super::Command::SkipTutorial);
    assert!(s.tutorial_text().is_none());
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    // Declared one-second production session ticks, not a wall-clock claim or
    // hardware benchmark. No save/knowledge/resource state is assigned.
    for second in 0..3600 {
        if second == 1200 || second == 2400 {
            use super::super::consumable_wheel::WheelAction as W;
            for action in [
                W::Hold(true),
                W::Primary(false),
                W::Hover(Some(if second == 1200 {
                    salimon_fabrication::ConsumableSlot::FoodOne
                } else {
                    salimon_fabrication::ConsumableSlot::FoodTwo
                })),
                W::Primary(true),
                W::Primary(false),
                W::Hold(false),
            ] {
                s.command(super::super::Command::Wheel(action));
            }
        }
        s.advance_game(Duration::from_secs(1));
        if second == 11 {
            s.collect_fabricator_output();
        }
    }
    assert_eq!(
        s.elapsed,
        Duration::from_secs(3600),
        "{:?}, {:?}",
        s.survival.snapshot(),
        s.consumables
    );
    assert!(!s.survival.snapshot().terminal);
    assert!(s.recovery.module.is_none());
    assert!(s.chart.routes().is_empty());
    assert!(s.fabricator.snapshot(true).jobs.is_empty());
    assert!(
        s.chart
            .entries()
            .all(|e| !e.id.as_str().starts_with("salimon-site:"))
    );
    let bytes = s.save_bytes(3700).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.save_bytes(3700).unwrap(), bytes);
    assert!(restored.tutorial_text().is_none());
}

#[test]
fn recovered_near_lead_can_be_walked_to_and_investigated_with_production_controls() {
    use super::super::{Command, ControlKey};
    let mut s = recovered();
    let target = site_position(0);
    fn aim(s: &mut GameplaySession, target: [f64; 3]) {
        let c = s.snapshot().character;
        let frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
        let old = sub(c.look_target_meters, c.eye_position_meters);
        let new = sub(target, c.eye_position_meters);
        let yaw = |v| dot(v, frame.axes[2]).atan2(dot(v, frame.axes[0]));
        let pitch = |v| {
            (dot(v, c.up.map(f64::from)) / length(v))
                .clamp(-1., 1.)
                .asin()
        };
        let delta = (yaw(new) - yaw(old) + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU)
            - std::f64::consts::PI;
        s.command(Command::Look(
            delta / 0.0022,
            -(pitch(new) - pitch(old)) / 0.0022,
        ));
    }
    aim(&mut s, target);
    s.command(Command::Control(ControlKey::Forward, true));
    for _ in 0..4000 {
        if length(sub(target, s.snapshot().character.eye_position_meters)) < 2.5 {
            break;
        }
        s.advance_automation(Duration::from_millis(16));
    }
    s.command(Command::Control(ControlKey::Forward, false));
    aim(&mut s, target);
    assert!(length(sub(target, s.snapshot().character.eye_position_meters)) < 2.5);
    s.command(Command::Interact);
    assert!(
        s.narrative_text()
            .unwrap()
            .contains("may emerge near Absenat")
    );
    assert!(s.chart.get(&site_id(0)).unwrap().observed().is_none());
}
