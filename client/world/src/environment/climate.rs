//! Bounded stylized climate, not CFD or a scientific weather forecast.
use super::{EnvironmentError as Error, random, seconds};
use crate::{CelestialBody, terrain::geology::GeologyDefinition};
use salimon_universe::{Epoch, synthesis::sha256};
use std::f64::consts::TAU;
pub const CLIMATE_VERSION: u16 = 1;

/// Explicit atmospheric chemistry and available condensable; never inferred from palette.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Chemistry {
    Vacuum = 0,
    Nitrogen = 1,
    CarbonDioxide = 2,
    Hydrogen = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Condensable {
    None = 0,
    Water = 1,
    CarbonDioxide = 2,
    Methane = 3,
}
/// Published authoring values, not automatic physical measurements or breathable-air claims.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimateInputs {
    pub chemistry: Chemistry,
    pub condensable: Condensable,
    pub pressure_pascal: f64,
    pub scale_height_meters: f64,
    pub rotation_seconds: f64,
    pub year_seconds: f64,
    pub axial_tilt_radians: f64,
    pub stellar_flux_w_m2: f64,
    pub albedo: f64,
    pub greenhouse_kelvin: f64,
    pub condensable_fraction: f64,
}
impl ClimateInputs {
    pub fn vacuum(rotation_seconds: f64, year_seconds: f64, stellar_flux_w_m2: f64) -> Self {
        Self {
            chemistry: Chemistry::Vacuum,
            condensable: Condensable::None,
            pressure_pascal: 0.0,
            scale_height_meters: 1.0,
            rotation_seconds,
            year_seconds,
            axial_tilt_radians: 0.0,
            stellar_flux_w_m2,
            albedo: 0.3,
            greenhouse_kelvin: 0.0,
            condensable_fraction: 0.0,
        }
    }
    fn values(self) -> [f64; 10] {
        [
            self.pressure_pascal,
            self.scale_height_meters,
            self.rotation_seconds,
            self.year_seconds,
            self.axial_tilt_radians,
            self.stellar_flux_w_m2,
            self.albedo,
            self.greenhouse_kelvin,
            self.condensable_fraction,
            0.0,
        ]
    }
    pub(super) fn validate(self) -> Result<(), Error> {
        if !self.values().iter().all(|v| v.is_finite())
            || !(0.0..=1e9).contains(&self.pressure_pascal)
            || !(1.0..=1e7).contains(&self.scale_height_meters)
            || !(1.0..=1e12).contains(&self.rotation_seconds)
            || !(1.0..=1e12).contains(&self.year_seconds)
            || !(0.0..=std::f64::consts::FRAC_PI_2).contains(&self.axial_tilt_radians)
            || !(0.0..=1e7).contains(&self.stellar_flux_w_m2)
            || !(0.0..=1.0).contains(&self.albedo)
            || !(0.0..=2000.0).contains(&self.greenhouse_kelvin)
            || !(0.0..=1.0).contains(&self.condensable_fraction)
            || (self.chemistry == Chemistry::Vacuum) != (self.pressure_pascal == 0.0)
            || (self.chemistry == Chemistry::Vacuum
                && (self.condensable != Condensable::None || self.greenhouse_kelvin != 0.0))
            || (self.condensable == Condensable::None && self.condensable_fraction != 0.0)
        {
            return Err(Error::InvalidDefinition);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ClimateDefinition {
    geology: GeologyDefinition,
    seed: u64,
    inputs: ClimateInputs,
    identity: String,
}
/// Same planet/position/epoch yields the same result, independent of stepping/streaming.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherSample {
    pub definition_id: String,
    pub province_id: String,
    pub epoch: Epoch,
    pub climate_temperature_kelvin: f64,
    pub temperature_kelvin: f64,
    pub pressure_pascal: f64,
    pub cloud_fraction: f64,
    pub storm_fraction: f64,
    /// Body-fixed Cartesian vector tangent to the surface; caller transforms to inertial gas axes.
    pub wind_mps: [f64; 3],
    pub precipitation: Condensable,
    pub precipitation_kg_m2_s: f64,
    /// Dimensionless atmospheric extinction per meter. Zero in vacuum (unlimited visibility).
    pub extinction_per_meter: f64,
}
impl ClimateDefinition {
    pub fn new(
        geology: GeologyDefinition,
        version: u16,
        seed: u64,
        inputs: ClimateInputs,
    ) -> Result<Self, Error> {
        if version != CLIMATE_VERSION {
            return Err(Error::UnsupportedVersion);
        }
        inputs.validate()?;
        let mut d = Self {
            geology,
            seed,
            inputs,
            identity: String::new(),
        };
        d.identity = sha256(&d.to_bytes());
        Ok(d)
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn geology(&self) -> &GeologyDefinition {
        &self.geology
    }
    pub fn inputs(&self) -> ClimateInputs {
        self.inputs
    }
    pub fn compatible_with(&self, identity: &str, geology: &GeologyDefinition) -> bool {
        self.identity == identity && self.geology == *geology
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"SLMCLM01".to_vec();
        b.extend(CLIMATE_VERSION.to_le_bytes());
        b.extend(self.seed.to_le_bytes());
        b.extend([self.inputs.chemistry as u8, self.inputs.condensable as u8]);
        for v in self.inputs.values() {
            b.extend(v.to_le_bytes());
        }
        b.extend(self.geology.to_bytes());
        b
    }
    pub fn from_bytes(body: CelestialBody, b: &[u8]) -> Result<Self, Error> {
        if b.len() < 100
            || b.len() > crate::terrain::MAX_DEFINITION_BYTES + 160
            || &b[..8] != b"SLMCLM01"
        {
            return Err(Error::InvalidEncoding);
        }
        let version = u16::from_le_bytes([b[8], b[9]]);
        let seed = u64::from_le_bytes(b[10..18].try_into().map_err(|_| Error::InvalidEncoding)?);
        let chemistry = match b[18] {
            0 => Chemistry::Vacuum,
            1 => Chemistry::Nitrogen,
            2 => Chemistry::CarbonDioxide,
            3 => Chemistry::Hydrogen,
            _ => return Err(Error::InvalidEncoding),
        };
        let condensable = match b[19] {
            0 => Condensable::None,
            1 => Condensable::Water,
            2 => Condensable::CarbonDioxide,
            3 => Condensable::Methane,
            _ => return Err(Error::InvalidEncoding),
        };
        let mut v = [0.0; 10];
        for (i, value) in v.iter_mut().enumerate() {
            *value = f64::from_le_bytes(
                b[20 + i * 8..28 + i * 8]
                    .try_into()
                    .map_err(|_| Error::InvalidEncoding)?,
            );
        }
        if v[9].to_bits() != 0 {
            return Err(Error::InvalidEncoding);
        }
        Self::new(
            GeologyDefinition::from_bytes(body, &b[100..])?,
            version,
            seed,
            ClimateInputs {
                chemistry,
                condensable,
                pressure_pascal: v[0],
                scale_height_meters: v[1],
                rotation_seconds: v[2],
                year_seconds: v[3],
                axial_tilt_radians: v[4],
                stellar_flux_w_m2: v[5],
                albedo: v[6],
                greenhouse_kelvin: v[7],
                condensable_fraction: v[8],
            },
        )
    }
    /// Continuous on the sphere and in time; no cube-face/region-specific noise or mutable RNG.
    pub fn sample(&self, direction: [f64; 3], epoch: Epoch) -> Result<WeatherSample, Error> {
        let t = seconds(epoch)?;
        let surface = self.geology.sample(direction)?;
        let p = self.inputs;
        let season = (t.rem_euclid(p.year_seconds) / p.year_seconds * TAU).sin();
        let day = t.rem_euclid(p.rotation_seconds) / p.rotation_seconds * TAU;
        let declination = p.axial_tilt_radians * season;
        let latitude = direction[1].clamp(-1.0, 1.0).asin();
        let solar = (latitude - declination).cos().max(0.0);
        let base = (p.stellar_flux_w_m2 * (1.0 - p.albedo) / (4.0 * 5.670374419e-8))
            .sqrt()
            .sqrt();
        let climate = (base * (0.7 + 0.3 * solar) + p.greenhouse_kelvin
            - surface.surface.elevation_meters * 0.0065)
            .max(0.0);
        let phase = t.rem_euclid(604800.0) / 604800.0 * TAU;
        let field = (direction[0] * 3.0
            + direction[2] * 2.0
            + direction[1]
            + phase
            + random(self.seed, 1) * TAU)
            .sin();
        let vacuum = p.chemistry == Chemistry::Vacuum;
        let storm = if vacuum {
            0.0
        } else {
            ((field - 0.15) / 0.85).clamp(0.0, 1.0).powi(2)
        };
        let pressure = p.pressure_pascal
            * (-surface.surface.elevation_meters / p.scale_height_meters)
                .clamp(-20.0, 20.0)
                .exp();
        // Global smooth tangential circulation. No singular east basis at the poles.
        let raw = [
            -direction[2],
            0.2 * (phase + direction[0]).sin(),
            direction[0],
        ];
        let radial: f64 = raw.iter().zip(direction).map(|(a, b)| a * b).sum();
        let wind = if vacuum {
            [0.0; 3]
        } else {
            std::array::from_fn(|i| {
                (raw[i] - radial * direction[i])
                    * (2.0 + 18.0 * storm)
                    * (p.pressure_pascal / 101325.0).min(1.0)
            })
        };
        let daylight = (direction[0] * day.cos() + direction[2] * day.sin()).max(0.0);
        let temperature =
            (climate + (daylight - 0.5) * if vacuum { 120.0 } else { 12.0 } - storm * 4.0).max(0.0);
        let condensation_k = match p.condensable {
            Condensable::None => 0.0,
            Condensable::Water => 373.15,
            Condensable::CarbonDioxide => 216.6,
            Condensable::Methane => 111.7,
        };
        // A smooth ten-kelvin transition avoids cloud/rain popping at the coarse
        // chemistry-dependent cutoff. This is not a pressure-dependent phase model.
        let condensation = ((condensation_k - temperature) / 10.0).clamp(0.0, 1.0);
        let condensation = condensation * condensation * (3.0 - 2.0 * condensation);
        let cloud = if vacuum {
            0.0
        } else {
            (0.5 + 0.5 * field) * p.condensable_fraction * condensation
        };
        let precipitation = cloud * storm * 0.001 * (0.5 + 0.5 * surface.geology.weathering);
        Ok(WeatherSample {
            definition_id: self.identity.clone(),
            province_id: surface.geology.province_id,
            epoch,
            climate_temperature_kelvin: climate,
            temperature_kelvin: temperature,
            pressure_pascal: pressure,
            cloud_fraction: cloud,
            storm_fraction: storm,
            wind_mps: wind,
            precipitation: if precipitation > 0.0 {
                p.condensable
            } else {
                Condensable::None
            },
            precipitation_kg_m2_s: precipitation,
            extinction_per_meter: if vacuum {
                0.0
            } else {
                (cloud * 0.0001 + storm * 0.001) * (pressure / 101325.0).min(10.0)
            },
        })
    }
}
#[cfg(test)]
mod tests;
