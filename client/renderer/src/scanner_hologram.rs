//! Bounded ship-local holographic projection geometry; no discovery authority.
use crate::SceneInstance;
pub const HOLOGRAM_CENTER: [f64; 3] = [5.2, 1.75, -0.9];
pub const HOLOGRAM_RADIUS: f64 = 0.30;
pub const MAX_HOLOGRAM_CONTACTS: usize = 64;
#[derive(Clone, Copy, Debug)]
pub struct HologramContact {
    pub relative_ship_meters: [f64; 3],
    pub selected: bool,
}
/// Depth-tested low-poly projector and three orthogonal dotted great circles.
/// The caller maps these ship-local instances through the current ship frame.
/// Radial distance is logarithmic; angles/depth retain the full 3D direction.
pub fn scanner_hologram(
    contacts: &[HologramContact],
    powered: bool,
    range_meters: f64,
) -> Vec<SceneInstance> {
    let mut out = Vec::with_capacity(104 + MAX_HOLOGRAM_CONTACTS * 2);
    let mut cube = |center, half, color| {
        out.push(SceneInstance {
            center_meters: center,
            half_extents_meters: half,
            color,
            orientation: [0., 0., 0., 1.],
        })
    };
    cube(
        [5.2, 1.285, -0.9],
        [0.34, 0.015, 0.34],
        [0.08, 0.12, 0.15, 1.],
    );
    cube(
        [5.2, 1.315, -0.9],
        [0.08, 0.015, 0.08],
        if powered {
            [0.1, 0.9, 1., 1.]
        } else {
            [0.16, 0.18, 0.20, 1.]
        },
    );
    if !powered || !range_meters.is_finite() || range_meters <= 0. {
        return out;
    }
    for plane in 0..3 {
        for step in 0..32 {
            let angle = std::f64::consts::TAU * f64::from(step) / 32.;
            let mut p = HOLOGRAM_CENTER;
            p[(plane + 1) % 3] += HOLOGRAM_RADIUS * angle.cos();
            p[(plane + 2) % 3] += HOLOGRAM_RADIUS * angle.sin();
            cube(p, [0.004; 3], [0.04, 0.35, 0.48, 1.]);
        }
    }
    cube(HOLOGRAM_CENTER, [0.012; 3], [0.9, 0.95, 1., 1.]);
    for c in contacts
        .iter()
        .filter(|c| c.selected)
        .chain(contacts.iter().filter(|c| !c.selected))
        .take(MAX_HOLOGRAM_CONTACTS)
    {
        let v = c.relative_ship_meters;
        let distance = v[0].hypot(v[1]).hypot(v[2]);
        if !distance.is_finite() || distance > range_meters {
            continue;
        }
        let radius = HOLOGRAM_RADIUS * distance.ln_1p() / range_meters.ln_1p();
        let p = std::array::from_fn(|i| {
            HOLOGRAM_CENTER[i]
                + if distance > 0. {
                    v[i] / distance * radius
                } else {
                    0.
                }
        });
        cube(
            p,
            [if c.selected { 0.020 } else { 0.012 }; 3],
            if c.selected {
                [1., 0.65, 0.10, 1.]
            } else {
                [0.10, 0.90, 1., 1.]
            },
        );
        // Elevation cue joins each contact to the horizontal projection plane.
        if (p[1] - HOLOGRAM_CENTER[1]).abs() > 0.004 {
            cube(
                [p[0], (p[1] + HOLOGRAM_CENTER[1]) * 0.5, p[2]],
                [
                    0.002,
                    ((p[1] - HOLOGRAM_CENTER[1]).abs() * 0.5) as f32,
                    0.002,
                ],
                [0.10, 0.40, 0.50, 1.],
            );
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_projection_depth_bounds_blackout_and_invalid_inputs() {
        let contacts = [
            HologramContact {
                relative_ship_meters: [0., 100., 0.],
                selected: true,
            },
            HologramContact {
                relative_ship_meters: [0., -100., 0.],
                selected: false,
            },
        ];
        let v = scanner_hologram(&contacts, true, 100.);
        assert!(v.iter().any(
            |p| (p.center_meters[1] - HOLOGRAM_CENTER[1] - HOLOGRAM_RADIUS).abs() < 1e-12
                && p.half_extents_meters[0] == 0.020
        ));
        assert!(v.iter().any(
            |p| (p.center_meters[1] - HOLOGRAM_CENTER[1] + HOLOGRAM_RADIUS).abs() < 1e-12
                && p.half_extents_meters[0] == 0.012
        ));
        assert_eq!(scanner_hologram(&contacts, false, 100.).len(), 2);
        assert_eq!(scanner_hologram(&contacts, true, f64::NAN).len(), 2);
        assert!(scanner_hologram(&vec![contacts[0]; 1000], true, 100.).len() <= 99 + 128);
    }
}
