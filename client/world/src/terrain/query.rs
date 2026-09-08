use super::{LayerKind, TerrainDefinition, TerrainError};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SurfaceSample {
    pub elevation_meters: f64,
    pub body_local_position_meters: [f64; 3],
    pub outward_normal: [f64; 3],
    /// Tangent magnitude of the height gradient divided by radial magnitude.
    pub slope: f64,
}
impl TerrainDefinition {
    /// Reject non-unit input rather than quietly changing cache/query coordinates.
    pub fn height(&self, direction: [f64; 3]) -> Result<f64, TerrainError> {
        if !direction.iter().all(|v| v.is_finite()) || (norm(direction) - 1.0).abs() > 1e-12 {
            return Err(TerrainError::InvalidDirection);
        }
        Ok(self.evaluate(direction))
    }
    /// Normal uses a fixed 0.25 m central-difference stencil in body-local f64.
    pub fn sample(&self, direction: [f64; 3]) -> Result<SurfaceSample, TerrainError> {
        let height = self.height(direction)?;
        let axis = if direction[0].abs() < 0.8 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let u = unit(cross(direction, axis));
        let v = cross(direction, u);
        let epsilon = 0.25 / self.radius_meters;
        let gradient = [u, v].map(|t| {
            let plus = unit(std::array::from_fn(|i| direction[i] + epsilon * t[i]));
            let minus = unit(std::array::from_fn(|i| direction[i] - epsilon * t[i]));
            (self.evaluate(plus) - self.evaluate(minus)) / 0.5
        });
        let radial = 1.0 + height / self.radius_meters;
        let normal = unit(std::array::from_fn(|i| {
            radial * direction[i] - gradient[0] * u[i] - gradient[1] * v[i]
        }));
        Ok(SurfaceSample {
            elevation_meters: height,
            body_local_position_meters: direction.map(|x| x * (self.radius_meters + height)),
            outward_normal: normal,
            slope: gradient[0].hypot(gradient[1]) / radial,
        })
    }
    fn evaluate(&self, direction: [f64; 3]) -> f64 {
        if let Some(start) = &self.canonical {
            return start
                .height_meters(direction)
                .expect("validated finite unit direction");
        }
        let mut height = 0.0;
        for (index, layer) in self.layers.iter().enumerate() {
            let point = direction.map(|x| x * (self.radius_meters / layer.wavelength_meters));
            let noise = noise(self.seed.wrapping_add(index as u64), point);
            let value = match layer.kind {
                LayerKind::Hills => noise,
                LayerKind::Ridges => 1.0 - 2.0 * noise.abs(),
            };
            height += layer.amplitude_meters * value;
        }
        for feature in &self.features {
            let distance = norm(std::array::from_fn(|i| direction[i] - feature.direction[i]))
                * self.radius_meters;
            let t = (distance / feature.radius_meters).min(1.0);
            let weight = 1.0 - t * t * (3.0 - 2.0 * t);
            height += feature.elevation_meters * weight;
        }
        height
    }
}
pub(super) fn norm(v: [f64; 3]) -> f64 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = norm(v);
    v.map(|x| x / n)
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
// Fixed integer mixing + trilinear smooth value noise. No trigonometry, mutable
// RNG, face coordinates, platform hasher, world origin or time enters evaluation.
fn noise(seed: u64, p: [f64; 3]) -> f64 {
    let cell = p.map(|x| x.floor() as i64);
    let t = std::array::from_fn::<_, 3, _>(|i| {
        let x = p[i] - cell[i] as f64;
        x * x * (3.0 - 2.0 * x)
    });
    let mut result = 0.0;
    for corner in 0..8 {
        let mut hash = seed;
        let mut weight = 1.0;
        for axis in 0..3 {
            let bit = (corner >> axis) & 1;
            weight *= if bit == 0 { 1.0 - t[axis] } else { t[axis] };
            for byte in (cell[axis] + bit).to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        hash = (hash ^ (hash >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        hash = (hash ^ (hash >> 27)).wrapping_mul(0x94d049bb133111eb);
        let value = ((hash ^ (hash >> 31)) >> 11) as f64 / (1_u64 << 53) as f64;
        result += weight * (value * 2.0 - 1.0);
    }
    result
}
