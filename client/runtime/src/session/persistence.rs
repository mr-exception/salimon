//! Explicit versioned save composition. Owners validate/reconstruct; publish once.
use super::{GameplaySession, ViewMode};
#[cfg(test)]
mod contracts;
#[cfg(test)]
mod journal_tests;
mod legacy;
mod physical;
use super::survival::save::Snapshot as SurvivalState;
use legacy::migrate_propulsion;
use physical::PhysicalState;
use salimon_persistence::{SaveError, Validate, invalid};

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionState {
    physical: PhysicalState,
    #[serde(default)]
    offline: Option<super::offline::OfflineClock>,
    #[serde(default)]
    surface_age: Option<salimon_ship::surface_age::SurfaceAge>,
    #[serde(default)]
    finishes: salimon_ship::finishes::Finishes,
    #[serde(default)]
    lighting: salimon_ship::lighting::Lighting,
    #[serde(default)]
    core_upgrades: salimon_ship::core_upgrades::CoreUpgrades,
    #[serde(default)]
    mounts: Option<salimon_ship::attachments::Attachments>,
    #[serde(default)]
    cargo_door: Option<super::cargo::CargoDoor>,
    #[serde(default)]
    habitation: Option<super::habitation::Habitation>,
    #[serde(default)]
    archive: Option<super::archive::Archive>,
    #[serde(default)]
    recovery: Option<super::recovery::Recovery>,
    #[serde(default)]
    construction: Option<super::construction::ConstructionSave>,
    fabrication: salimon_fabrication::FabricationSaveState,
    equipment: crate::equipment::EquipmentToolbar,
    #[serde(default)]
    tutorial: Option<super::tutorial::Tutorial>,
    #[serde(default)]
    survival: Option<SurvivalState>,
}
impl Validate for SessionState {
    fn validate(&self) -> Result<(), SaveError> {
        self.physical.validate()?;
        if let Some(clock) = &self.offline {
            clock.validate()?;
            clock.validate_sites(
                self.construction
                    .iter()
                    .flat_map(|c| c.projects.jobs().map(|p| p.id.0)),
            )?;
        }
        if let Some(age) = &self.surface_age {
            age.validate()?;
        }
        self.finishes.validate()?;
        super::lighting::save::validate_context(&self.lighting)?;
        self.core_upgrades.validate()?;
        if let Some(m) = &self.mounts {
            m.validate()?;
        }
        if let Some(h) = &self.habitation {
            h.validate()?;
        }
        if let Some(door) = &self.cargo_door {
            door.validate()?;
        }
        if let Some(construction) = &self.construction {
            construction.validate()?;
        }
        if let Some(archive) = &self.archive {
            archive.validate()?;
        }
        if let Some(tutorial) = &self.tutorial {
            tutorial.validate()?;
        }
        self.fabrication.validate()?;
        super::recovery::save::validate_with_fabrication(&self.recovery, &self.fabrication)?;
        if let Some(survival) = &self.survival {
            survival.validate()?;
        }
        self.equipment.validate_save(
            self.physical.has_carried_fragment()
                || self.mounts.as_ref().is_some_and(|m| m.carried.is_some()),
        )?;
        Ok(())
    }
}
impl GameplaySession {
    pub(crate) fn save_bytes(&self, now_unix_seconds: u64) -> Result<Vec<u8>, SaveError> {
        let state = SessionState {
            physical: PhysicalState::capture(self),
            offline: Some(
                self.offline
                    .captured(now_unix_seconds, self.construction_safety_record()),
            ),
            surface_age: Some(self.surface_age.clone()),
            finishes: self.finishes.clone(),
            lighting: self.lighting.clone(),
            core_upgrades: self.core_upgrades.clone(),
            mounts: Some(self.mounts.clone()),
            cargo_door: Some(self.cargo_door.clone()),
            habitation: Some(self.habitation.clone()),
            archive: Some(self.archive.clone()),
            recovery: Some(self.recovery.clone()),
            construction: Some(self.building.source.clone()),
            fabrication: salimon_fabrication::FabricationSaveState::capture(
                &self.fabricator,
                &self.consumables,
            ),
            equipment: self.equipment.clone(),
            tutorial: Some(self.tutorial.clone()),
            survival: Some(SurvivalState::capture(self)),
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
        let surface_age =
            super::surface_age::save::restore(save.state.surface_age, save.schema_version)?;
        let offline = super::offline::save::restore(
            save.state.offline,
            save.schema_version,
            save.saved_at_unix_seconds,
        )?;
        let survival = super::survival::save::restore(save.state.survival, save.schema_version)?;
        let tutorial = super::tutorial::save::restore(save.state.tutorial, save.schema_version)?;
        let archive = super::archive::save::restore(save.state.archive, save.schema_version)?;
        let recovery = super::recovery::save::restore(save.state.recovery, save.schema_version)?;
        let construction =
            super::construction::save::restore(save.state.construction, save.schema_version)?;
        let cargo_door = super::cargo::save::restore(save.state.cargo_door, save.schema_version)?;
        let habitation =
            super::habitation::save::restore(save.state.habitation, save.schema_version)?;
        super::lighting::save::validate_schema(&save.state.lighting, save.schema_version)?;
        let (fabricator, consumables) =
            super::fabrication::save::restore(save.state.fabrication, save.schema_version)?;
        let physical = save.state.physical.restore()?;
        let mounts = super::mounts::save::restore(save.state.mounts, save.schema_version)?;
        super::finishes::save::validate_schema(&save.state.finishes, save.schema_version)?;
        super::core_upgrades::save::validate_schema(
            &save.state.core_upgrades,
            save.schema_version,
        )?;
        let mut restored = Self {
            lighting: save.state.lighting,
            core_upgrades: save.state.core_upgrades,
            surface_age,
            finishes: save.state.finishes,

            mounts,
            building: super::construction::Building::from_source(construction)?,
            seed: physical.seed,
            cargo_door,
            habitation,
            archive,
            recovery,
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
            offline,
            elapsed: save.simulated_time,
            update_clock: crate::update_clock::UpdateClock::from_saved_backlog(save.pending_time),
            // Automation policy belongs to the current host, never the save file.
            policy: self.policy,
            ..Default::default()
        };
        if restored.cargo_door.open_fraction < 1.0
            && !restored.cargo_sweep_clear(0.0, restored.cargo_door.open_fraction)
        {
            return Err(invalid(
                "restored cargo door overlaps player or physical cargo",
            ));
        }
        restored.core_upgrades.validate_context(
            super::core_upgrades::catalog(),
            &restored.mining.session,
            &restored.ship,
        )?;
        restored.validate_construction_receipts()?;
        restored.validate_finishes()?;
        restored.validate_surface_age()?;
        restored.validate_cabin_restore()?;
        restored.validate_mounts()?;
        super::mounts::save::reconcile_carried_mass(&mut restored)?;
        restored.sync_mount_collision();
        restored.sync_cargo_geometry();
        let frame = super::frames::character_ship_frame(restored.ship.snapshot().pose);
        crate::carrying::sync_ship_mass(&mut restored.mining, frame, &mut restored.ship);
        restored.sync_fabricator_mass();
        restored
            .ship
            .set_fabricator_watts(restored.fabricator.power_demand().map_or(0., |d| d.watts))
            .map_err(|_| invalid("restored machine/Core demand"))?;
        restored.enforce_survival_restrictions();
        if restored.survival.snapshot().terminal {
            restored.freeze_terminal();
        }
        restored.audio_epoch = self.audio_epoch.wrapping_add(1);
        *self = restored;
        Ok(())
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
            (self.survival.snapshot().terminal, (self.building.source.projects.jobs().map(|p|(p.id,p.work.completed,p.work.damaged,p.work.stopped,p.work.destroyed)).collect::<Vec<_>>(), (&self.building.source.removed_partitions,((&self.building.source.open_doors,&self.building.source.damage),self.module_checkpoint())))), self.survival.active_effects().map(|(id,stacks,_)|(id,stacks)).collect::<Vec<_>>(),(self.run.serial, ship.engines_enabled), (&self.tutorial, (&self.archive, (&self.finishes, (&self.lighting,&self.core_upgrades))), ((self.mounts.carried.as_ref().map(|c|c.id), self.mounts.mounts.iter().map(|m|(m.id,m.rail,m.secured)).collect::<Vec<_>>()), self.recovery.checkpoint_events()), (self.cargo_door.opening,self.cargo_door.open_fraction == 0.0,self.cargo_door.open_fraction == 1.0), (self.habitation.bunk_deploying,self.habitation.bunk_open_fraction==0.0,self.habitation.bunk_open_fraction==1.0,self.cabin_checkpoint()))))
            .expect("discrete checkpoint events contain only serializable finite identities and integer quantities")
    }
}
