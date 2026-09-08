//! Completed internal doorway geometry and manual occupancy-gated control.
use super::*;

impl GameplaySession {
    pub(super) fn toggle_completed_door(&mut self, id: PlaceholderId) {
        let p = self
            .building
            .source
            .projects
            .inspect(id)
            .expect("aimed completed door");
        if self.building.source.open_doors.contains(&id) {
            let snap = self.snapshot();
            let f = character_ship_frame(snap.ship.pose);
            let eye = f.world_to_local(snap.character.eye_position_meters);
            let b = *door_solids(p, false).last().expect("closed door leaf");
            let r = salimon_character::PLAYER_RADIUS_METERS;
            let foot = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
            let occupied = eye[0] + r > b[0]
                && eye[0] - r < b[1]
                && eye[2] + r > b[4]
                && eye[2] - r < b[5]
                && foot < b[3]
                && foot + salimon_character::PLAYER_BODY_HEIGHT_METERS > b[2];
            if occupied
                || self.mining.session.fragments().iter().any(|p| {
                    salimon_character::cargo::sphere_overlaps_box(
                        f.world_to_local(p.transform().position().meters()),
                        crate::fragment_physics::sphere_radius(*p),
                        b,
                    )
                })
            {
                self.feedback = Some("Door obstructed; clear player and belongings");
                return;
            }
            self.building.source.open_doors.retain(|v| *v != id);
            self.feedback = Some("Internal door closed");
        } else {
            self.building.source.open_doors.push(id);
            self.building.source.open_doors.sort();
            self.feedback = Some("Internal door opened");
        }
        self.sync_mount_collision();
    }
}

pub(super) fn door_solids(p: &Placeholder, open: bool) -> Vec<[f64; 6]> {
    let s = p.space;
    let axis = if s.max[0] - s.min[0] < s.max[2] - s.min[2] {
        0
    } else {
        2
    };
    let along = 2 - axis;
    let mid = (s.min[along] + s.max[along]) / 2.;
    let base = [s.min[0], s.max[0], s.min[1], s.max[1], s.min[2], s.max[2]];
    let mut left = base;
    left[along * 2 + 1] = mid - 0.7;
    let mut right = base;
    right[along * 2] = mid + 0.7;
    let mut lintel = base;
    lintel[2] = s.max[1] - 0.1;
    lintel[along * 2] = mid - 0.7;
    lintel[along * 2 + 1] = mid + 0.7;
    let mut out = vec![left, right, lintel];
    if !open {
        let mut leaf = base;
        leaf[axis * 2] += 0.02;
        leaf[axis * 2 + 1] -= 0.02;
        leaf[along * 2] = mid - 0.7;
        leaf[along * 2 + 1] = mid + 0.7;
        leaf[3] -= 0.1;
        out.push(leaf);
    }
    out
}
