//! Typed schema-1 state composition. Restore builds temporary owners, never ticks.
use super::{GameplaySession, ViewMode};
use crate::mining::ResourceGameplay;
use salimon_persistence::{SaveError, Validate, invalid};
use salimon_world::resources::FragmentId;
use std::collections::HashMap;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PhysicalState {
    pub(crate) seed: u64,
    view_mode: ViewMode,
    camera: salimon_world::CameraPrototype,
    character: salimon_character::CharacterSaveState,
    ship: salimon_ship::ShipSaveState,
    terrain: salimon_world::terrain::TerrainMode,
    ship_gravity_disabled: bool,
    mining_config: salimon_world::spherical_mining::MiningConfig,
    world: salimon_world::mining::MiningSaveState,
    #[serde(with = "salimon_persistence::pairs")]
    anchors: HashMap<FragmentId, salimon_character::ShipLocalPoint>,
    #[serde(with = "salimon_persistence::pairs")]
    motion: HashMap<FragmentId, crate::fragment_physics::FragmentMotion>,
    sleep: salimon_physics::SleepSaveState<FragmentId>,
}
pub(crate) struct RestoredPhysical {
    pub(crate) seed: u64,
    pub(crate) view_mode: ViewMode,
    pub(crate) camera: salimon_world::CameraPrototype,
    pub(crate) character: salimon_character::CharacterController,
    pub(crate) ship: salimon_ship::ShipController,
    pub(crate) mining: ResourceGameplay,
}
impl Validate for PhysicalState {
    fn validate(&self) -> Result<(), SaveError> {
        self.character.validate()?;
        self.ship.validate()?;
        if let Some(s) = self.character.nearby_surface()
            && !salimon_world::CELESTIAL_BODIES.iter().any(|b| {
                b.center.meters() == s.body_center_meters && b.radius_meters == s.radius_meters
            })
        {
            return Err(invalid("character surface is not in world catalog"));
        }
        self.world.validate()?;
        salimon_world::spherical_mining::MiningConfig::new(
            self.mining_config.cap_kg(),
            self.mining_config.rate_kg_per_second(),
        )
        .map_err(|_| invalid("mining config"))?;
        self.camera.validate_save()?;
        let ids: std::collections::HashSet<_> =
            self.world.fragments.iter().map(|f| f.id()).collect();
        for (id, anchor) in &self.anchors {
            if !ids.contains(id)
                || Some(*id) == self.world.carried
                || anchor.meters().iter().any(|v| !v.is_finite())
            {
                return Err(invalid("ship fragment anchor reference"));
            }
        }
        for (id, m) in &self.motion {
            if !ids.contains(id)
                || m.velocity
                    .iter()
                    .chain(m.angular_velocity.iter())
                    .any(|v| !v.is_finite())
            {
                return Err(invalid("fragment motion reference"));
            }
            if let Some(q) = m.ship_orientation {
                let n: f64 = q.iter().map(|v| v * v).sum();
                if !n.is_finite() || (n - 1.).abs() > 1e-9 {
                    return Err(invalid("fragment frame orientation"));
                }
            }
        }
        Ok(())
    }
}
impl PhysicalState {
    pub(crate) fn capture(s: &GameplaySession) -> Self {
        Self {
            seed: s.seed,
            view_mode: s.view_mode,
            camera: s.camera_prototype.clone(),
            character: s.character.save_state(),
            ship: s.ship.save_state(),
            terrain: s.mining.terrain,
            ship_gravity_disabled: s.mining.ship_gravity_disabled,
            mining_config: s.mining.config,
            world: s.mining.session.save_state(),
            anchors: s.mining.ship_fragments.clone(),
            motion: s.mining.fragment_motion.clone(),
            sleep: s.mining.fragment_sleep.save_state(),
        }
    }
    pub(crate) fn restore(self) -> Result<RestoredPhysical, SaveError> {
        self.validate()?;
        let mut ship = salimon_ship::ShipController::from_save(self.ship)?;
        let query = (self.terrain == salimon_world::terrain::TerrainMode::Representative)
            .then_some(crate::terrain::character_ground as salimon_character::GroundQuery);
        let character = salimon_character::CharacterController::from_save(self.character, query)?;
        let mut mining = ResourceGameplay::from_saved_world(
            salimon_world::mining::MiningSession::from_save(self.world)?,
            self.anchors,
            self.motion,
            self.terrain,
            self.ship_gravity_disabled,
            self.mining_config,
        );
        mining.fragment_sleep = salimon_physics::SleepTracker::from_save(self.sleep, |id, p| {
            mining.session.fragment(*id)?;
            let mut p = p.clone();
            if let salimon_physics::Surface::Sphere(body) = p.surface {
                if !salimon_world::CELESTIAL_BODIES
                    .iter()
                    .any(|b| b.center.meters() == body.center && b.radius_meters == body.radius)
                {
                    return None;
                }
                if mining.terrain == salimon_world::terrain::TerrainMode::Representative {
                    p.surface = salimon_physics::Surface::Sampled {
                        body,
                        query: crate::terrain::physics_ground,
                        point_query: crate::terrain::physics_ground_point,
                    };
                }
            }
            Some(p)
        })?;
        let frame = super::frames::character_ship_frame(ship.snapshot().pose);
        crate::carrying::sync_ship_mass(&mut mining, frame, &mut ship);
        Ok(RestoredPhysical {
            seed: self.seed,
            view_mode: self.view_mode,
            camera: self.camera,
            character,
            ship,
            mining,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{self, Config, Scenario};
    use std::time::Duration;
    fn initialize(scenario: Scenario) -> GameplaySession {
        let mut s = GameplaySession::default();
        e2e::initialize(
            &mut s,
            Config {
                scenario,
                seed: 363,
                step: Duration::from_millis(16),
                fragment_load: None,
            },
        )
        .unwrap();
        s
    }
    fn roundtrip(s: &GameplaySession) -> GameplaySession {
        let source = PhysicalState::capture(s);
        source.validate().unwrap();
        let bytes = serde_json::to_vec(&source).unwrap();
        let decoded: PhysicalState = serde_json::from_slice(&bytes).unwrap();
        let r = decoded.restore().unwrap();
        let mut restored = GameplaySession {
            seed: r.seed,
            view_mode: r.view_mode,
            camera_prototype: r.camera,
            character: r.character,
            ship: r.ship,
            mining: r.mining,
            ..Default::default()
        };
        restored.sync_fabricator_mass();
        assert_eq!(
            serde_json::to_value(PhysicalState::capture(&restored)).unwrap(),
            serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()
        );
        assert!(restored.mining.fragment_previous.is_empty());
        assert_eq!(
            restored.movement_input,
            salimon_character::MovementInput::default()
        );
        restored
    }
    #[test]
    fn physical_roundtrip_landed_moving_eva_carrying_blackout_and_terrain() {
        for scenario in [
            Scenario::LandedEarth,
            Scenario::CoastingSpace,
            Scenario::EvaApproach,
            Scenario::CargoFlight,
            Scenario::CargoBlackout,
            Scenario::CoreRestart,
            Scenario::TerrainMoon,
            Scenario::TerrainMarsApproach,
        ] {
            let mut s = initialize(scenario);
            s.advance_automation(Duration::from_millis(48));
            let restored = roundtrip(&s);
            assert_eq!(s.snapshot().ship, restored.snapshot().ship);
            assert_eq!(s.snapshot().character, restored.snapshot().character);
            assert_eq!(
                s.mining.session.fragments(),
                restored.mining.session.fragments()
            );
        }
    }
    #[test]
    fn physical_roundtrip_five_hundred_fragments_and_contact_continuation() {
        let mut s = initialize(Scenario::LandedEarth);
        crate::benchmark_fixture::initialize(
            &mut s,
            500,
            crate::benchmark_fixture::Layout::Scattered,
            363,
            true,
        )
        .unwrap();
        s.advance_automation(Duration::from_millis(64));
        let mut restored = roundtrip(&s);
        for _ in 0..4 {
            s.advance_automation(Duration::from_millis(16));
            restored.advance_automation(Duration::from_millis(16));
        }
        assert_eq!(
            s.mining.session.fragments(),
            restored.mining.session.fragments()
        );
        assert_eq!(
            s.snapshot().ship.total_mass_kg,
            restored.snapshot().ship.total_mass_kg
        );
    }
    #[test]
    fn physical_sleep_and_mid_takeoff_continue_without_events() {
        let mut s = initialize(Scenario::LandedEarth);
        crate::benchmark_fixture::initialize(
            &mut s,
            8,
            crate::benchmark_fixture::Layout::SphereDeck,
            363,
            false,
        )
        .unwrap();
        for _ in 0..100 {
            s.advance_automation(Duration::from_millis(16));
        }
        let sleeping: Vec<_> = s
            .mining
            .session
            .fragments()
            .iter()
            .filter(|f| s.mining.fragment_sleep.is_sleeping(f.id()))
            .map(|f| f.id())
            .collect();
        assert!(!sleeping.is_empty());
        let restored = roundtrip(&s);
        for id in sleeping {
            assert!(restored.mining.fragment_sleep.is_sleeping(id));
        }
        let mut s = initialize(Scenario::CockpitEarth);
        s.ship.trigger_landing_action();
        s.advance_automation(Duration::from_millis(512));
        assert!(matches!(
            s.ship.snapshot().flight_state,
            salimon_ship::FlightState::AssistedTakeoff { .. }
        ));
        let mut restored = roundtrip(&s);
        for _ in 0..10 {
            s.advance_automation(Duration::from_millis(16));
            restored.advance_automation(Duration::from_millis(16));
        }
        assert_eq!(s.snapshot().ship, restored.snapshot().ship);
    }
    #[test]
    fn physical_rejects_dangling_identity_and_invalid_domain_state() {
        let s = initialize(Scenario::LandedEarth);
        let mut state = PhysicalState::capture(&s);
        state.anchors.insert(
            FragmentId(999),
            salimon_character::ShipLocalPoint::new([0.; 3]),
        );
        assert!(state.restore().is_err());
        let mut value = serde_json::to_value(PhysicalState::capture(&s)).unwrap();
        value["ship"]["door_open_fraction"] = serde_json::json!(2.0);
        assert!(
            serde_json::from_value::<PhysicalState>(value)
                .unwrap()
                .restore()
                .is_err()
        );
    }
}

#[cfg(test)]
mod journal_tests {
    use salimon_world::{
        CelestialBodyId, WorldPosition,
        mining::MiningSession,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId},
    };
    use std::time::Duration;
    #[test]
    fn depletion_survives_streaming_and_consumed_ids_never_reallocate() {
        let make = || {
            ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 123,
                },
                RawMaterial::new(ResourceId::IronOre, 20.).unwrap(),
                WorldPosition::new(0., 0., 0.),
                20.,
            )
            .unwrap()
        };
        let mut world = MiningSession::default();
        let mut deposit = make();
        world.extract(&mut deposit, Duration::from_millis(500));
        let id = world.fragments()[0].id();
        assert!(world.remove_fragment(id));
        let saved = world.save_state();
        let mut restored = MiningSession::from_save(
            serde_json::from_slice(&serde_json::to_vec(&saved).unwrap()).unwrap(),
        )
        .unwrap();
        let mut streamed = make();
        // Reconcile through the ordinary extraction path, including a zero tick.
        restored.extract(&mut streamed, Duration::ZERO);
        assert_eq!(streamed.remaining_mass_kg(), deposit.remaining_mass_kg());
        restored.extract(&mut streamed, Duration::from_millis(500));
        assert!(restored.fragments()[0].id().0 > id.0);
        assert_eq!(restored.save_state().extracted_mass_kg, 2.);
    }
}
