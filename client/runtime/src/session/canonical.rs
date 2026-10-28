//! Current canonical surface composition and exact saved definition binding.
use super::GameplaySession;
pub(crate) fn definition_binding() -> salimon_persistence::universe::DefinitionBinding {
    let start = &salimon_world::canonical::START;
    salimon_persistence::universe::DefinitionBinding {
        body: start.manifest().planet.id.clone(),
        version: start.definition().version,
        sha256: start.definition().sha256.clone(),
    }
}
pub(crate) fn initialize(session: &mut GameplaySession) -> Result<(), String> {
    session.enable_terrain(salimon_world::canonical::BODIES[0], [0., 1., 0.])
}

/// Evidence fixture: standing near a real published ore deposit, then ordinary
/// equipment/input/mining/carry/physics code owns every subsequent action.
pub(crate) fn mining_initial_conditions(session: &mut GameplaySession) -> Result<(), String> {
    initialize(session)?;
    let start = &salimon_world::canonical::START;
    let site = start
        .manifest()
        .resources
        .outcrops
        .iter()
        .find(|s| s.material == salimon_universe::starting_system::Material::IronOre)
        .ok_or("missing canonical ore")?;
    let point = start
        .surface_point(site.tangent_xz_meters)
        .map_err(|e| e.to_string())?;
    let frame = session.terrain_frame(session.ship.snapshot());
    let deposits = session
        .mining
        .nearby(point, session.seed)
        .map_err(|e| format!("canonical deposits: {e:?}"))?;
    let entry = deposits
        .iter()
        .find(|e| e.deposit.id().local == salimon_world::canonical::outcrop_local_id(site.material))
        .ok_or("canonical ore not in range")?;
    let mut hint = point;
    hint[0] -= entry.bounds_radius_meters + 2.0;
    session.character =
        salimon_character::CharacterController::on_surface(frame, hint).map_err(str::to_owned)?;
    let ship_frame = super::frames::character_ship_frame(session.ship.snapshot().pose);
    let snapshot = session.character.snapshot(ship_frame, frame);
    let v = salimon_math::sub(
        entry.deposit.position().meters(),
        snapshot.eye_position_meters,
    );
    let up = snapshot.up.map(f64::from);
    let d = salimon_math::scale(v, 1.0 / salimon_math::length(v));
    let yaw =
        salimon_math::dot(d, ship_frame.axes[2]).atan2(salimon_math::dot(d, ship_frame.axes[0]));
    let pitch = salimon_math::dot(d, up).clamp(-1., 1.).asin();
    session
        .character
        .apply_mouse_delta(yaw / 0.0022, -pitch / 0.0022);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn canonical_gameplay_all_surface_consumers_share_published_ground() {
        let mut app = GameplaySession::new_run();
        initialize(&mut app).unwrap();
        let ship = app.ship.snapshot();
        assert_eq!(
            ship.flight_state,
            salimon_ship::FlightState::Landed {
                body: salimon_world::CelestialBodyId::Vesper
            }
        );
        assert_eq!(ship.nearby_body.unwrap().name, "Vesper");
        let frame = app.terrain_frame(ship);
        let point = salimon_world::canonical::START
            .surface_point([14., 0.])
            .unwrap();
        let ground = frame.sample(point);
        let physics = crate::terrain::physics_ground(
            salimon_physics::SphereSurface {
                center: frame.body_center_meters,
                radius: frame.radius_meters,
            },
            point,
        );
        assert_eq!(physics.point, ground.position_meters);
        assert_eq!(physics.normal, ground.normal);
        assert_eq!(
            crate::terrain::physics_ground_point(
                salimon_physics::SphereSurface {
                    center: frame.body_center_meters,
                    radius: frame.radius_meters
                },
                point
            ),
            ground.position_meters
        );
        assert_eq!(app.mining.nearby(point, 0).unwrap().len(), 5);
        assert_eq!(
            app.mining.nearby(point, 9999).unwrap(),
            app.mining.nearby(point, 0).unwrap()
        );
        for b in salimon_world::CELESTIAL_BODIES.iter().filter(|b| {
            matches!(
                b.id,
                salimon_world::CelestialBodyId::Earth
                    | salimon_world::CelestialBodyId::Moon
                    | salimon_world::CelestialBodyId::Mars
            )
        }) {
            let mut fixture = GameplaySession::default();
            fixture.enable_terrain(*b, [0., 1., 0.]).unwrap();
            assert_eq!(
                fixture.terrain_frame(fixture.ship.snapshot()).radius_meters,
                b.radius_meters
            );
        }
    }
    #[test]
    fn canonical_mining_funds_real_fragments_and_restore_never_replenishes_deposits() {
        let mut app = GameplaySession::new_run();
        mining_initial_conditions(&mut app).unwrap();
        let snapshot = app.snapshot();
        app.equipment
            .select(crate::equipment::ToolbarSlot::One, false);
        app.mining.set_mouse_held(true);
        let frame = super::super::frames::character_ship_frame(snapshot.ship.pose);
        assert!(
            app.mining
                .target(&app.equipment, snapshot.character, frame, false, 0)
                .is_some()
        );
        for _ in 0..125 {
            app.mining.advance(
                &app.equipment,
                Duration::from_millis(16),
                snapshot.character,
                frame,
                false,
                0,
            );
        }
        app.mining.set_mouse_held(false);
        app.mining.advance(
            &app.equipment,
            Duration::ZERO,
            snapshot.character,
            frame,
            false,
            0,
        );
        assert!(app.mining.session.extracted_mass_kg() > 3.9);
        assert!(!app.mining.session.fragments().is_empty());
        let bytes = app.save_bytes(100).unwrap();
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&bytes).unwrap();
        assert_eq!(restored.save_bytes(100).unwrap(), bytes);
        let point = snapshot.character.eye_position_meters;
        let entries = restored.mining.nearby(point, 0).unwrap();
        let ore = entries.iter().find(|e| e.deposit.id().local == 1).unwrap();
        assert!(ore.deposit.remaining_mass_kg() < 497.);
        let _ = restored
            .mining
            .nearby([10_000., 10_000., 10_000.], 999)
            .unwrap();
        let again = restored.mining.nearby(point, 999).unwrap();
        assert_eq!(
            again
                .iter()
                .find(|e| e.deposit.id().local == 1)
                .unwrap()
                .deposit
                .remaining_mass_kg(),
            ore.deposit.remaining_mass_kg()
        );
    }
    #[test]
    fn canonical_restore_refuses_unknown_world_digest_and_inflated_or_unknown_depletion_atomically()
    {
        let mut app = GameplaySession::new_run();
        initialize(&mut app).unwrap();
        let bytes = app.save_bytes(100).unwrap();
        for kind in 0..4 {
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            match kind {
                0 => {
                    value["state"]["physical"]["canonical_definition"]["sha256"] =
                        "0".repeat(64).into()
                }
                1 => {
                    value["state"]["physical"]["canonical_definition"]["body"] =
                        "fixture:wrong".into()
                }
                2 => {
                    value["state"]["physical"]["world"]["remaining"] =
                        serde_json::json!([[{"body":"Vesper","local":1},501.0]])
                }
                _ => {
                    value["state"]["physical"]["world"]["remaining"] =
                        serde_json::json!([[{"body":"Vesper","local":999},0.0]])
                }
            }
            assert!(
                app.restore_bytes(&serde_json::to_vec(&value).unwrap())
                    .is_err()
            );
            assert_eq!(app.save_bytes(100).unwrap(), bytes);
        }
    }
    #[test]
    fn canonical_mined_material_enters_existing_cargo_and_fabricator_exactly_once() {
        let mut app = GameplaySession::new_run();
        mining_initial_conditions(&mut app).unwrap();
        let snapshot = app.snapshot();
        let frame = super::super::frames::character_ship_frame(snapshot.ship.pose);
        app.equipment
            .select(crate::equipment::ToolbarSlot::One, false);
        app.mining.set_mouse_held(true);
        for _ in 0..125 {
            app.mining.advance(
                &app.equipment,
                Duration::from_millis(16),
                snapshot.character,
                frame,
                false,
                0,
            );
        }
        app.mining.set_mouse_held(false);
        app.mining.advance(
            &app.equipment,
            Duration::ZERO,
            snapshot.character,
            frame,
            false,
            0,
        );
        let fragment = app.mining.session.fragments()[0];
        // Declared intake fixture puts the existing character at the machine;
        // production pickup/source ownership and intake perform the transaction.
        app.character = Default::default();
        assert!(app.mining.session.pick_up(fragment.id()));
        app.sync_carried();
        let before = app.fabricator.mass_contribution().stored_milligrams;
        app.insert_fabricator_fragment();
        assert!(app.mining.session.fragment(fragment.id()).is_none());
        assert_eq!(
            app.fabricator.mass_contribution().stored_milligrams,
            before + (fragment.material().mass_kg() * 1e6).round() as u64
        );
        app.insert_fabricator_fragment();
        assert_eq!(
            app.feedback,
            Some("Carry a fragment to feed the Fabricator")
        );
        let saved = app.save_bytes(100).unwrap();
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&saved).unwrap();
        assert_eq!(restored.save_bytes(100).unwrap(), saved);
    }
    #[test]
    fn canonical_takeoff_and_landing_are_funded_partition_independent_and_restore_body_context() {
        let mut app = GameplaySession::new_run();
        initialize(&mut app).unwrap();
        app.ship.set_cockpit_control(true);
        let before = app.ship.snapshot().energy_core.stored_joules;
        app.ship.trigger_landing_action();
        let mut split = app.ship.clone();
        app.ship.advance(Duration::from_secs(6));
        for _ in 0..375 {
            split.advance(Duration::from_millis(16));
        }
        assert_eq!(app.ship.snapshot(), split.snapshot());
        assert_eq!(
            app.ship.snapshot().flight_state,
            salimon_ship::FlightState::Flying
        );
        assert!(app.ship.snapshot().energy_core.stored_joules < before);
        app.ship.trigger_landing_action();
        app.ship.advance(Duration::from_secs(8));
        assert_eq!(
            app.ship.snapshot().flight_state,
            salimon_ship::FlightState::Landed {
                body: salimon_world::CelestialBodyId::Vesper
            }
        );
        let mut restored = GameplaySession::default();
        restored
            .restore_bytes(&app.save_bytes(100).unwrap())
            .unwrap();
        assert_eq!(
            restored.ship.snapshot().terrain,
            salimon_world::terrain::TerrainMode::Canonical
        );
        restored.ship.set_cockpit_control(true);
        restored.ship.set_stored_core_energy_joules(0);
        restored.ship.trigger_landing_action();
        assert_eq!(
            restored.ship.snapshot().flight_state,
            salimon_ship::FlightState::Landed {
                body: salimon_world::CelestialBodyId::Vesper
            }
        );
    }
}
