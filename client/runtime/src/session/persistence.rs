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
        let mut total = salimon_ship::STARTER_DRY_MASS_KG;
        for fragment in &self.world.fragments {
            total += fragment.material().mass_kg();
            if !total.is_finite() {
                return Err(invalid("physical mass overflow"));
            }
        }
        salimon_world::spherical_mining::MiningConfig::new(
            self.mining_config.cap_kg(),
            self.mining_config.rate_kg_per_second(),
        )
        .map_err(|_| invalid("mining config"))?;
        self.camera.validate_save()?;
        let ids: std::collections::HashSet<_> =
            self.world.fragments.iter().map(|f| f.id()).collect();
        // Validate logical sleep state even when decoding without publishing it.
        salimon_physics::SleepTracker::from_save(self.sleep.clone(), |id, p| {
            ids.contains(id).then(|| p.clone())
        })?;
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
            view_mode: ViewMode::Gameplay,
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
            camera: self.camera,
            character,
            ship,
            mining,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionState {
    physical: PhysicalState,
    #[serde(default)]
    archive: Option<super::archive::Archive>,
    fabrication: salimon_fabrication::FabricationSaveState,
    equipment: crate::equipment::EquipmentToolbar,
    #[serde(default)]
    tutorial: Option<super::tutorial::Tutorial>,
    #[serde(default)]
    survival: Option<SurvivalState>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct SurvivalState {
    vitals: salimon_survival::Survival,
    catalog: salimon_survival::consumables::Catalog,
    restrictions: salimon_survival::restrictions::RestrictionConfig,
    run: super::lifecycle::RunState,
}
impl Validate for SurvivalState {
    fn validate(&self) -> Result<(), SaveError> {
        self.vitals.validate()?;
        self.catalog.validate_registered(&self.vitals)?;
        self.restrictions.validate()?;
        self.run.validate()
    }
}
impl Validate for SessionState {
    fn validate(&self) -> Result<(), SaveError> {
        self.physical.validate()?;
        if let Some(archive) = &self.archive {
            archive.validate()?;
        }
        if let Some(tutorial) = &self.tutorial {
            tutorial.validate()?;
        }
        self.fabrication.validate()?;
        if let Some(survival) = &self.survival {
            survival.validate()?;
        }
        self.equipment
            .validate_save(self.physical.world.carried.is_some())?;
        Ok(())
    }
}
impl GameplaySession {
    pub(crate) fn save_bytes(&self, now_unix_seconds: u64) -> Result<Vec<u8>, SaveError> {
        let state = SessionState {
            physical: PhysicalState::capture(self),
            archive: Some(self.archive.clone()),
            fabrication: salimon_fabrication::FabricationSaveState::capture(
                &self.fabricator,
                &self.consumables,
            ),
            equipment: self.equipment.clone(),
            tutorial: Some(self.tutorial.clone()),
            survival: Some(SurvivalState {
                vitals: self.survival.clone(),
                catalog: self.consumable_effects.clone(),
                restrictions: self.restriction_config,
                run: self.run.clone(),
            }),
        };
        salimon_persistence::SaveGame::new(
            state,
            self.elapsed,
            self.update_clock.backlog(),
            now_unix_seconds,
        )?
        .encode()
    }
    /// Decode, validate and reconstruct in isolation, then publish exactly once.
    /// Native focus/capture and filesystem scheduling remain shell-owned (#366).
    pub(crate) fn restore_bytes(&mut self, bytes: &[u8]) -> Result<(), SaveError> {
        let migrated = migrate_propulsion(bytes)?;
        let save = salimon_persistence::SaveGame::<SessionState>::decode(&migrated)?;
        let mut survival = match (save.schema_version, save.state.survival) {
            (1, None) => SurvivalState {
                vitals: Default::default(),
                catalog: salimon_survival::consumables::Catalog::starter(),
                restrictions: Default::default(),
                run: Default::default(),
            },
            (2..=6, Some(state)) => state,
            _ => return Err(invalid("survival payload does not match envelope version")),
        };
        if save.schema_version == 2 {
            // Schema 2 recorded only a cumulative counter. There is no missing
            // movement to infer, and no restore displacement to charge.
            survival.run.journey.initialize_tracking();
        } else if survival.run.journey.tracking.is_none() {
            return Err(invalid("schema-3 journey sampling source missing"));
        }
        survival.validate()?;
        let tutorial = match (save.schema_version, save.state.tutorial) {
            (4..=6, Some(tutorial)) => tutorial,
            (1..=3, _) => super::tutorial::Tutorial::dismissed(),
            _ => return Err(invalid("tutorial source missing for save schema")),
        };
        let archive = match (save.schema_version, save.state.archive) {
            (6, Some(archive)) => archive,
            (1..=5, _) => Default::default(),
            _ => return Err(invalid("archive source missing for save schema")),
        };
        let (fabricator, consumables) = save.state.fabrication.restore()?;
        let physical = save.state.physical.restore()?;
        let mut restored = Self {
            seed: physical.seed,
            archive,
            tutorial,
            view_mode: ViewMode::Gameplay,
            survival: survival.vitals,
            consumable_effects: survival.catalog,
            restriction_config: survival.restrictions,
            run: survival.run,
            camera_prototype: physical.camera,
            character: physical.character,
            ship: physical.ship,
            mining: physical.mining,
            fabricator,
            consumables,
            equipment: save.state.equipment,
            elapsed: save.simulated_time,
            update_clock: crate::update_clock::UpdateClock::from_saved_backlog(save.pending_time),
            // Automation policy belongs to the current host, never the save file.
            automation_step: self.automation_step,
            interpolate_presentation: self.interpolate_presentation,
            measure_physics: self.measure_physics,
            ..Default::default()
        };
        restored.sync_fabricator_mass();
        restored
            .ship
            .set_fabricator_watts(restored.fabricator.power_demand().map_or(0., |d| d.watts))
            .map_err(|_| invalid("restored machine/Core demand"))?;
        restored.enforce_survival_restrictions();
        if restored.survival.snapshot().terminal {
            restored.freeze_terminal();
        }
        *self = restored;
        Ok(())
    }
}

/// v1-v4 force capacities are deliberately superseded, not converted into an
/// arbitrary coefficient. Preserve energy, velocity and all other source state.
fn migrate_propulsion(bytes: &[u8]) -> Result<Vec<u8>, SaveError> {
    if bytes.len() > salimon_persistence::MAX_SAVE_BYTES {
        return Err(invalid("save exceeds size limit"));
    }
    #[derive(serde::Deserialize)]
    struct Header {
        schema_version: u32,
    }
    let version = serde_json::from_slice::<Header>(bytes)
        .map_err(|e| SaveError::Encoding(e.to_string()))?
        .schema_version;
    if !(1..=salimon_persistence::SCHEMA_VERSION).contains(&version) {
        return Err(SaveError::UnsupportedVersion(version));
    }
    if version == salimon_persistence::SCHEMA_VERSION {
        // Preserve typed decoding's duplicate-field rejection, and avoid a
        // full DOM/reserialization pass for ordinary current-schema restores.
        return Ok(bytes.to_vec());
    }
    let mut value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| SaveError::Encoding(e.to_string()))?;
    if version <= 4 {
        let ship = value["state"]["physical"]["ship"]
            .as_object_mut()
            .ok_or_else(|| invalid("legacy ship source missing"))?;
        ship.entry("engines_enabled").or_insert(true.into());
        let p = ship
            .get_mut("propulsion")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(|| invalid("legacy propulsion missing"))?;
        if p.contains_key("forward_force_newtons") || p.contains_key("reverse_force_newtons") {
            for key in ["forward_force_newtons", "reverse_force_newtons"] {
                if !p
                    .get(key)
                    .and_then(serde_json::Value::as_f64)
                    .is_some_and(|v| v.is_finite() && v >= 0.0)
                {
                    return Err(invalid("legacy force configuration"));
                }
                p.remove(key);
            }
            p.insert("impulse_ns_per_joule".into(), 1.0.into());
        }
    }
    serde_json::to_vec(&value).map_err(|e| SaveError::Encoding(e.to_string()))
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
            view_mode: ViewMode::Gameplay,
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

impl GameplaySession {
    /// Discrete gameplay events only: movement/progress remain covered by the
    /// periodic timer. No presentation or held input becomes save authority.
    pub(crate) fn checkpoint_events(&self) -> Vec<u8> {
        let machine = self
            .fabricator
            .snapshot(self.ship.snapshot().power_available);
        let ship = self.ship.snapshot();
        let jobs: Vec<_> = machine
            .jobs
            .iter()
            .map(|j| (j.id, j.remaining_quantity, j.completed_quantity))
            .collect();
        serde_json::to_vec(&(self.mining.session.fragments().len(), self.mining.session.fragments().last().map(|f| f.id()), self.mining.session.attached_sphere_id(), self.mining.session.carried_id(),
            ship.flight_state, ship.door_state, ship.power_available,
            machine.stored, jobs, machine.tray, &self.consumables, &self.equipment,
            self.survival.snapshot().terminal, self.survival.active_effects().map(|(id,stacks,_)|(id,stacks)).collect::<Vec<_>>(),(self.run.serial, ship.engines_enabled), (&self.tutorial, &self.archive)))
            .expect("discrete checkpoint events contain only serializable finite identities and integer quantities")
    }
}
