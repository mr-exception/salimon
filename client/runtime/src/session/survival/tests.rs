use super::*;
use crate::e2e::{self, Config, Scenario};
use crate::session::{Command, ControlKey};
use salimon_survival::{Points, Survival};
use serde_json::json;

fn session(scenario: Scenario) -> GameplaySession {
    let mut s = GameplaySession::default();
    e2e::initialize(
        &mut s,
        Config {
            scenario,
            seed: 369,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )
    .unwrap();
    s
}
fn vitals(s: &mut GameplaySession, hunger: u32, fatigue: u32) {
    // Real validated domain state, not a second permission flag in the host.
    let mut value = serde_json::to_value(Survival::default()).unwrap();
    value["hunger"] = serde_json::to_value(Points::whole(hunger).unwrap()).unwrap();
    value["fatigue"] = serde_json::to_value(Points::whole(fatigue).unwrap()).unwrap();
    s.survival = serde_json::from_value(value).unwrap();
    salimon_persistence::Validate::validate(&s.survival).unwrap();
}

#[test]
fn movement_is_authoritative_in_cabin_surface_and_eva_without_scaling_ship_drift() {
    for scenario in [
        Scenario::LandedEarth,
        Scenario::ResourceApproach,
        Scenario::OpenSpace,
    ] {
        let mut normal = session(scenario);
        let mut tired = session(scenario);
        for s in [&mut normal, &mut tired] {
            let mut state = serde_json::to_value(s.character.save_state()).unwrap();
            let pose = s.snapshot().ship.pose;
            let frame = crate::session::frames::character_ship_frame(pose);
            state["position"] = match scenario {
                Scenario::ResourceApproach => {
                    let body = salimon_world::CELESTIAL_BODIES
                        .iter()
                        .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
                        .unwrap();
                    let mut world = body.center.meters();
                    world[0] += 100.0;
                    world[1] += body.radius_meters + salimon_character::PLAYER_EYE_HEIGHT_METERS;
                    json!({"Surface":{"world":world}})
                }
                Scenario::OpenSpace => {
                    json!({"Space":{"world":frame.local_to_world([50.0,10.0,50.0])}})
                }
                _ => state["position"].clone(),
            };
            s.character = salimon_character::CharacterController::from_save(
                serde_json::from_value(state).unwrap(),
                None,
            )
            .unwrap();
        }
        vitals(&mut tired, 30, 30); // .9 * .85 = .765
        let start = normal.snapshot().character.eye_position_meters;
        for s in [&mut normal, &mut tired] {
            s.command(Command::Control(ControlKey::Forward, true));
            s.advance_automation(Duration::from_millis(160));
        }
        let distance = |s: &GameplaySession| {
            s.snapshot()
                .character
                .eye_position_meters
                .iter()
                .zip(start)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        assert!(distance(&normal) > 0.1, "{scenario:?}");
        let ratio = distance(&tired) / distance(&normal);
        assert!((ratio - 0.765).abs() < 0.02, "{scenario:?}: {ratio}");
        assert_eq!(normal.snapshot().ship, tired.snapshot().ship);
    }
}

#[test]
fn exhaustion_revokes_held_thrust_steering_and_mining_without_replaying_after_recovery() {
    let mut s = session(Scenario::FlightRegression);
    s.command(Command::Control(ControlKey::Left, true));
    s.command(Command::Throttle(salimon_ship::ThrottleDirection::Up, true));
    s.advance_automation(Duration::from_millis(160));
    vitals(&mut s, 100, 0);
    let before = s.snapshot().ship;
    s.command(Command::Mining(true));
    s.command(Command::Throttle(salimon_ship::ThrottleDirection::Up, true));
    s.command(Command::Control(ControlKey::Forward, true));
    let response = crate::automation::execute_session(
        &mut s,
        &json!({"protocol":1,"id":1,"op":"thruster","direction":1}).to_string(),
    );
    assert_eq!(response["result"]["applied"], false);
    s.advance_automation(Duration::from_millis(160));
    let after = s.snapshot().ship;
    assert_eq!(
        after.velocity_meters_per_second,
        before.velocity_meters_per_second
    );
    assert_eq!(after.pose.orientation, before.pose.orientation);
    assert_eq!(after.thruster_percentage, 0);
    assert!(!after.cockpit_control_active);
    assert!(!s.mining.input.held);
    assert_eq!(s.ship_control_input, ShipControlInput::default());
    vitals(&mut s, 100, 100);
    s.advance_automation(Duration::from_millis(160));
    assert_eq!(s.snapshot().ship.thruster_percentage, 0);
    assert_eq!(s.ship_control_input, ShipControlInput::default());
}

#[test]
fn exhausted_can_exit_cockpit_operate_door_and_use_emergency_actions_but_not_fabricate() {
    let mut s = session(Scenario::CockpitEarth);
    vitals(&mut s, 0, 0);
    s.command(Command::Landing);
    assert!(matches!(
        s.snapshot().ship.flight_state,
        salimon_ship::FlightState::Landed { .. }
    ));
    s.command(Command::Interact);
    assert_eq!(
        s.character.location(),
        salimon_character::CharacterLocation::InsideShip
    );
    for action in [Action::Consume, Action::Door, Action::Menu] {
        assert!(s.admit_survival_action(action));
    }
    s.crafting_ui.open = true;
    let machine = s.fabrication_snapshot();
    s.crafting_action(crate::session::crafting_ui::CraftingAction::Add);
    assert!(!s.crafting_ui.open);
    assert_eq!(s.fabrication_snapshot(), machine);
    assert!(!s.permitted_interaction(InteractionTarget::Cockpit));
    assert!(s.permitted_interaction(InteractionTarget::ExitDoor));
    assert!(!s.admit_survival_action(Action::Carry));
    assert!(s.admit_survival_action(Action::Walk));
}

#[test]
fn slowed_actions_use_active_time_without_blocking_emergency_controls_or_machine_jobs() {
    let mut s = session(Scenario::LandedEarth);
    vitals(&mut s, 10, 10);
    assert_eq!(
        s.mining_delta(Duration::from_millis(16)),
        Duration::from_millis(9)
    );
    assert!(s.admit_survival_action(Action::Fabricate));
    assert!(!s.admit_survival_action(Action::Carry));
    assert!(s.admit_survival_action(Action::Door));
    assert!(s.admit_survival_action(Action::Consume));
    let delay = s.restrictions().interaction_delay();
    s.advance_automation(delay + Duration::from_millis(16));
    assert!(s.admit_survival_action(Action::Carry));
    let inspect = crate::automation::inspect_session(&s);
    assert_eq!(inspect["survival_restrictions"]["movement_bps"], 4875);
    assert_eq!(inspect["survival_restrictions"]["interaction_bps"], 5625);
}

#[test]
fn committed_assist_is_not_cancelled_by_exhaustion() {
    let mut s = session(Scenario::CockpitEarth);
    s.command(Command::Landing);
    assert!(matches!(
        s.snapshot().ship.flight_state,
        salimon_ship::FlightState::AssistedTakeoff { .. }
    ));
    vitals(&mut s, 100, 0);
    s.advance_automation(Duration::from_secs(7));
    assert_eq!(
        s.snapshot().ship.flight_state,
        salimon_ship::FlightState::Flying
    );
    assert_eq!(s.snapshot().ship.thruster_percentage, 0);
}

#[test]
fn exhausted_door_uses_real_interact_route_and_attached_mining_is_released_without_mass_loss() {
    use salimon_world::{
        CelestialBodyId, WorldPosition,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    };
    let mut s = session(Scenario::LandedEarth);
    let mut state = serde_json::to_value(s.character.save_state()).unwrap();
    state["position"] = json!({"Inside":{"local":[-7.0,
        salimon_character::SHIP_FLOOR_HEIGHT_METERS + salimon_character::PLAYER_EYE_HEIGHT_METERS,0.0]}});
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(state).unwrap(),
        None,
    )
    .unwrap();
    vitals(&mut s, 0, 0);
    s.command(Command::Interact);
    s.advance_automation(Duration::from_millis(16));
    assert!(s.snapshot().ship.door_open_fraction > 0.0);

    let pose =
        ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0]).unwrap();
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: CelestialBodyId::Earth,
            local: 369,
        },
        RawMaterial::new(ResourceId::IronOre, 100.0).unwrap(),
        pose.position(),
        100.0,
    )
    .unwrap();
    s.mining.session.tick_spherical(
        true,
        Some(&mut deposit),
        Duration::from_millis(16),
        s.mining.config,
        pose,
    );
    assert!(s.mining.session.attached_sphere_id().is_some());
    let mass = s.mining.session.extracted_mass_kg();
    s.mining.input.held = true;
    s.enforce_survival_restrictions();
    assert!(s.mining.session.attached_sphere_id().is_none());
    assert_eq!(s.mining.session.extracted_mass_kg(), mass);
    assert!(!s.mining.input.held);
    vitals(&mut s, 100, 100);
    s.enforce_survival_restrictions();
    assert!(!s.mining.input.held);
}

#[test]
fn exhausted_player_cannot_enqueue_but_preexisting_jobs_keep_full_powered_time() {
    use salimon_fabrication::{FabricationCommand as C, RecipeId};
    use salimon_world::resources::{RawMaterial, ResourceId};
    let mut normal = session(Scenario::LandedEarth);
    let mut exhausted = session(Scenario::LandedEarth);
    for s in [&mut normal, &mut exhausted] {
        for id in [ResourceId::OrganicBiomass, ResourceId::WaterIce] {
            s.execute_fabricator(C::Insert(RawMaterial::new(id, 1.0).unwrap()))
                .unwrap();
        }
        s.execute_fabricator(C::Enqueue {
            recipe: RecipeId::new("food-ration").unwrap(),
            quantity: 1,
        })
        .unwrap();
    }
    vitals(&mut exhausted, 0, 0);
    for s in [&mut normal, &mut exhausted] {
        s.advance_automation(Duration::from_millis(160));
    }
    assert_eq!(
        normal.fabrication_snapshot(),
        exhausted.fabrication_snapshot()
    );
    assert_eq!(
        exhausted.fabrication_snapshot().jobs[0].progress,
        Duration::from_millis(160)
    );
    assert_eq!(
        normal.snapshot().ship.energy_core,
        exhausted.snapshot().ship.energy_core
    );
}
