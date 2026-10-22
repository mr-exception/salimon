//! Portable scenario extensions. Checkpoints use real disk storage and the normal
//! launch transaction; fixture placement never claims native input/route coverage.
use super::*;
use crate::{automation, checkpoints::Checkpoints, e2e};
use salimon_fabrication::{ConsumableSlot, FabricationCommand, RecipeId};
use salimon_persistence::{
    Validate,
    storage::{NativeStore, SaveManager},
};
use salimon_survival::{Points, effects::*};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

mod rates;

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "salimon-375-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn initialize(scenario: e2e::Scenario) -> GameplaySession {
    let mut s = GameplaySession::new_run();
    e2e::initialize(
        &mut s,
        e2e::Config {
            scenario,
            seed: 375,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )
    .unwrap();
    if scenario == e2e::Scenario::EvaApproach {
        // The existing scenario starts seated for its exit route. This isolated
        // checkpoint fixture starts outside, with the real serialized EVA state.
        let ship = s.ship.snapshot();
        let frame = frames::character_ship_frame(ship.pose);
        let mut character = serde_json::to_value(s.character.save_state()).unwrap();
        character["position"] = json!({"Space":{"world":frame.local_to_world([50., 10., 50.])}});
        character["eva_inherited_velocity"] = json!(ship.velocity_meters_per_second);
        s.character = salimon_character::CharacterController::from_save(
            serde_json::from_value(character).unwrap(),
            None,
        )
        .unwrap();
        s.ship.set_cockpit_control(false);
        assert_eq!(s.character.location(), CharacterLocation::Space);
    }
    s
}
fn value(s: &GameplaySession) -> Value {
    serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap()
}
fn validate(bytes: &[u8]) -> Result<(), salimon_persistence::SaveError> {
    GameplaySession::default().restore_bytes(bytes)
}
fn checkpoint(dir: &Directory, s: &GameplaySession, utc: u64) {
    let mut manager = SaveManager::new(NativeStore::open(dir.0.clone()).unwrap());
    manager.recover(validate).unwrap();
    manager
        .checkpoint(&s.save_bytes(utc).unwrap(), validate)
        .unwrap();
}
fn reload(dir: &Directory, utc: u64) -> GameplaySession {
    let mut s = GameplaySession {
        automation_step: Some(Duration::from_millis(16)),
        ..Default::default()
    };
    let host = Checkpoints::open_at(dir.0.clone(), &mut s, utc).unwrap();
    drop(host); // Release worker and exclusive filesystem lease before reopening.
    // Normal launch deliberately excludes automation host policy from saves.
    s.automation_step = Some(Duration::from_millis(16));
    s
}
fn protocol(s: &mut GameplaySession, mut request: Value) -> Value {
    request["protocol"] = json!(1);
    request["id"] = json!(375);
    automation::execute_session(s, &request.to_string())
}
fn meters(s: &mut GameplaySession, hunger: u32, fatigue: u32) {
    let mut v = serde_json::to_value(&s.survival).unwrap();
    v["hunger"] = serde_json::to_value(Points::whole(hunger).unwrap()).unwrap();
    v["fatigue"] = serde_json::to_value(Points::whole(fatigue).unwrap()).unwrap();
    s.survival = serde_json::from_value(v).unwrap();
    s.survival.validate().unwrap();
}
fn enqueue(s: &mut GameplaySession, recipe: &str, quantity: u32) {
    s.execute_fabricator(FabricationCommand::Enqueue {
        recipe: RecipeId::new(recipe).unwrap(),
        quantity,
    })
    .unwrap();
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    version: u32,
    steps: Vec<Step>,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Step {
    Protocol { request: Value, ok: bool },
    Advance { milliseconds: u64 },
    Checkpoint { utc: u64 },
    Reload { utc: u64 },
    Expect { pointer: String, equals: Value },
    Enqueue { recipe: String, quantity: u32 },
    Collect,
    Energy { joules: u64 },
    Damage { points: u32 },
    CompareContinuation { milliseconds: u64 },
    Freeze { milliseconds: u64 },
}

#[test]
fn persistence_scenario_dsl_crafting_blackout_pause_q_and_reload() {
    let suite: Suite = serde_json::from_str(include_str!(
        "../../../../scenarios/portable/persistence-survival.json"
    ))
    .unwrap();
    assert_eq!(suite.version, 1);
    let dir = Directory::new();
    let mut s = initialize(e2e::Scenario::LandedEarth);
    for (index, step) in suite.steps.into_iter().enumerate() {
        eprintln!("persistence-survival step {index}: {step:?}");
        match step {
            Step::Protocol { request, ok } => assert_eq!(protocol(&mut s, request)["ok"], ok),
            Step::Advance { milliseconds } => {
                s.advance_automation(Duration::from_millis(milliseconds))
            }
            Step::Checkpoint { utc } => checkpoint(&dir, &s, utc),
            Step::Reload { utc } => s = reload(&dir, utc),
            Step::Expect { pointer, equals } => assert_eq!(
                value(&s).pointer(&pointer).expect("existing save pointer"),
                &equals,
                "{pointer}"
            ),
            Step::Enqueue { recipe, quantity } => enqueue(&mut s, &recipe, quantity),
            Step::Collect => s.collect_fabricator_output(),
            Step::Energy { joules } => s.ship.set_stored_core_energy_joules(joules),
            Step::Damage { points } => s
                .survival
                .command(salimon_survival::Command::Damage(
                    Points::whole(points).unwrap(),
                ))
                .unwrap(),
            Step::CompareContinuation { milliseconds } => {
                let mut restored = GameplaySession::default();
                restored.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
                for owner in [&mut s, &mut restored] {
                    owner.advance_automation(Duration::from_millis(milliseconds));
                }
                assert_eq!(value(&s), value(&restored));
            }
            Step::Freeze { milliseconds } => {
                assert!(s.is_paused());
                let before = value(&s);
                s.advance_automation(Duration::from_millis(milliseconds));
                assert_eq!(value(&s), before);
            }
        }
    }
}

fn add_effects(s: &mut GameplaySession) {
    let mut rules = Rules::default();
    rules.effects.push(EffectRule {
        id: EffectId(375),
        behavior: Behavior::Damage {
            source: Source::Burning,
            amount: Points::whole(1).unwrap(),
            wound: false,
        },
        lifetime: Lifetime::Timed(Duration::from_secs(20)),
        interval: Some(Duration::from_secs(3)),
        max_stacks: 1,
        removal: Removal::Dispellable,
    });
    s.survival = salimon_survival::Survival::with_rules(Default::default(), rules).unwrap();
    meters(s, 50, 20);
    s.survival
        .command(salimon_survival::Command::Damage(
            Points::whole(40).unwrap(),
        ))
        .unwrap();
    s.survival
        .apply_consumable(s.consumable_effects.get("medkit").unwrap())
        .unwrap();
    s.survival.apply_effect(EffectId(375)).unwrap();
}

#[test]
fn complete_disk_roundtrip_matrix_and_deterministic_continuation() {
    for scenario in [
        e2e::Scenario::LandedEarth,
        e2e::Scenario::CoastingSpace,
        e2e::Scenario::EvaApproach,
        e2e::Scenario::CargoFlight,
        e2e::Scenario::CargoBlackout,
        e2e::Scenario::CoreRestart,
        e2e::Scenario::TerrainMoon,
    ] {
        let dir = Directory::new();
        let mut s = initialize(scenario);
        add_effects(&mut s);
        enqueue(&mut s, "food-ration", 1);
        s.advance_automation(Duration::from_millis(1375));
        // Include the unfinished fixed tick, partial effects, FIFO and physical state.
        checkpoint(&dir, &s, 100);
        let mut restored = reload(&dir, 100);
        let mut expected = value(&s);
        expected["state"]["survival"]["vitals"]["offline_watermark"] = json!(100);
        assert_eq!(value(&restored), expected, "{scenario:?}");
        for milliseconds in [9, 100, 1003, 3200] {
            for owner in [&mut s, &mut restored] {
                owner.advance_automation(Duration::from_millis(milliseconds));
            }
        }
        expected = value(&s);
        expected["state"]["survival"]["vitals"]["offline_watermark"] = json!(100);
        assert_eq!(value(&restored), expected, "{scenario:?} continuation");
    }
}

#[test]
fn disk_resume_freezes_everything_except_fatigue_and_replay_is_idempotent() {
    let dir = Directory::new();
    let mut s = initialize(e2e::Scenario::CargoFlight);
    add_effects(&mut s);
    enqueue(&mut s, "food-ration", 1);
    s.advance_automation(Duration::from_millis(1375));
    checkpoint(&dir, &s, 100);
    let mut expected = value(&s);
    let mut resumed = reload(&dir, 21700); // Exactly the approved six-hour cap.
    assert_eq!(resumed.survival.snapshot().fatigue, Points::FULL);
    expected["state"]["survival"]["vitals"]["fatigue"] =
        serde_json::to_value(Points::FULL).unwrap();
    expected["state"]["survival"]["vitals"]["offline_watermark"] = json!(21700);
    assert_eq!(value(&resumed), expected);
    for utc in [21700, 99, 100, 21699, 21700] {
        resumed = reload(&dir, utc);
        assert_eq!(value(&resumed), expected, "replay/rollback at {utc}");
    }
}

#[test]
#[ignore = "extended 500-fragment whole-session continuation; nightly portable verification"]
fn persistence_extended_500_fragments_depletion_streaming_and_identity_high_water() {
    use salimon_world::{
        CelestialBodyId, WorldPosition,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId},
    };
    let dir = Directory::new();
    let mut s = initialize(e2e::Scenario::LandedEarth);
    crate::benchmark_fixture::initialize(
        &mut s,
        500,
        crate::benchmark_fixture::Layout::Dense,
        375,
        false,
    )
    .unwrap();
    let make = || {
        ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 375_000,
            },
            RawMaterial::new(ResourceId::WaterIce, 1.0).unwrap(),
            WorldPosition::new(0., 0., 0.),
            1.0,
        )
        .unwrap()
    };
    let mut deposit = make();
    s.mining
        .session
        .extract(&mut deposit, Duration::from_secs(1));
    let id = s.mining.session.fragments().last().unwrap().id();
    assert_eq!(deposit.remaining_mass_kg(), 0.0);
    assert!(s.mining.session.remove_fragment(id));
    s.sync_carried();
    add_effects(&mut s);
    enqueue(&mut s, "food-ration", 1);
    checkpoint(&dir, &s, 100);
    let mut restored = reload(&dir, 100);
    assert_eq!(restored.mining.session.fragments().len(), 500);
    assert_eq!(restored.fabrication_snapshot(), s.fabrication_snapshot());
    assert_eq!(restored.consumables, s.consumables);
    assert_eq!(
        restored.ship.snapshot().total_mass_kg,
        s.ship.snapshot().total_mass_kg
    );
    let mut streamed = make();
    restored
        .mining
        .session
        .extract(&mut streamed, Duration::ZERO);
    assert_eq!(streamed.remaining_mass_kg(), 0.0);
    let count = restored.mining.session.fragments().len();
    restored
        .mining
        .session
        .extract(&mut streamed, Duration::from_secs(1));
    assert_eq!(restored.mining.session.fragments().len(), count);
    assert!(
        restored
            .mining
            .session
            .fragments()
            .iter()
            .all(|f| f.id() != id)
    );
    for owner in [&mut s, &mut restored] {
        let mut fresh = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 375_001,
            },
            RawMaterial::new(ResourceId::WaterIce, 1.0).unwrap(),
            WorldPosition::new(0., 0., 0.),
            1.0,
        )
        .unwrap();
        owner
            .mining
            .session
            .extract(&mut fresh, Duration::from_secs(1));
        let next_id = owner.mining.session.fragments().last().unwrap().id();
        assert!(next_id.0 > id.0, "consumed identity must not be recycled");
        assert!(owner.mining.session.remove_fragment(next_id));
    }
    for owner in [&mut s, &mut restored] {
        owner.advance_automation(Duration::from_millis(32));
    }
    let mut expected = value(&s);
    expected["state"]["survival"]["vitals"]["offline_watermark"] = json!(100);
    assert_eq!(value(&restored), expected);
}

#[test]
fn malformed_whole_session_restores_are_atomic() {
    let mut s = initialize(e2e::Scenario::CargoFlight);
    add_effects(&mut s);
    enqueue(&mut s, "food-ration", 1);
    let before = value(&s);
    for (pointer, replacement) in [
        (
            "/schema_version",
            json!(salimon_persistence::SCHEMA_VERSION + 1),
        ),
        ("/state/physical/ship/door_open_fraction", json!(2)),
        ("/state/survival/vitals/health", json!(u64::MAX)),
        ("/state/tutorial/completed", json!(255)),
        ("/state/physical/anchors", json!([[9999999, [0, 0, 0]]])),
        (
            "/state/physical/motion",
            json!([[9999999, {"velocity":[0,0,0],"angular_velocity":[0,0,0],"ship_orientation":[0,0,0,0]}]]),
        ),
    ] {
        let mut bad = before.clone();
        *bad.pointer_mut(pointer).expect("existing mutation target") = replacement;
        assert!(
            s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "{pointer}"
        );
        assert_eq!(
            value(&s),
            before,
            "failed restore published state at {pointer}"
        );
    }
}

#[test]
fn integrity_valid_but_domain_invalid_generation_falls_back_and_future_schema_stops_launch() {
    let s = initialize(e2e::Scenario::LandedEarth);
    let good = s.save_bytes(100).unwrap();
    for (pointer, replacement, future) in [
        ("/state/survival/vitals/health", json!(u64::MAX), false),
        (
            "/state/physical/anchors",
            json!([[9999999, [0, 0, 0]]]),
            false,
        ),
        (
            "/schema_version",
            json!(salimon_persistence::SCHEMA_VERSION + 1),
            true,
        ),
    ] {
        let dir = Directory::new();
        let mut manager = SaveManager::new(NativeStore::open(dir.0.clone()).unwrap());
        manager.checkpoint(&good, validate).unwrap();
        let mut bad: Value = serde_json::from_slice(&good).unwrap();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        // Bypass domain validation deliberately: another game version or a
        // logically invalid payload can still have a completely valid storage CRC.
        manager
            .checkpoint(&serde_json::to_vec(&bad).unwrap(), |_| Ok(()))
            .unwrap();
        drop(manager);
        let mut visible = GameplaySession::new_run();
        let before = value(&visible);
        let opened = Checkpoints::open_at(dir.0.clone(), &mut visible, 100);
        if future {
            assert!(matches!(
                opened,
                Err(salimon_persistence::SaveError::UnsupportedVersion(_))
            ));
            assert_eq!(value(&visible), before);
        } else {
            assert!(opened.unwrap().warning.is_some());
            let mut expected = value(&s);
            expected["state"]["survival"]["vitals"]["offline_watermark"] = json!(100);
            assert_eq!(value(&visible), expected);
        }
    }
}

/// Emit validated disk checkpoints for reference renderer/lifecycle review.
/// Explicit fixtures are not evidence of physical travel or naturally elapsed death.
#[test]
#[ignore = "writes opt-in native review fixtures; requires SALIMON_NATIVE_REVIEW_DIR"]
fn native_reference_checkpoints_376() {
    let root = PathBuf::from(
        std::env::var_os("SALIMON_NATIVE_REVIEW_DIR").expect("isolated output directory required"),
    );
    std::fs::create_dir(&root).expect("output must be new; never overwrite a live save");
    let utc = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let cases = [
        ("landed", e2e::Scenario::LandedEarth),
        ("cockpit", e2e::Scenario::CockpitEarth),
        ("moving", e2e::Scenario::CoastingSpace),
        ("eva", e2e::Scenario::EvaApproach),
        ("terminal", e2e::Scenario::LandedEarth),
    ];
    let mut manifest = Vec::new();
    for (name, scenario) in cases {
        let mut s = initialize(scenario);
        meters(&mut s, 30, 20);
        s.survival
            .damage(Damage {
                source: Source::Environment,
                amount: Points::whole(40).unwrap(),
                wound: false,
            })
            .unwrap();
        let slot = ConsumableSlot::ALL[3];
        let item = s.consumables.item(slot).unwrap();
        let selection = consumables::Selection {
            slot,
            job: item.job,
            unit: item.unit,
        };
        s.consume_selection(selection).unwrap();
        enqueue(&mut s, "food-ration", 1);
        s.advance_game(Duration::from_secs(1));
        if name == "terminal" {
            s.survival
                .damage(Damage {
                    source: Source::Environment,
                    amount: Points::FULL,
                    wound: false,
                })
                .unwrap();
            s.advance_game(Duration::from_millis(16));
            assert!(s.survival.snapshot().terminal);
        }
        let bytes = s.save_bytes(utc).unwrap();
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&bytes).unwrap();
        assert_eq!(restored.save_bytes(utc).unwrap(), bytes);
        let directory = root.join(name);
        let mut manager = SaveManager::new(NativeStore::open(directory).unwrap());
        manager.checkpoint(&bytes, validate).unwrap();
        manager.checkpoint(&bytes, validate).unwrap();
        manifest.push(json!({"case":name,"scenario":scenario.name(),"fixture":true,"saved_at_utc":utc,"active_effects":name!="terminal","scope":"validated fixture checkpoint; not native input, travel, natural death or performance acceptance"}));
    }
    std::fs::write(
        root.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
