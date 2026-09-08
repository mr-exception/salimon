//! Outer boarding geometry, gate target and source-derived exit placement.
use super::*;

pub(in crate::session) fn boarding_exit(structure: &ShipStructure) -> Option<[f64; 4]> {
    let cell = structure
        .occupied_cells()
        .filter(|(c, id)| id.0 >= 1024 && c.y == 0 && c.z == 1)
        .map(|(c, _)| c)
        .min_by_key(|c| c.x)?;
    let start = cell.bounds(ORIGIN).ok()?.min[0];
    Some([
        start - 4.8,
        start,
        salimon_character::boarding_ramp_tip_meters()[1],
        SHIP_FLOOR_HEIGHT_METERS + 0.1,
    ])
}
pub(super) fn boarding_solids(structure: &ShipStructure, open: bool) -> Vec<[f64; 6]> {
    let Some([tip, start, bottom, top]) = boarding_exit(structure) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for i in 0..20 {
        let x0 = tip + (start - tip) * f64::from(i) / 20.;
        let x1 = tip + (start - tip) * f64::from(i + 1) / 20.;
        let y = bottom + (top - bottom) * f64::from(i + 1) / 20.;
        out.push([x0, x1, y - 0.12, y, -0.7, 0.7]);
        for z in [-0.7, 0.65] {
            out.push([x0, x1, y + 1., y + 1.05, z, z + 0.05]);
            if i % 4 == 0 {
                out.push([x0, x0 + 0.05, y, y + 1., z, z + 0.05]);
            }
        }
    }
    out.extend([
        [start - 0.05, start, top + 1., top + 1.2, 0.9, 1.0],
        [start + 0.1, start + 0.15, top + 1., top + 1.2, 0.9, 1.0],
    ]);
    if !open {
        out.push([
            start + 0.02,
            start + 0.08,
            SHIP_FLOOR_HEIGHT_METERS,
            SHIP_FLOOR_HEIGHT_METERS + 2.3,
            -0.7,
            0.7,
        ]);
    }
    out
}
impl GameplaySession {
    pub(in crate::session) fn modular_gate_clear(&self) -> bool {
        let Some([_, x, _, y]) = boarding_exit(&self.building.structure) else {
            return true;
        };
        let snap = self.snapshot();
        let f = character_ship_frame(snap.ship.pose);
        let eye = f.world_to_local(snap.character.eye_position_meters);
        let b = [x - 0.1, x + 0.15, y - 0.1, y + 2.3, -0.7, 0.7];
        let r = salimon_character::PLAYER_RADIUS_METERS;
        let foot = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
        !(eye[0] + r > b[0]
            && eye[0] - r < b[1]
            && eye[2] + r > b[4]
            && eye[2] - r < b[5]
            && foot < b[3]
            && foot + salimon_character::PLAYER_BODY_HEIGHT_METERS > b[2])
            && self.mining.session.fragments().iter().all(|p| {
                !salimon_character::cargo::sphere_overlaps_box(
                    f.world_to_local(p.transform().position().meters()),
                    crate::fragment_physics::sphere_radius(*p),
                    b,
                )
            })
    }
    pub(in crate::session) fn modular_gate_target(&self, eye: [f64; 3], look: [f64; 3]) -> bool {
        let Some([_, x, _, y]) = boarding_exit(&self.building.structure) else {
            return false;
        };
        let target = [if eye[0] < x { x - 0.01 } else { x + 0.11 }, y + 1.1, 0.95];
        let d = salimon_math::sub(target, eye);
        let ray = salimon_math::sub(look, eye);
        let distance = salimon_math::length(d);
        distance < 2.5
            && salimon_math::dot(d, ray) / (distance * salimon_math::length(ray)).max(1e-9) > 0.96
            && self.construction_solids().iter().all(|b| {
                salimon_character::cargo::box_sight_obstruction(eye, target, *b)
                    .is_none_or(|hit| hit + 0.05 >= distance)
            })
    }
}
