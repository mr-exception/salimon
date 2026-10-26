//! Maps physical identities and successful use to cosmetic exposure. Never repairs condition.
pub(super) mod save;
use super::GameplaySession;
use salimon_ship::surface_age::*;
use std::{collections::BTreeSet, time::Duration};

impl GameplaySession {
    fn surface_ids(&self) -> BTreeSet<u64> {
        let mut ids: BTreeSet<_> = [
            FABRICATOR,
            GALLEY,
            CABIN_FLOOR,
            CARGO_FLOOR,
            PILOT_SEAT,
            TOOL,
            1001,
        ]
        .into();
        if self.habitation.cabin.is_some() {
            ids.extend([3001, 3002, 3010, 3050, 3020]);
        }
        for m in &self.mounts.mounts {
            ids.extend([m.id, m.rail]);
        }
        // Tombstones retain the history of detached/destroyed identities; replacement IDs start fresh.
        for p in self
            .building
            .source
            .projects
            .jobs()
            .filter(|p| p.work.completed)
        {
            ids.insert(project(p.id.0));
            for i in 0..p.cells.len() {
                ids.insert(part(p.id.0 * 1024 + 1024 + i as u64));
            }
        }
        ids
    }
    pub(super) fn sync_surface_age(&mut self) {
        for id in self.surface_ids() {
            self.surface_age.register(id);
        }
    }
    pub(super) fn validate_surface_age(&self) -> Result<(), salimon_persistence::SaveError> {
        let ids = self.surface_ids();
        if self.surface_age.ids().any(|id| !ids.contains(&id)) {
            return Err(salimon_persistence::invalid(
                "surface age has no physical identity",
            ));
        }
        Ok(())
    }
    pub(super) fn advance_surface_age(&mut self, delta: Duration) {
        self.sync_surface_age();
        self.surface_age.advance(delta);
        if !self.offline_power {
            if self.ship.snapshot().cockpit_control_active {
                self.surface_age.contact(PILOT_SEAT, delta);
            }
            if let Some(r) = self.habitation.cabin.as_ref().and_then(|c| c.rest.as_ref()) {
                let id = match r.kind {
                    super::cabin::RestKind::Seat => 3002,
                    super::cabin::RestKind::Bunk => 1001,
                };
                // Fabric contact is gentler than machinery/track cycles.
                self.surface_age.contact(id, delta / 60);
            }
        }
    }
    pub(super) fn record_floor_contact(&mut self, before: Option<[f64; 3]>, delta: Duration) {
        if let (Some(a), Some(b)) = (before, self.character.local_ship_position()) {
            let d = salimon_math::length(salimon_math::sub(a, b));
            if d > 0.00001 && d <= delta.as_secs_f64() * 20. {
                self.surface_age.contact(
                    if b[0] < -4.65 {
                        CARGO_FLOOR
                    } else {
                        CABIN_FLOOR
                    },
                    delta,
                );
                // Completed cells own their wear even if neighboring partitions change.
                for p in self.building.structure.parts().filter(|p| p.id.0 >= 1024) {
                    if (0..3).all(|i| b[i] >= p.space.min[i] && b[i] <= p.space.max[i]) {
                        self.surface_age.contact(part(p.id.0), delta);
                    }
                }
            }
        }
    }
    pub(crate) fn ship_surface_ages(&self) -> [f32; 8] {
        [
            CABIN_FLOOR,
            CARGO_FLOOR,
            FABRICATOR,
            GALLEY,
            PILOT_SEAT,
            TOOL,
            1001,
            3020,
        ]
        .map(|id| self.surface_age.intensity(id))
    }
    pub(crate) fn fixture_surface_age(
        &self,
        id: u64,
        mesh: salimon_renderer::FixtureMesh,
    ) -> [f32; 4] {
        use salimon_renderer::FixtureMesh as M;
        let (id, material) = match mesh {
            M::Bunk => (1001, 2.),
            M::SeatPad | M::SeatBack => (3002, 2.),
            M::LockerDoor | M::LockerShell => (3020, 0.),
            M::SeatBase => (3002, 1.),
            M::ScreenPanel => (3010, 2.),
            M::Shutter | M::ShutterHandle => (3050, 1.),
            M::Harness => return [0.; 4],
            _ => (
                if id > 20000 && id < 23000 {
                    id - 20000
                } else if id > 10000 && id < 13000 {
                    id - 10000
                } else {
                    id
                },
                1.,
            ),
        };
        [
            self.surface_age.intensity(id),
            material,
            (id % 17) as f32,
            0.,
        ]
    }
}

impl GameplaySession {
    /// A shared flush service cover is a cosmetic layer on existing installed
    /// geometry. A scarred variant follows real condition; no collider/rigid body.
    pub(crate) fn construction_surface_instances(
        &self,
        ship: salimon_ship::ShipSnapshot,
    ) -> Vec<salimon_renderer::FixtureInstance> {
        use salimon_renderer::{FixtureInstance, FixtureMesh, FixturePose};
        let frame = super::frames::character_ship_frame(ship.pose);
        let light = self.lighting_dto();
        let cover = |id, space: salimon_ship::structure::Envelope, damaged| {
            let furniture = id & (1 << 63) != 0;
            // Furniture covers sit against the solid upper edge (the stool seat),
            // never suspended in the open volume between its legs.
            let local = [
                (space.min[0] + space.max[0]) / 2.,
                if furniture {
                    space.max[1] - 0.05
                } else {
                    (space.min[1] + space.max[1]) / 2.
                },
                if furniture {
                    space.min[2] - 0.005
                } else {
                    space.max[2] - 0.12
                },
            ];
            FixtureInstance {
                id,
                mesh: if damaged {
                    FixtureMesh::PanelDamaged
                } else {
                    FixtureMesh::Panel
                },
                pose: FixturePose {
                    position_meters: frame.local_to_world(local),
                    orientation_xyzw: ship.pose.orientation,
                    scale: if furniture {
                        0.06
                    } else {
                        ((space.max[1] - space.min[1]).min(space.max[0] - space.min[0]) * 0.3)
                            .min(0.45)
                    },
                },
                tint: [1.; 3],
                powered: ship.power_available,
                illumination: Some(light.sample(local)),
                surface_age: [self.surface_age.intensity(id), 0., (id % 17) as f32, 0.],
            }
        };
        let mut out = Vec::new();
        for p in self.building.structure.parts().filter(|p| p.id.0 >= 1024) {
            out.push(cover(
                part(p.id.0),
                p.space,
                self.building.source.damage.damaged.contains(&p.id),
            ));
        }
        for p in self.building.source.projects.jobs().filter(|p| {
            p.work.completed
                && !p.work.destroyed
                && !matches!(
                    p.blueprint.id.as_str(),
                    "hull-cell" | "deck-cell" | "two-deck-room"
                )
        }) {
            out.push(cover(project(p.id.0), p.space, p.work.damaged));
        }
        out
    }
}

/// Explicit evidence seeds; repeated latch cycles run the real clearance/motion owner.
/// Passive seed changes exposure only, not the world's survival or power clock.
pub(crate) fn evidence_initial_conditions(
    s: &mut GameplaySession,
    scenario: &str,
) -> Result<(), String> {
    use super::{
        Command,
        cabin::Furniture,
        construction::damage::{Impact, Target},
    };
    if matches!(
        scenario,
        "surface-age-damage" | "surface-age-repair" | "surface-age-replacement"
    ) {
        super::construction::evidence_initial_conditions(s, true)?;
        s.command(Command::Interact);
        s.advance_construction(Duration::from_secs(10));
        s.sync_surface_age();
        let id = s
            .building
            .source
            .projects
            .jobs()
            .next()
            .ok_or("missing fixture project")?
            .id;
        s.surface_age
            .contact(project(id.0), Duration::from_secs(1200));
        if scenario == "surface-age-replacement" {
            // Declared evidence camera clears the existing conservative removal
            // envelope; removal uses its typed physical boundary, then ordinary placement/intake.
            let eye = [
                1.0,
                salimon_character::SHIP_FLOOR_HEIGHT_METERS
                    + salimon_character::PLAYER_EYE_HEIGHT_METERS,
                1.8,
            ];
            let target = [-1.25, salimon_character::SHIP_FLOOR_HEIGHT_METERS, 3.25];
            let d = salimon_math::sub(target, eye);
            let mut camera =
                serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
            camera["position"] = serde_json::json!({"Inside":{"local":eye}});
            camera["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
            camera["pitch_radians"] = (d[1] / salimon_math::length(d)).asin().into();
            s.character = salimon_character::CharacterController::from_save(
                serde_json::from_value(camera).map_err(|e| e.to_string())?,
                None,
            )
            .map_err(|e| e.to_string())?;
            s.dismantle_construction(Target::Project(id))
                .map_err(|e| format!("replacement dismantle: {e}"))?;
            let fragment = s
                .mining
                .session
                .fragments()
                .first()
                .ok_or_else(|| format!("replacement salvage absent: {:?}", s.feedback))?
                .id();
            if !s.mining.session.pick_up(fragment) {
                return Err("replacement salvage pickup rejected".into());
            }
            s.sync_carried();
            s.command(Command::Build(super::construction::BuildAction::Place));
            s.command(Command::Interact);
            s.advance_construction(Duration::from_secs(10));
            let replacement = s
                .building
                .source
                .projects
                .jobs()
                .find(|p| p.id != id && p.work.completed && !p.work.destroyed)
                .ok_or("replacement not physically completed")?
                .id;
            if s.surface_age.intensity(project(replacement.0)) != 0.
                || s.surface_age.intensity(project(id.0)) < 0.31
            {
                return Err("replacement inherited or erased old age".into());
            }
            return Ok(());
        }
        s.construction_impact(Impact {
            target: Target::Project(id),
            destroy: false,
            survival_ppm: 1_000_000,
        })
        .map_err(str::to_owned)?;
        if scenario == "surface-age-repair" {
            s.command(Command::Build(super::construction::BuildAction::Repair));
        }
    } else {
        super::cabin::evidence_initial_conditions(s, "scout-cabin-locker")?;
        s.sync_surface_age();
        if scenario == "surface-age-idle" {
            s.surface_age.advance(Duration::from_secs(86400));
        } else if scenario != "surface-age-new" && scenario != "surface-age-offline" {
            for _ in 0..1200 {
                s.toggle_cabin_fixture(Furniture::Locker);
                s.advance_cabin(Duration::from_millis(650));
            }
            if s.surface_age.intensity(3020) < 0.31 {
                return Err("evidence latch cycles obstructed".into());
            }
        }
        if scenario == "surface-age-offline" {
            s.reconcile_construction_offline(Duration::from_secs(86400), &[])
                .map_err(|e| e.to_string())?;
        }
        if scenario == "surface-age-recolor" {
            s.finish_action(super::finishes::FinishAction::Next);
        }
        if scenario == "surface-age-blackout" {
            s.ship.set_stored_core_energy_joules(0);
        }
        if scenario == "surface-age-reload" {
            let bytes = s.save_bytes(100).map_err(|e| e.to_string())?;
            s.restore_bytes(&bytes).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Command, finishes::FinishAction};
    use salimon_renderer::FixtureMesh;
    fn copy(s: &GameplaySession) -> GameplaySession {
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
        r
    }
    #[test]
    fn surface_age_actual_latch_travel_recolor_blackout_pause_and_restore() {
        let mut s = GameplaySession::new_run();
        super::super::cabin::evidence_initial_conditions(&mut s, "scout-cabin-locker").unwrap();
        s.sync_surface_age();
        for _ in 0..600 {
            for _ in 0..2 {
                s.toggle_cabin_fixture(super::super::cabin::Furniture::Locker);
                s.advance_cabin(Duration::from_millis(650));
            }
        }
        assert!(
            s.surface_age.intensity(3020) > 0.31,
            "actual admitted cycles must add wear"
        );
        let age = s.surface_age.clone();
        s.finish_action(FinishAction::Next);
        assert_eq!(s.surface_age, age);
        s.ship.set_stored_core_energy_joules(0);
        assert_eq!(
            s.fixture_instances(s.ship.snapshot())
                .iter()
                .find(|i| i.mesh == FixtureMesh::LockerDoor)
                .unwrap()
                .surface_age[0],
            s.surface_age.intensity(3020)
        );
        s.command(Command::TogglePause);
        s.advance_game(Duration::from_secs(100));
        assert_eq!(s.surface_age, age);
        assert_eq!(copy(&s).surface_age, age);
    }
    #[test]
    fn surface_age_offline_partition_and_no_restore_credit_or_occupant_use() {
        let mut a = GameplaySession::new_run();
        a.sync_surface_age();
        let mut b = copy(&a);
        a.reconcile_construction_offline(Duration::from_secs(60), &[])
            .unwrap();
        for _ in 0..60 {
            b.reconcile_construction_offline(Duration::from_secs(1), &[])
                .unwrap();
        }
        assert_eq!(a.surface_age, b.surface_age);
        let restored = copy(&a);
        assert_eq!(restored.surface_age, a.surface_age);
        assert!(a.surface_age.intensity(1001) < 0.00001);
    }
    #[test]
    fn surface_age_real_damage_clean_repair_retains_age_and_lost_cell_identity() {
        use super::super::construction::{
            damage::{Impact, Target},
            room_tests::room,
        };
        use salimon_ship::structure::StructureId;
        let (mut s, _) = room();
        s.advance_construction(Duration::from_secs(300));
        s.sync_surface_age();
        let id = StructureId(2048);
        let key = part(id.0);
        s.surface_age.contact(key, Duration::from_secs(1200));
        let age = s.surface_age.clone();
        let pristine = s.construction_surface_instances(s.ship.snapshot());
        s.construction_impact(Impact {
            target: Target::Part(id),
            destroy: false,
            survival_ppm: 1_000_000,
        })
        .unwrap();
        let damaged = s.construction_surface_instances(s.ship.snapshot());
        assert_eq!(
            damaged.iter().find(|i| i.id == key).unwrap().mesh,
            FixtureMesh::PanelDamaged
        );
        assert_eq!(s.surface_age, age);
        // Physical repair's canonical condition update is the only visual input.
        s.building.source.damage.damaged.remove(&id);
        assert_eq!(
            s.construction_surface_instances(s.ship.snapshot()),
            pristine
        );
        assert_eq!(copy(&s).surface_age, age);
        s.construction_impact(Impact {
            target: Target::Part(id),
            destroy: true,
            survival_ppm: 500_000,
        })
        .unwrap();
        assert!(
            !s.construction_surface_instances(s.ship.snapshot())
                .iter()
                .any(|i| i.id == key)
        );
        assert_eq!(copy(&s).surface_age, age);
    }
    #[test]
    fn surface_age_physical_replacement_uses_reclaimed_material_with_fresh_identity() {
        let mut s = GameplaySession::default();
        evidence_initial_conditions(&mut s, "surface-age-replacement").unwrap();
        let jobs: Vec<_> = s.building.source.projects.jobs().collect();
        assert_eq!(jobs.len(), 2);
        assert!(jobs[0].work.destroyed);
        assert!(jobs[1].work.completed && !jobs[1].work.destroyed);
        assert!(s.surface_age.intensity(project(jobs[0].id.0)) > 0.31);
        assert_eq!(s.surface_age.intensity(project(jobs[1].id.0)), 0.);
        assert_eq!(
            s.ship.snapshot().total_mass_kg,
            salimon_ship::STARTER_DRY_MASS_KG + 1.
        );
        assert_eq!(copy(&s).surface_age, s.surface_age);
    }
    #[test]
    fn surface_age_current_omission_unknown_identity_and_legacy_smuggling_are_atomic() {
        let mut s = GameplaySession::new_run();
        s.advance_surface_age(Duration::from_secs(1));
        let bytes = s.save_bytes(100).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let mut missing = value.clone();
        missing["state"]
            .as_object_mut()
            .unwrap()
            .remove("surface_age");
        assert!(
            s.restore_bytes(&serde_json::to_vec(&missing).unwrap())
                .is_err()
        );
        let mut unknown = value.clone();
        unknown["state"]["surface_age"]["surfaces"]["999999"] =
            serde_json::json!({"born_nanos":0,"use_nanos":0});
        assert!(
            s.restore_bytes(&serde_json::to_vec(&unknown).unwrap())
                .is_err()
        );
        let mut legacy = value;
        legacy["schema_version"] = 22.into();
        assert!(
            s.restore_bytes(&serde_json::to_vec(&legacy).unwrap())
                .is_err()
        );
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
        legacy["state"]
            .as_object_mut()
            .unwrap()
            .remove("surface_age");
        s.restore_bytes(&serde_json::to_vec(&legacy).unwrap())
            .unwrap();
        assert_eq!(s.surface_age, Default::default());
    }
}
