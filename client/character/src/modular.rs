//! Host-supplied completed ship volumes and matching solid boxes. No build authority.
#[derive(Clone, Debug, Default)]
pub struct ModularInterior {
    pub volumes: Vec<[f64; 6]>,
    pub solids: Vec<[f64; 6]>,
    pub aft_port: bool,
}
impl ModularInterior {
    pub fn contains(&self, p: [f64; 3]) -> bool {
        self.volumes
            .iter()
            .any(|b| (0..3).all(|i| p[i] >= b[i * 2] - 1e-7 && p[i] <= b[i * 2 + 1] + 1e-7))
    }
    pub fn allows_sphere(&self, p: [f64; 3], r: f64) -> bool {
        r.is_finite()
            && r > 0.
            && p.iter().all(|v| v.is_finite())
            && [-r, r].into_iter().all(|x| {
                [-r, r].into_iter().all(|y| {
                    [-r, r]
                        .into_iter()
                        .all(|z| self.contains([p[0] + x, p[1] + y, p[2] + z]))
                })
            })
            && self
                .solids
                .iter()
                .all(|b| !crate::cargo::sphere_overlaps_box(p, (r - 1e-5).max(0.), *b))
    }
    pub fn support(&self, p: [f64; 3], r: f64, max_height: f64) -> Option<f64> {
        self.solids
            .iter()
            .filter(|b| {
                p[0] + r > b[0] + 1e-7
                    && p[0] - r < b[1] - 1e-7
                    && p[2] + r > b[4] + 1e-7
                    && p[2] - r < b[5] - 1e-7
                    && b[3] <= max_height + 1e-7
            })
            .map(|b| b[3])
            .max_by(f64::total_cmp)
    }
    pub fn sight(&self, eye: [f64; 3], target: [f64; 3]) -> Option<f64> {
        self.solids
            .iter()
            .filter_map(|b| crate::cargo::box_sight_obstruction(eye, target, *b))
            .min_by(f64::total_cmp)
    }
}
