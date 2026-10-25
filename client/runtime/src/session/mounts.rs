//! Physical placement/explicit mechanical restraints over existing resource identities.
use super::{GameplaySession, frames::character_ship_frame};
use salimon_character::CharacterLocation;
use salimon_math::{dot, length, sub};
use salimon_world::resources::ResourceTransform;
#[derive(Clone, Copy, Debug)]
pub(crate) enum MountAction {
    Place,
    ToggleLock,
    MoveAccessory,
}
impl GameplaySession {
    fn aimed_mount(&self) -> Option<u64> {
        let s = self.snapshot();
        if s.character.location != CharacterLocation::InsideShip {
            return None;
        }
        let frame = character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        let look = sub(frame.world_to_local(s.character.look_target_meters), eye);
        self.mounts
            .mounts
            .iter()
            .filter(|m| self.mounts.carried.as_ref().is_none_or(|c| c.id != m.id))
            .filter_map(|m| {
                let p = m.position();
                let delta = sub(p, eye);
                let distance = length(delta);
                let visible =
                    salimon_character::ship_sight_obstruction(eye, p, super::door_passable(s.ship))
                        .is_none_or(|d| d >= distance - 0.02)
                        && (self.cargo_door.open_fraction == 1.
                            || salimon_character::cargo::cargo_sight_obstruction(eye, p)
                                .is_none_or(|d| d >= distance - 0.02));
                (distance <= 2.
                    && dot(delta, look) / (distance * length(look)).max(1e-12) >= 0.97
                    && visible)
                    .then_some((m.id, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|v| v.0)
    }
    pub(crate) fn mount_action(&mut self, action: MountAction) {
        let before = self.mounts.clone();
        let target = self
            .aimed_mount()
            .or_else(|| self.mounts.carried.as_ref().map(|c| c.id));
        self.execute_mount_action(action);
        if self.mounts != before {
            let position = target
                .and_then(|id| {
                    self.mounts
                        .mounts
                        .iter()
                        .find(|m| m.id == id)
                        .or_else(|| before.mounts.iter().find(|m| m.id == id))
                })
                .map(|m| m.position())
                .unwrap_or_else(|| {
                    character_ship_frame(self.ship.snapshot().pose)
                        .world_to_local(self.snapshot().character.eye_position_meters)
                });
            self.mount_audio = (self.mount_audio.0.wrapping_add(1), position);
        }
    }
    fn execute_mount_action(&mut self, action: MountAction) {
        if !self.admit_survival_action(salimon_survival::restrictions::Action::Carry) {
            return;
        }
        match action {
            MountAction::MoveAccessory => {
                self.move_accessory();
            }
            MountAction::Place if self.mounts.carried.is_some() => {
                self.move_accessory();
            }
            MountAction::Place => {
                self.place_fragment();
            }
            MountAction::ToggleLock => {
                let Some(id) = self.aimed_mount() else {
                    self.feedback = Some("Aim at a nearby physical clamp - K to secure/release");
                    return;
                };
                if let Some(item) = self.mounts.release(id) {
                    self.mining.secured.remove(&item);
                    self.mining.fragment_sleep = Default::default();
                    self.mining.fragment_motion.remove(&item);
                    self.sync_accessory();
                    self.feedback = Some("Clamp released; item is loose - F to carry");
                    return;
                }
                let frame = character_ship_frame(self.ship.snapshot().pose);
                let candidates: Vec<_> = self
                    .mining
                    .session
                    .fragments()
                    .iter()
                    .filter(|f| Some(f.id()) != self.mining.session.carried_id())
                    .map(|f| {
                        (
                            f.id(),
                            frame.world_to_local(f.transform().position().meters()),
                            crate::fragment_physics::sphere_radius(*f),
                        )
                    })
                    .collect();
                for (item, p, r) in candidates {
                    if self.mounts.secure(id, item, p, r) {
                        self.mining.secured.insert(item);
                        self.mining.fragment_sleep = Default::default();
                        self.mining
                            .ship_fragments
                            .insert(item, salimon_character::ShipLocalPoint::new(p));
                        self.mining.fragment_motion.remove(&item);
                        self.mining.fragment_previous.remove(&item);
                        self.sync_accessory();
                        self.feedback = Some("Mechanical clamp secured; K to release");
                        return;
                    }
                }
                self.feedback = Some("Place one fitting object on the mount before securing");
            }
        }
    }
    fn place_fragment(&mut self) {
        let s = self.snapshot();
        if s.character.location != CharacterLocation::InsideShip {
            self.feedback = Some("Use F to drop outside");
            return;
        }
        let Some(id) = self.mining.session.carried_id() else {
            self.feedback = Some("Carry one physical object to place");
            return;
        };
        let frame = character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        let look = frame.world_to_local(s.character.look_target_meters);
        let d = sub(look, eye);
        let piece = *self.mining.session.fragment(id).unwrap();
        let r = crate::fragment_physics::sphere_radius(piece);
        let mount = self
            .aimed_mount()
            .and_then(|id| self.mounts.mounts.iter().find(|m| m.id == id));
        if mount.is_some_and(|m| m.secured.is_some() || r > m.radius) {
            self.feedback = Some("Mount occupied or object too large");
            return;
        }
        let mut height = mount.map_or(salimon_character::SHIP_FLOOR_HEIGHT_METERS, |m| {
            m.position()[1] + 0.03
        }) + r
            + 0.005;
        if mount.is_none() && d[1] < -1e-9 {
            for b in &self.mining.interior_solids {
                let top = b[3] + r + 0.005;
                let t = (top - eye[1]) / d[1];
                let x = eye[0] + t * d[0];
                let z = eye[2] + t * d[2];
                if t > 0.
                    && top > height
                    && x - r >= b[0]
                    && x + r <= b[1]
                    && z - r >= b[4]
                    && z + r <= b[5]
                {
                    height = top;
                }
            }
        }
        let t = (height - eye[1]) / d[1];
        if !t.is_finite() || t <= 0. || length(d) * t > 2.5 {
            self.feedback = Some("Aim down at a reachable clear surface - F to drop");
            return;
        }
        let p = std::array::from_fn(|i| eye[i] + t * d[i]);
        let player_overlap = (p[0] - eye[0]).hypot(p[2] - eye[2])
            < r + salimon_character::PLAYER_RADIUS_METERS
            && p[1] + r > eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
        let fixture_obstruction = self.mining.mount_geometry.mounts.iter().any(|m| {
            let q = m.position();
            salimon_character::cargo::box_sight_obstruction(
                eye,
                p,
                [
                    q[0] - 0.5,
                    q[0] + 0.5,
                    q[1] - 0.03,
                    q[1] + 0.03,
                    q[2] - 0.175,
                    q[2] + 0.175,
                ],
            )
            .is_some_and(|d| d < length(sub(p, eye)) - r - 0.01)
        });
        if player_overlap
            || self.mining.interior_solids.iter().any(|b| {
                salimon_character::cargo::sphere_overlaps_box(p, r, *b)
                    || salimon_character::cargo::box_sight_obstruction(eye, p, *b)
                        .is_some_and(|hit| hit < length(sub(p, eye)) - r - 0.01)
            })
            || fixture_obstruction
            || !salimon_character::cargo::cargo_position_allowed(
                p,
                r,
                self.cargo_door.open_fraction,
            )
            || mount.is_some_and(|m| {
                let q = sub(p, m.position());
                q[0].abs() + r > 0.5 || q[2].abs() + r > 0.175
            })
            || self.mining.session.fragments().iter().any(|f| {
                f.id() != id
                    && length(sub(
                        frame.world_to_local(f.transform().position().meters()),
                        p,
                    )) < r + crate::fragment_physics::sphere_radius(*f) + 0.005
            })
            || salimon_character::ship_sight_obstruction(eye, p, super::door_passable(s.ship))
                .is_some_and(|hit| hit < length(sub(p, eye)) - r)
            || (self.cargo_door.open_fraction < 1.
                && salimon_character::cargo::cargo_sight_obstruction(eye, p)
                    .is_some_and(|hit| hit < length(sub(p, eye)) - r))
        {
            self.feedback = Some("Placement obstructed; object remains carried");
            return;
        }
        let world = frame.local_to_world(p);
        let pose = ResourceTransform::new(
            salimon_world::WorldPosition::new(world[0], world[1], world[2]),
            piece.transform().orientation_xyzw(),
        )
        .unwrap();
        self.mining.session.drop_carried(pose);
        self.mining
            .ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new(p));
        self.mining.fragment_motion.insert(
            id,
            crate::fragment_physics::FragmentMotion {
                ship_orientation: Some(crate::fragment_physics::to_local_orientation(
                    frame,
                    pose.orientation_xyzw(),
                )),
                ..Default::default()
            },
        );
        self.sync_carried();
        self.feedback = Some("Object placed loose; K at clamp to explicitly secure");
    }
    pub(crate) fn validate_mounts(&mut self) -> Result<(), salimon_persistence::SaveError> {
        use salimon_persistence::{Validate, invalid};
        self.mounts.validate()?;
        if let Some(c) = &self.mounts.carried
            && !self.mining.session.carry_fixture(c.id)
        {
            return Err(invalid("multiple carried world objects"));
        }
        self.mining.mount_geometry = self.mounts.clone();
        if let Some(c) = &self.mounts.carried {
            self.mining.mount_geometry.mounts.retain(|m| m.id != c.id);
        }
        let s = self.snapshot();
        let frame = character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        for m in &self.mining.mount_geometry.mounts {
            let p = m.position();
            let bounds = [
                p[0] - 0.5,
                p[0] + 0.5,
                p[1] - 0.03,
                p[1] + 0.03,
                p[2] - 0.175,
                p[2] + 0.175,
            ];
            if s.character.location == CharacterLocation::InsideShip
                && eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS < bounds[3]
                && eye[0] + salimon_character::PLAYER_RADIUS_METERS > bounds[0]
                && eye[0] - salimon_character::PLAYER_RADIUS_METERS < bounds[1]
                && eye[2] + salimon_character::PLAYER_RADIUS_METERS > bounds[4]
                && eye[2] - salimon_character::PLAYER_RADIUS_METERS < bounds[5]
            {
                return Err(invalid("restored shelf overlaps player"));
            }
            if self.mining.session.fragments().iter().any(|f| {
                Some(f.id()) != self.mining.session.carried_id()
                    && salimon_character::cargo::sphere_overlaps_box(
                        frame.world_to_local(f.transform().position().meters()),
                        (crate::fragment_physics::sphere_radius(*f) - 0.0001).max(0.),
                        bounds,
                    )
            }) {
                return Err(invalid("restored shelf overlaps physical cargo"));
            }
        }
        self.mining.secured.clear();
        for m in &self.mounts.mounts {
            if let Some(id) = m.secured {
                let Some(f) = self.mining.session.fragment(id) else {
                    return Err(invalid("missing secured object"));
                };
                if Some(id) == self.mining.session.carried_id() {
                    return Err(invalid("secured object is carried"));
                }
                let Some(local) = self.mining.ship_fragments.get(&id) else {
                    return Err(invalid("secured object lacks ship frame"));
                };
                let mut check = self.mounts.clone();
                check.release(m.id);
                if !check.secure(
                    m.id,
                    id,
                    local.meters(),
                    crate::fragment_physics::sphere_radius(*f),
                ) {
                    return Err(invalid("secured object incompatible with physical mount"));
                }
                self.mining.secured.insert(id);
            }
        }
        Ok(())
    }
    pub(crate) fn mount_prompt(&self) -> Option<&'static str> {
        if self.mounts.carried.is_some() {
            return Some("Carrying shelf - aim at an empty habitation rail, G to attach");
        }
        self.aimed_mount()
            .map(|_| "[G] Place loose object [K] Secure/release [J] Move empty shelf")
    }
}

impl GameplaySession {
    fn move_accessory(&mut self) {
        use salimon_ship::attachments::{CarryingMount, HABITATION_RAILS};
        let s = self.snapshot();
        if s.character.location != CharacterLocation::InsideShip {
            self.feedback = Some("Attach accessory to a compatible cabin rail");
            return;
        }
        let frame = character_ship_frame(s.ship.pose);
        let eye = frame.world_to_local(s.character.eye_position_meters);
        let look = sub(frame.world_to_local(s.character.look_target_meters), eye);
        if let Some(c) = &self.mounts.carried {
            let candidate = HABITATION_RAILS
                .iter()
                .filter(|(_, p)| {
                    let delta = sub(*p, eye);
                    let distance = length(delta);
                    distance <= 2.
                        && dot(delta, look) / (distance * length(look)).max(1e-12) >= 0.97
                        && salimon_character::ship_sight_obstruction(eye, *p, true)
                            .is_none_or(|d| d >= distance - 0.02)
                })
                .min_by(|a, b| length(sub(a.1, eye)).total_cmp(&length(sub(b.1, eye))));
            let Some(&(rail, p)) = candidate else {
                self.feedback = Some("Aim at a compatible empty habitation rail");
                return;
            };
            let occupied = self.mining.session.fragments().iter().any(|f| {
                salimon_character::cargo::sphere_overlaps_box(
                    frame.world_to_local(f.transform().position().meters()),
                    crate::fragment_physics::sphere_radius(*f),
                    [
                        p[0] - 0.5,
                        p[0] + 0.5,
                        p[1] - 0.03,
                        p[1] + 0.03,
                        p[2] - 0.175,
                        p[2] + 0.175,
                    ],
                )
            });
            let capsule_overlaps = eye[0] + salimon_character::PLAYER_RADIUS_METERS > p[0] - 0.5
                && eye[0] - salimon_character::PLAYER_RADIUS_METERS < p[0] + 0.5
                && eye[2] + salimon_character::PLAYER_RADIUS_METERS > p[2] - 0.175
                && eye[2] - salimon_character::PLAYER_RADIUS_METERS < p[2] + 0.175
                && eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS < p[1] + 0.03;
            if occupied || capsule_overlaps {
                self.feedback = Some("Rail blocked by physical belongings");
                return;
            }
            let id = c.id;
            if !self.mining.session.release_fixture(id) {
                self.feedback = Some("Accessory carry identity mismatch");
                return;
            }
            let m = self.mounts.mounts.iter_mut().find(|m| m.id == id).unwrap();
            m.rail = rail;
            m.rail_origin = p;
            m.offset = [0.; 3];
            self.mounts.carried = None;
            self.sync_carried();
            self.feedback = Some("Existing accessory fastened; no fabrication material spent");
        } else {
            if self.aimed_mount() != Some(2001) {
                self.feedback = Some("Aim at the portable shelf; heavy restraint stays installed");
                return;
            }
            if self
                .mounts
                .mounts
                .iter()
                .any(|m| m.id == 2001 && m.secured.is_some())
                || self.mining.session.carried_object().is_some()
            {
                self.feedback = Some("Clear the shelf and carry slot first");
                return;
            }
            let m = self.mounts.mounts.iter().find(|m| m.id == 2001).unwrap();
            if self.mining.session.fragments().iter().any(|f| {
                let p = frame.world_to_local(f.transform().position().meters());
                let delta = sub(p, m.position());
                delta[0].abs() < 0.5 + crate::fragment_physics::sphere_radius(*f)
                    && delta[2].abs() < 0.175 + crate::fragment_physics::sphere_radius(*f)
                    && delta[1].abs() < 0.4
            }) {
                self.feedback = Some("Remove all loose belongings from shelf before moving it");
                return;
            }
            if !self.mining.session.carry_fixture(m.id) {
                return;
            }
            let p = frame.local_to_world(m.position());
            self.mounts.carried = Some(CarryingMount {
                id: m.id,
                world_position: p,
            });
            self.equipment.sync_carrying(true);
            self.mining.clear_input();
            self.sync_carried();
            self.feedback = Some("Empty shelf unfastened - aim at a compatible rail and press G");
        }
    }
    pub(super) fn sync_accessory(&mut self) {
        let s = self.snapshot();
        let frame = character_ship_frame(s.ship.pose);
        if let Some(c) = &mut self.mounts.carried {
            let eye = s.character.eye_position_meters;
            let direction = sub(s.character.look_target_meters, eye);
            let direction = direction.map(|v| v / length(direction).max(1e-12));
            c.world_position = std::array::from_fn(|i| {
                eye[i] + direction[i] * 0.8 - f64::from(s.character.up[i]) * 0.35
            });
            let local = frame.world_to_local(c.world_position);
            let absent = !salimon_character::ship_contains_object(local);
            self.mining
                .ship_mass
                .set_dry_equipment_absent(
                    c.id,
                    absent.then_some(salimon_ship::attachments::PORTABLE_SHELF_MASS_KG),
                )
                .expect("registered accessory mass");
        } else {
            self.mining
                .ship_mass
                .set_dry_equipment_absent(2001, None)
                .expect("registered accessory");
        }
        self.ship
            .set_total_mass_kg(self.mining.ship_mass.total_kg())
            .expect("valid accessory mass");
        self.mining.mount_geometry = self.mounts.clone();
        if let Some(c) = &self.mounts.carried {
            self.mining.mount_geometry.mounts.retain(|m| m.id != c.id);
        }
        self.sync_mount_collision();
    }
}

impl GameplaySession {
    pub(super) fn sync_mount_collision(&mut self) {
        let mut bounds: Vec<_> = self
            .mining
            .mount_geometry
            .mounts
            .iter()
            .map(|m| {
                let p = m.position();
                [
                    p[0] - 0.5,
                    p[0] + 0.5,
                    p[1] - 0.03,
                    p[1] + 0.03,
                    p[2] - 0.175,
                    p[2] + 0.175,
                ]
            })
            .collect();
        for id in &self.mining.secured {
            if let Some(f) = self.mining.session.fragment(*id)
                && let Some(local) = self.mining.ship_fragments.get(id)
            {
                let p = local.meters();
                let r = crate::fragment_physics::sphere_radius(*f);
                bounds.push([p[0] - r, p[0] + r, p[1] - r, p[1] + r, p[2] - r, p[2] + r]);
            }
        }
        let mut cabin = self.cabin_solids();
        cabin.extend(self.construction_solids());
        bounds.extend_from_slice(&cabin);
        if !self.mining.interior_solids.is_empty() && self.mining.interior_solids != cabin {
            self.mining.fragment_sleep = Default::default();
        }
        self.mining.interior_solids = cabin;
        self.character.set_interior_obstacles(bounds);
    }
}

/// Opt-in native evidence only: explicitly seeded 32 g sample, never a mining-route claim.
pub(crate) fn evidence_initial_conditions(s: &mut GameplaySession) -> Result<(), String> {
    s.mounts = salimon_ship::attachments::Attachments::starter();
    let eye = [-1.6, 1.9973, 1.8];
    let target = [-1.6, 1.15, 2.7];
    let d = sub(target, eye);
    let mut v = serde_json::to_value(s.character.save_state()).map_err(|e| e.to_string())?;
    v["position"] = serde_json::json!({"Inside":{"local":eye}});
    v["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
    v["pitch_radians"] = (d[1] / length(d)).asin().into();
    s.character = salimon_character::CharacterController::from_save(
        serde_json::from_value(v).map_err(|e| e.to_string())?,
        None,
    )
    .map_err(|e| e.to_string())?;
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let world = frame.local_to_world([-1.6, 1.5, 2.0]);
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(world[0], world[1], world[2]),
        [0., 0., 0., 1.],
    )
    .map_err(|e| format!("{e:?}"))?;
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 576000,
        },
        RawMaterial::new(ResourceId::IronOre, 0.032).map_err(|e| format!("{e:?}"))?,
        pose.position(),
        0.032,
    )
    .map_err(|e| format!("{e:?}"))?;
    let id = (0..400)
        .find_map(|_| {
            s.mining.session.tick_spherical(
                true,
                Some(&mut deposit),
                salimon_world::spherical_mining::SPHERE_TICK,
                Default::default(),
                pose,
            )
        })
        .ok_or("evidence sphere did not release")?;
    if !s.mining.session.pick_up(id) {
        return Err("evidence carry slot occupied".into());
    }
    s.equipment.sync_carrying(true);
    s.sync_carried();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Command;
    use salimon_world::resources::{FragmentId, ResourceId};
    use std::time::Duration;
    fn aim(s: &mut GameplaySession, eye: [f64; 3], target: [f64; 3]) {
        let d = sub(target, eye);
        let mut v = serde_json::to_value(s.character.save_state()).unwrap();
        v["position"] = serde_json::json!({"Inside":{"local":eye}});
        v["yaw_radians"] = d[2].atan2(d[0]).rem_euclid(std::f64::consts::TAU).into();
        v["pitch_radians"] = (d[1] / length(d)).asin().into();
        s.character = salimon_character::CharacterController::from_save(
            serde_json::from_value(v).unwrap(),
            None,
        )
        .unwrap();
    }
    fn piece(s: &mut GameplaySession, p: [f64; 3]) -> FragmentId {
        let frame = character_ship_frame(s.ship.snapshot().pose);
        let world = frame.local_to_world(p);
        let id = crate::sphere_fixtures::sphere(
            &mut s.mining.session,
            ResourceId::IronOre,
            0.032,
            ResourceTransform::new(
                salimon_world::WorldPosition::new(world[0], world[1], world[2]),
                [0., 0., 0., 1.],
            )
            .unwrap(),
        );
        s.mining
            .ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new(p));
        id
    }
    #[test]
    fn place_secure_blackout_restore_release_preserves_exact_object_and_mass() {
        let mut s = GameplaySession::new_run();
        aim(&mut s, [-1.6, 1.9973, 1.8], [-1.6, 1.15, 2.7]);
        assert_eq!(s.aimed_mount(), Some(2001));
        s.command(Command::Mount(MountAction::ToggleLock));
        assert_eq!(s.mount_audio.0, 0, "empty clamp rejection emits no sound");
        let id = piece(&mut s, [-1.6, 1.5, 2.0]);
        assert!(s.mining.session.pick_up(id));
        s.equipment.sync_carrying(true);
        let original = *s.mining.session.fragment(id).unwrap();
        s.command(Command::Mount(MountAction::Place));
        assert_eq!(s.mining.session.carried_id(), None, "{:?}", s.feedback);
        s.command(Command::Mount(MountAction::ToggleLock));
        assert!(s.mounts.is_secured(id), "{:?}", s.feedback);
        assert_eq!(s.mount_audio.0, 1, "only actual fastening emits a cue");
        let local = s.mining.ship_fragments[&id];
        let mass = s.ship.snapshot().total_mass_kg;
        s.ship.set_stored_core_energy_joules(0);
        s.advance_automation(Duration::from_secs(1));
        assert_eq!(s.mining.ship_fragments[&id], local);
        assert_eq!(
            s.mining.session.fragment(id).unwrap().material(),
            original.material()
        );
        assert_eq!(s.ship.snapshot().total_mass_kg, mass);
        let bytes = s.save_bytes(100).unwrap();
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&bytes).unwrap();
        assert_eq!(restored.mounts, s.mounts);
        assert_eq!(restored.mount_audio.0, 0, "load never replays a fastening");
        assert_eq!(restored.mining.secured, s.mining.secured);
        restored.command(Command::Mount(MountAction::ToggleLock));
        assert!(!restored.mounts.is_secured(id));
        assert!(restored.mining.session.fragment(id).is_some());
    }
    #[test]
    fn empty_accessory_uses_shared_slot_and_reattaches_without_resource_debit() {
        let mut s = GameplaySession::new_run();
        aim(&mut s, [-1.6, 1.9973, 1.8], [-1.6, 1.15, 2.7]);
        let before = serde_json::to_value(salimon_fabrication::FabricationSaveState::capture(
            &s.fabricator,
            &s.consumables,
        ))
        .ok();
        s.command(Command::Mount(MountAction::MoveAccessory));
        assert!(s.mounts.carried.is_some(), "{:?}", s.feedback);
        let id = piece(&mut s, [-1.6, 0.5, 1.0]);
        assert!(!s.mining.session.pick_up(id));
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert!(r.mining.session.carried_object().is_some());
        aim(&mut r, [-2.8, 1.9973, 1.8], [-2.8, 1.15, 2.7]);
        r.command(Command::Mount(MountAction::Place));
        assert!(r.mounts.carried.is_none(), "{:?}", r.feedback);
        assert_eq!(
            r.mounts.mounts.iter().find(|m| m.id == 2001).unwrap().rail,
            2020
        );
        assert!(r.mining.session.carried_object().is_none());
        assert_eq!(
            before,
            serde_json::to_value(salimon_fabrication::FabricationSaveState::capture(
                &r.fabricator,
                &r.consumables
            ))
            .ok()
        );
    }
    #[test]
    fn accessory_boundary_mass_restore_and_discrete_events_do_not_double_count() {
        let mut s = GameplaySession::new_run();
        aim(&mut s, [-1.6, 1.9973, 1.8], [-1.6, 1.15, 2.7]);
        s.command(Command::Mount(MountAction::MoveAccessory));
        let full = s.ship.snapshot().total_mass_kg;
        let frame = character_ship_frame(s.ship.snapshot().pose);
        let world = frame.local_to_world([-20., 2., 0.]);
        let mut v = serde_json::to_value(s.character.save_state()).unwrap();
        v["position"] = serde_json::json!({"Space":{"world":world}});
        s.character = salimon_character::CharacterController::from_save(
            serde_json::from_value(v).unwrap(),
            None,
        )
        .unwrap();
        s.sync_carried();
        assert_eq!(
            s.ship.snapshot().total_mass_kg,
            full - salimon_ship::attachments::PORTABLE_SHELF_MASS_KG
        );
        let before = s.checkpoint_events();
        s.command(Command::Look(10., 0.));
        s.sync_carried();
        assert_eq!(
            s.checkpoint_events(),
            before,
            "hand animation cannot trigger every-frame saves"
        );
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(
            r.ship.snapshot().total_mass_kg,
            s.ship.snapshot().total_mass_kg
        );
        assert_eq!(r.mounts, s.mounts);
    }
    #[test]
    fn corrupt_attachment_restore_is_atomic_and_legacy_save_adds_no_colliders() {
        let mut s = GameplaySession::new_run();
        let good = s.save_bytes(100).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&good).unwrap();
        v["state"]["mounts"]["mounts"][0]["secured"] = 999.into();
        assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        assert_eq!(s.save_bytes(100).unwrap(), good);
        v["schema_version"] = 10.into();
        v["state"].as_object_mut().unwrap().remove("mounts");
        v["state"]["habitation"]
            .as_object_mut()
            .unwrap()
            .remove("cabin");
        s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert!(s.mounts.mounts.is_empty());
    }
}
