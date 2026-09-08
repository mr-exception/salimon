//! Portable release → pickup/carry/drop → mechanical Core composition regression.
use crate::{
    automation,
    e2e::{self, Scenario},
    session::GameplaySession,
    session::frames::character_ship_frame,
};
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    spherical_mining::{MiningConfig, SPHERE_TICK},
};
use std::time::Duration;

fn command(app: &mut GameplaySession, value: serde_json::Value) {
    let mut value = value;
    value["protocol"] = 1.into();
    value["id"] = 1.into();
    let response = automation::execute_session(app, &value.to_string());
    assert_eq!(response["ok"], true, "{response}");
}
fn f(app: &mut GameplaySession) {
    command(
        app,
        serde_json::json!({"op":"key","key":"grab_drop","pressed":true}),
    );
    command(
        app,
        serde_json::json!({"op":"key","key":"grab_drop","pressed":false}),
    );
}

#[test]
fn cap_and_short_deposit_spheres_keep_identity_radius_and_yield_through_core() {
    for resource in [
        ResourceId::IronOre,
        ResourceId::SilicateRock,
        ResourceId::WaterIce,
    ] {
        for mass in [10.0, 0.25] {
            let mut app = GameplaySession::default();
            e2e::initialize(
                &mut app,
                e2e::Config {
                    scenario: Scenario::CoreRestart,
                    seed: 0,
                    step: SPHERE_TICK,
                    fragment_load: None,
                },
            )
            .unwrap();
            app.mining = Default::default();
            app.mining.ship_gravity_disabled = true;
            let frame = character_ship_frame(app.ship.snapshot().pose);
            let player = app
                .character
                .snapshot(frame, app.terrain_frame(app.ship.snapshot()));
            let forward = salimon_math::sub(player.look_target_meters, player.eye_position_meters);
            let forward = salimon_math::scale(forward, 1.0 / salimon_math::length(forward));
            let point = salimon_math::add(
                player.eye_position_meters,
                salimon_math::scale(forward, 0.85),
            );
            let pose = ResourceTransform::new(
                WorldPosition::new(point[0], point[1], point[2]),
                [0.0, 0.0, 0.0, 1.0],
            )
            .unwrap();
            let material = RawMaterial::new(resource, mass).unwrap();
            let mut deposit = ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 234,
                },
                material,
                pose.position(),
                mass,
            )
            .unwrap();
            let mut released = None;
            for _ in 0..400 {
                let id = app.mining.session.tick_spherical(
                    true,
                    Some(&mut deposit),
                    SPHERE_TICK,
                    MiningConfig::default(),
                    pose,
                );
                if let Some(id) = id {
                    released = Some(id);
                    break;
                }
                let attached = app.mining.session.attached_sphere_id().unwrap();
                assert!(!app.mining.session.pick_up(attached));
                assert!(crate::fragment_physics::snapshots(&app.mining, frame, point).is_empty());
            }
            let id = released.unwrap();
            assert!(deposit.remaining_mass_kg() < 1e-10);
            assert_eq!(app.mining.session.fragments().len(), 1);
            let original = app.mining.session.fragments()[0];
            assert_eq!(original.id(), id);
            crate::sphere_fixtures::assert_sphere(original, resource, mass);
            let radius = crate::fragment_physics::sphere_radius(original);
            let inspection = automation::inspect_session(&app);
            let serialized = &inspection["world"]["fragments"][0];
            assert_eq!(serialized["side_meters"].as_f64(), Some(radius * 2.0));
            assert!(
                (serialized["sphere_visual"]["radius_meters"]
                    .as_f64()
                    .unwrap()
                    - radius)
                    .abs()
                    < 1e-12
            );
            assert_eq!(
                serialized["ship_local_position_meters"]
                    .as_array()
                    .unwrap()
                    .len(),
                3
            );
            assert!(serialized.get("authored_mesh_scale").is_none());
            assert!((original.material().mass_kg() - mass).abs() < 1e-10);
            command(
                &mut app,
                serde_json::json!({"op":"aim_fragment","fragment_id":id.0}),
            );
            f(&mut app);
            assert_eq!(app.mining.session.carried_id(), Some(id));
            assert!(crate::fragment_physics::snapshots(&app.mining, frame, point).is_empty());
            f(&mut app);
            assert_eq!(app.mining.session.carried_id(), None);
            assert!(app.mining.ship_fragments.contains_key(&id));
            command(&mut app, serde_json::json!({"op":"restore_look"}));
            command(
                &mut app,
                serde_json::json!({"op":"aim_fragment","fragment_id":id.0}),
            );
            f(&mut app);
            assert_eq!(app.mining.session.carried_id(), Some(id));
            command(&mut app, serde_json::json!({"op":"restore_look"}));
            let carried = app.mining.session.fragments()[0];
            assert_eq!(carried.id(), original.id());
            assert_eq!(carried.source(), original.source());
            assert_eq!(carried.material(), original.material());
            assert_eq!(crate::fragment_physics::sphere_radius(carried), radius);
            let initial = app.ship.snapshot().energy_core.stored_joules;
            command(&mut app, serde_json::json!({"op":"interact"}));
            assert!(app.mining.session.fragments().is_empty());
            assert_eq!(app.mining.session.carried_id(), None);
            app.advance_game(Duration::from_secs(1));
            assert!(app.ship.snapshot().energy_core.stored_joules > initial);
            assert!(!app.mining.ship_fragments.contains_key(&id));
            let mut stale =
                ResourceDeposit::new(original.source(), material, pose.position(), mass).unwrap();
            app.mining.session.tick_spherical(
                false,
                Some(&mut stale),
                Duration::ZERO,
                MiningConfig::default(),
                pose,
            );
            assert!(stale.remaining_mass_kg() < 1e-10);
        }
    }
}

#[test]
fn analytic_radius_matches_authored_render_pickup_and_contact_contracts() {
    use salimon_world::{
        carrying::aimed_fragment, mining::MiningRay, resource_fragments::diameter_meters,
    };
    for resource in [
        ResourceId::IronOre,
        ResourceId::SilicateRock,
        ResourceId::WaterIce,
    ] {
        for mass in [0.001, 0.25, 10.0] {
            for anchor in [0.0, 1e12] {
                let center = WorldPosition::new(anchor, anchor, anchor + 2.0);
                let pose = ResourceTransform::new(center, [0.0, 0.0, 0.0, 1.0]).unwrap();
                let mut tool = crate::mining::ResourceGameplay::default();
                let id = crate::sphere_fixtures::sphere(&mut tool.session, resource, mass, pose);
                let piece = *tool.session.fragment(id).unwrap();
                let radius = crate::fragment_physics::sphere_radius(piece);
                let mesh = crate::resource_presentation::fragment_mesh(piece);
                let rendered_radius =
                    mesh.authored_mesh_scale * salimon_renderer::SPHERE_AUTHORED_RADIUS_METERS;
                assert!((rendered_radius - radius).abs() < 1e-12);
                assert_eq!(diameter_meters(piece), radius * 2.0);
                assert_ne!(mesh.authored_mesh_scale, diameter_meters(piece));
                let proxy = crate::resource_presentation::fragment_visual(piece);
                assert!((f64::from(proxy.half_extents_meters[0]) - radius).abs() < 1e-7);
                let frame = salimon_character::ShipFrame {
                    origin_meters: [anchor; 3],
                    axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                };
                let states = crate::fragment_physics::snapshots(&tool, frame, center.meters());
                assert_eq!(states[0].1.radius, radius);
                assert_eq!(states[0].1.ground_support_meters, radius);
                // Probe just inside/outside the visible radius, including near a
                // distant anchor. This detects accidentally picking with mesh scale.
                for (fraction, expected) in [(0.9, Some(id)), (1.1, None)] {
                    let eye = WorldPosition::new(anchor + radius * fraction, anchor, anchor);
                    let look = eye.translated([0.0, 0.0, 4.0]);
                    assert_eq!(
                        aimed_fragment(
                            MiningRay::new(eye, look).unwrap(),
                            &[piece],
                            None,
                            f64::INFINITY
                        ),
                        expected
                    );
                }
            }
        }
    }
}
