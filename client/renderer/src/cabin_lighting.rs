//! Bounded presentation-only room fill and five local fixture contributions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CabinLighting {
    /// Room fill including actual exterior window contribution, cabin then cargo.
    pub ambient: [[f32; 4]; 2],
    /// Ship-local XYZ and inverse-square range coefficient.
    pub positions: [[f32; 4]; 6],
    /// Linear RGB; W is room index (0 cabin, 1 cargo).
    pub colors: [[f32; 4]; 6],
}
impl Default for CabinLighting {
    fn default() -> Self {
        Self {
            ambient: [[0.1, 0.14, 0.2, 0.]; 2],
            positions: [[0.; 4]; 6],
            colors: [[0.; 4]; 6],
        }
    }
}
impl CabinLighting {
    pub fn sample(self, p: [f64; 3]) -> [f32; 3] {
        let room = usize::from(p[0] < -4.65);
        let mut out = [
            self.ambient[room][0],
            self.ambient[room][1],
            self.ambient[room][2],
        ];
        for (pos, color) in self.positions.into_iter().zip(self.colors) {
            if color[3] as usize != room {
                continue;
            }
            let d = (0..3)
                .map(|i| (p[i] - f64::from(pos[i])).powi(2))
                .sum::<f64>() as f32;
            for i in 0..3 {
                out[i] += color[i] / (1. + d * pos[3]);
            }
        }
        out.map(|v| v.clamp(0., 2.))
    }
    pub(crate) fn values(self) -> Result<Vec<f32>, crate::RendererError> {
        let values: Vec<_> = self
            .ambient
            .into_iter()
            .chain(self.positions)
            .chain(self.colors)
            .flatten()
            .collect();
        if values.iter().any(|v| !v.is_finite() || v.abs() > 100.) {
            return Err(crate::RendererError::new(
                "cabin lighting",
                "invalid bounded light DTO",
            ));
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_light_falloff_room_isolation_and_invalid_dtos() {
        let mut l = CabinLighting::default();
        l.positions[0] = [0., 1., 0., 1.];
        l.colors[0] = [0.5, 0.3, 0.2, 0.];
        let near = l.sample([0., 1., 0.]);
        let far = l.sample([4., 1., 0.]);
        assert!(near[0] > far[0]);
        assert_eq!(l.sample([-6., 1., 0.]), [0.1, 0.14, 0.2]);
        l.colors[0][0] = f32::NAN;
        assert!(l.values().is_err());
    }
}
