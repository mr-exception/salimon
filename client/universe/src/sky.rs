//! Read-only presentation projection. Visibility conveys no discovery/navigation rights.
use crate::{
    Catalog, CatalogId, Error, Record, coordinates::GalacticPosition, epoch::Epoch, invalid,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StarSource {
    pub id: CatalogId,
    pub position: GalacticPosition,
    pub epoch: Epoch,
    /// Missing motion remains unknown: hold reference position, do not invent velocity.
    pub velocity_mps: Option<[f64; 3]>,
    pub radius_meters: f64,
    pub temperature_kelvin: Option<f64>,
    pub luminosity_solar: Option<f64>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub direction: [f64; 3],
    pub angular_radius_radians: f64,
    pub color: [f32; 3],
    pub brightness: f32,
}
impl StarSource {
    pub fn validate(&self, catalog: &Catalog) -> Result<(), Error> {
        if !matches!(catalog.get(&self.id), Some(Record::Star(_)))
            || !self.radius_meters.is_finite()
            || self.radius_meters <= 0.
            || self
                .velocity_mps
                .is_some_and(|v| !v.iter().all(|x| x.is_finite()))
            || self
                .temperature_kelvin
                .is_some_and(|x| !x.is_finite() || !(1000.0..=100000.0).contains(&x))
            || self
                .luminosity_solar
                .is_some_and(|x| !x.is_finite() || x <= 0.)
        {
            return Err(invalid("sky source identity/physical properties"));
        }
        Ok(())
    }
    pub fn project(
        &self,
        observer: GalacticPosition,
        epoch: Epoch,
    ) -> Result<Option<Projection>, Error> {
        let dt = epoch.seconds_since(self.epoch)?;
        let p = if let Some(v) = self.velocity_mps {
            self.position.translated(v.map(|x| x * dt))?
        } else {
            self.position
        };
        let relative = p.relative_meters(observer)?;
        let distance = relative[0].hypot(relative[1]).hypot(relative[2]);
        if !distance.is_finite()
            || distance <= self.radius_meters
            || self.radius_meters <= 0.
            || !self.radius_meters.is_finite()
        {
            return Err(invalid("sky observer inside or invalid star"));
        }
        // Approximate inverse-square visual exposure, explicitly not calibrated photometry.
        let flux = self.luminosity_solar.unwrap_or(1.) * (3.085677581491367e16 / distance).powi(2);
        if flux < 0.0001 {
            return Ok(None);
        }
        let t = self.temperature_kelvin.unwrap_or(5772.);
        let color = if t < 4500. {
            [1., 0.62, 0.37]
        } else if t > 7500. {
            [0.65, 0.78, 1.]
        } else {
            [1., 0.94, 0.82]
        };
        Ok(Some(Projection {
            direction: relative.map(|x| x / distance),
            angular_radius_radians: (self.radius_meters / distance).asin(),
            color,
            brightness: (flux.sqrt() * 0.3).clamp(0.02, 100.) as f32,
        }))
    }
}
pub fn decode(bytes: &[u8], catalog: &Catalog) -> Result<Vec<StarSource>, Error> {
    if bytes.len() > 256 * 1024 {
        return Err(invalid("sky sidecar byte bound"));
    }
    let stars: Vec<StarSource> =
        serde_json::from_slice(bytes).map_err(|e| crate::Error::Encoding(e.to_string()))?;
    if stars.len() > 128 {
        return Err(invalid("sky source count bound"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for star in &stars {
        star.validate(catalog)?;
        if !ids.insert(&star.id) {
            return Err(invalid("duplicate sky identity"));
        }
    }
    Ok(stars)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sky_split_origin_motion_distance_and_unknown_velocity() {
        let origin = GalacticPosition::new([230000000, 0, 0], [0.; 3]).unwrap();
        let epoch = Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], crate::TimeScale::Tt).unwrap();
        let mut s = StarSource {
            id: CatalogId::from_source("test", "star").unwrap(),
            position: origin.translated([1e12, 0., 0.]).unwrap(),
            epoch,
            velocity_mps: Some([0., 1e8, 0.]),
            radius_meters: 7e8,
            temperature_kelvin: Some(3000.),
            luminosity_solar: Some(1.),
        };
        let p = s.project(origin, epoch).unwrap().unwrap();
        assert_eq!(p.direction, [1., 0., 0.]);
        assert!((p.angular_radius_radians - 0.0007000000571666793).abs() < 1e-12);
        let later = epoch.advance_nanos(1_000_000_000_000).unwrap();
        assert!(s.project(origin, later).unwrap().unwrap().direction[1] > 0.09);
        s.velocity_mps = None;
        assert_eq!(p, s.project(origin, later).unwrap().unwrap());
        assert!(s.project(s.position, epoch).is_err());
    }
}
