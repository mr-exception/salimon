//! Completed structural source, stable cell/face identities and shared geometry.
use super::boarding::{boarding_exit, boarding_solids};
use super::*;
use salimon_ship::structure::{Direction, FaceKind, shell::*};

pub(super) fn structural(p: &Placeholder) -> bool {
    matches!(
        p.blueprint.id.as_str(),
        "hull-cell" | "deck-cell" | "two-deck-room"
    )
}
impl ConstructionSave {
    pub(super) fn structure(
        &self,
        completing: Option<PlaceholderId>,
    ) -> Result<ShipStructure, salimon_ship::structure::StructureError> {
        let rooms: Vec<_> = self
            .projects
            .jobs()
            .filter(|p| {
                !p.work.destroyed && structural(p) && (p.work.completed || Some(p.id) == completing)
            })
            .map(|p| ShellRoom {
                project: p.id.0,
                cells: p.cells.clone(),
                cell_milligrams: (0..p.cells.len())
                    .map(|i| {
                        p.blueprint
                            .requirements
                            .iter()
                            .map(|r| {
                                let total = p.funding.delivered(r.resource);
                                let n = p.cells.len() as u64;
                                total / n + u64::from((i as u64) < total % n)
                            })
                            .sum()
                    })
                    .collect(),
            })
            .collect();
        build_shells_with_losses(
            &starter_structure(),
            &rooms,
            &self.removed_partitions,
            &self.damage.lost,
        )
    }
}
impl GameplaySession {
    pub(super) fn installation_clear(&self, next: &ShipStructure) -> bool {
        let snap = self.snapshot();
        let frame = character_ship_frame(snap.ship.pose);
        let eye = frame.world_to_local(snap.character.eye_position_meters);
        let r = salimon_character::PLAYER_RADIUS_METERS;
        let foot = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
        if let Some([tip, _, bottom, _]) = boarding_exit(next) {
            let local = frame.world_to_local(self.terrain_frame(snap.ship).eye_on_ground(
                frame.local_to_world([
                    tip,
                    bottom + salimon_character::PLAYER_EYE_HEIGHT_METERS,
                    0.,
                ]),
            ));
            if (local[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS - bottom).abs() > 0.3 {
                return false;
            }
        }
        if boarding_exit(next) != boarding_exit(&self.building.structure)
            && let Some([tip, start, bottom, top]) = boarding_exit(&self.building.structure)
        {
            if eye[0] >= tip - 0.3
                && eye[0] <= start + 0.3
                && eye[2].abs() < 1.
                && eye[1] >= bottom
                && foot <= top + 2.
            {
                return false;
            }
            if self.mining.session.fragments().iter().any(|f| {
                let p = frame.world_to_local(f.transform().position().meters());
                p[0] >= tip - 0.3
                    && p[0] <= start + 0.3
                    && p[2].abs() < 1.
                    && p[1] >= bottom
                    && p[1] <= top + 2.
            }) {
                return false;
            }
        }
        let old = self.structural_solids();
        let boxes = next.parts().filter(|p| p.id.0 >= 1024).flat_map(|p| {
            p.collision
                .iter()
                .map(|b| [b.min[0], b.max[0], b.min[1], b.max[1], b.min[2], b.max[2]])
        });
        // Include the connector in the final clearance transaction, not only the
        // reserved room volume. A body on the old ramp must never be trapped.
        let y = SHIP_FLOOR_HEIGHT_METERS;
        let connector = [
            [-8.5, -7.04, y, y + 0.1, -1.4, 1.4],
            [-8.5, -7.04, y + 2.3, y + 2.4, -1.4, 1.4],
            [-8.5, -7.04, y, y + 2.4, -1.4, -1.3],
            [-8.5, -7.04, y, y + 2.4, 1.3, 1.4],
        ];
        boxes
            .chain(connector)
            .chain(boarding_solids(
                next,
                crate::session::door_passable(snap.ship),
            ))
            .filter(|b| !old.contains(b))
            .all(|b| {
                let player = eye[0] + r > b[0]
                    && eye[0] - r < b[1]
                    && eye[2] + r > b[4]
                    && eye[2] - r < b[5]
                    && foot < b[3]
                    && foot + salimon_character::PLAYER_BODY_HEIGHT_METERS > b[2];
                !player
                    && self.mining.session.fragments().iter().all(|f| {
                        !salimon_character::cargo::sphere_overlaps_box(
                            frame.world_to_local(f.transform().position().meters()),
                            crate::fragment_physics::sphere_radius(*f),
                            b,
                        )
                    })
            })
    }
    pub(super) fn structural_support(&self, space: Envelope) -> bool {
        // The first attachment uses the existing aft cargo aperture. Other
        // attachments require an actual completed neighboring volume, never
        // a support-only starter grid cell behind an uncut authored wall.
        let eps = 1e-7;
        let floor = SHIP_FLOOR_HEIGHT_METERS;
        ((space.max[0] + 8.5).abs() < eps
            && space.min[1] <= floor + eps
            && space.max[1] > floor + 2.
            && space.min[2] <= -1.5
            && space.max[2] >= 1.5)
            || self.building.source.projects.jobs().any(|p| {
                p.work.completed
                    && !p.work.destroyed
                    && structural(p)
                    && (0..3).filter(|axis| *axis != 1).any(|axis| {
                        let other = if axis == 0 { 2 } else { 0 };
                        ((space.min[axis] - p.space.max[axis]).abs() < eps
                            || (space.max[axis] - p.space.min[axis]).abs() < eps)
                            && space.min[other] < p.space.max[other]
                            && space.max[other] > p.space.min[other]
                            && (space.min[1] - p.space.min[1]).abs() < eps
                    })
            })
    }
    pub(in crate::session) fn structural_solids(&self) -> Vec<[f64; 6]> {
        let mut out: Vec<_> = self
            .building
            .structure
            .parts()
            .filter(|p| p.id.0 >= 1024)
            .flat_map(|p| {
                p.collision
                    .iter()
                    .map(|b| [b.min[0], b.max[0], b.min[1], b.max[1], b.min[2], b.max[2]])
            })
            .collect();
        if self.modular_port() {
            let y = SHIP_FLOOR_HEIGHT_METERS;
            out.extend([
                [-8.5, -7.04, y, y + 0.1, -1.4, 1.4],
                [-8.5, -7.04, y + 2.3, y + 2.4, -1.4, 1.4],
                [-8.5, -7.04, y, y + 2.4, -1.4, -1.3],
                [-8.5, -7.04, y, y + 2.4, 1.3, 1.4],
            ]);
        }
        out.extend(boarding_solids(
            &self.building.structure,
            crate::session::door_passable(self.ship.snapshot()),
        ));
        out
    }
    pub(in crate::session) fn modular_port(&self) -> bool {
        self.building
            .structure
            .occupied_cells()
            .any(|(c, id)| c == Cell::new(-3, 0, 1) && id.0 >= 1024)
    }
    pub(in crate::session) fn structural_volumes(&self) -> Vec<[f64; 6]> {
        let mut volumes: Vec<_> = self
            .building
            .structure
            .parts()
            .filter(|p| p.id.0 >= 1024)
            .map(|p| {
                let b = p.space;
                [b.min[0], b.max[0], b.min[1], b.max[1], b.min[2], b.max[2]]
            })
            .collect();
        if self.modular_port() {
            let y = SHIP_FLOOR_HEIGHT_METERS;
            volumes.push([-8.5, -6.8, y, y + 2.4, -1.4, 1.4]);
        }
        if let Some([tip, start, bottom, top]) = boarding_exit(&self.building.structure) {
            volumes.push([tip - 0.5, start + 0.1, bottom, top + 2.4, -0.7, 0.7]);
        }
        volumes
    }
    /// Builder Delete removes an aimed ordinary partition; exterior and deck
    /// boundaries, stair supports and the authored Scout hull cannot be removed.
    pub(super) fn remove_partition(&mut self) {
        let snap = self.snapshot();
        let f = character_ship_frame(snap.ship.pose);
        let eye = f.world_to_local(snap.character.eye_position_meters);
        let ray_direction =
            salimon_math::sub(f.world_to_local(snap.character.look_target_meters), eye);
        let candidate = self
            .building
            .structure
            .parts()
            .flat_map(|p| p.faces.iter().map(move |face| (p.id, face)))
            .filter(|(_, face)| {
                face.kind == FaceKind::Doorway
                    && !matches!(face.direction, Direction::Up | Direction::Down)
            })
            .filter_map(|(id, face)| {
                let neighbor = face.cell.neighbor(face.direction)?;
                if !self
                    .building
                    .structure
                    .occupied_cells()
                    .any(|(c, part)| c == neighbor && part.0 >= 1024)
                {
                    return None;
                }
                let mut b = face.cell.bounds(ORIGIN).ok()?;
                let axis = if matches!(face.direction, Direction::Forward | Direction::Aft) {
                    0
                } else {
                    2
                };
                if matches!(face.direction, Direction::Forward | Direction::Port) {
                    b.min[axis] = b.max[axis] - 0.1;
                } else {
                    b.max[axis] = b.min[axis] + 0.1;
                }
                ray_distance(eye, ray_direction, b).map(|d| (id, face.direction, d))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2));
        let Some((part, direction, distance)) = candidate else {
            self.feedback = Some("Aim at an internal partition within reach");
            return;
        };
        let target = salimon_math::add(
            eye,
            salimon_math::scale(
                ray_direction,
                distance / salimon_math::length(ray_direction),
            ),
        );
        let occluded = salimon_character::ship_sight_obstruction(
            eye,
            target,
            crate::session::door_passable(snap.ship),
        )
        .is_some_and(|hit| hit + 0.02 < distance);
        if distance > REACH_METERS
            || occluded
            || self.construction_solids().iter().any(|b| {
                ray_distance(eye, ray_direction, envelope(*b))
                    .is_some_and(|hit| hit + 0.02 < distance)
            })
        {
            return;
        }
        let edit = PartitionEdit { part, direction };
        let mut next = self.building.source.clone();
        next.removed_partitions.push(edit);
        match next.structure(None) {
            Ok(structure) => {
                self.building.source = next;
                self.building.structure = structure;
                self.sync_mount_collision();
                self.feedback = Some("Internal partition removed");
            }
            Err(_) => self.feedback = Some("Partition removal is unsafe"),
        }
    }
}

pub(super) fn supported_by(structure: &ShipStructure, s: Envelope) -> bool {
    let parts: Vec<_> = structure.parts().filter(|p| p.id.0 >= 1024).collect();
    [s.min[0] + 0.001, s.max[0] - 0.001].into_iter().all(|x| {
        [s.min[2] + 0.001, s.max[2] - 0.001].into_iter().all(|z| {
            parts.iter().flat_map(|p| &p.collision).any(|b| {
                x >= b.min[0]
                    && x <= b.max[0]
                    && z >= b.min[2]
                    && z <= b.max[2]
                    && (b.max[1] - s.min[1]).abs() < 1e-6
            })
        })
    })
}
