//! Empty reconstruction state and explicit grant-bearing new-run construction.
use super::*;

impl Default for GameplaySession {
    fn default() -> Self {
        Self {
            paused: false,
            scanner: Default::default(),
            chart: salimon_universe::scanner::chart::Chart::new(scanner::CATALOG.revision())
                .expect("checked chart catalog"),
            surface_chart: Default::default(),
            field_marks: Default::default(),
            navigation_cache: Default::default(),
            offline: Default::default(),
            offline_power: false,
            audio_epoch: 0,
            mount_audio: (0, [0.; 3]),
            lighting: Default::default(),
            core_upgrades: Default::default(),
            surface_age: Default::default(),
            finishes: Default::default(),
            finish_group: 0,
            mounts: Default::default(),
            cargo_door: Default::default(),
            habitation: Default::default(),
            building: Default::default(),
            archive: Default::default(),
            recovery: Default::default(),
            archive_reader: Default::default(),
            tutorial: tutorial::Tutorial::dismissed(),
            run: Default::default(),
            consumable_effects: salimon_survival::consumables::Catalog::starter(),
            survival: Default::default(),
            restriction_config: Default::default(),
            interaction_ready_at: Duration::ZERO,
            presentation: Default::default(),
            update_clock: UpdateClock::default(),
            camera_prototype: CameraPrototype::default(),
            character: CharacterController::default(),
            ship: ShipController::default(),
            automation_aim_delta: None,
            mining: Default::default(),
            fabricator: salimon_fabrication::Fabricator::starter(),
            crafting_ui: Default::default(),
            consumables: Default::default(),
            wheel: Default::default(),
            equipment: Default::default(),
            input: Default::default(),
            view_mode: ViewMode::default(),
            seed: 0,
            policy: Default::default(),
            latest_physics: Default::default(),
            feedback: None,
            elapsed: Duration::ZERO,
        }
    }
}
impl GameplaySession {
    /// Grants starter supplies only for a new run, never for restore.
    pub(crate) fn new_run() -> Self {
        let (fabricator, consumables) = salimon_fabrication::Fabricator::starter_with_supplies();
        let mut session = Self {
            mounts: salimon_ship::attachments::Attachments::starter(),
            habitation: habitation::Habitation::starter(),
            fabricator,
            consumables,
            tutorial: tutorial::Tutorial::default(),
            ..Default::default()
        };
        session.sync_fabricator_mass();
        session.sync_accessory();
        session
    }
}
