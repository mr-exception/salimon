//! Canonical passive geological sources. No terrain deformation, hazards or new colliders.
use super::{
    EnvironmentError as Error,
    climate::{Chemistry, ClimateDefinition},
    random, seconds,
};
use crate::{
    CelestialBody,
    terrain::geology::{GeologicalProcess, GeologyProfile, Substrate},
};
use salimon_universe::{Epoch, synthesis::sha256};
use std::f64::consts::{PI, TAU};
pub const ACTIVITY_VERSION: u16 = 1;
const MAX_CANDIDATES: u8 = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ActivityKind {
    Dormant = 0,
    Vent = 1,
    Volcanic = 2,
    Seismic = 3,
    Cryovolcanic = 4,
}
/// Explicit per-body activity authorization/model. Ancient volcanism alone never enables sources.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActivityInputs {
    pub kind: ActivityKind,
    pub candidate_count: u8,
    pub heat_flux_w_m2: f64,
    pub site_radius_meters: f64,
    pub cycle_seconds: f64,
    pub active_fraction: f64,
}
impl ActivityInputs {
    pub fn dormant() -> Self {
        Self {
            kind: ActivityKind::Dormant,
            candidate_count: 0,
            heat_flux_w_m2: 0.0,
            site_radius_meters: 1.0,
            cycle_seconds: 86400.0,
            active_fraction: 0.0,
        }
    }
    fn values(self) -> [f64; 4] {
        [
            self.heat_flux_w_m2,
            self.site_radius_meters,
            self.cycle_seconds,
            self.active_fraction,
        ]
    }
    fn validate(self, climate: &ClimateDefinition) -> Result<(), Error> {
        if self.candidate_count > MAX_CANDIDATES
            || !self.values().iter().all(|v| v.is_finite())
            || !(0.0..=10000.0).contains(&self.heat_flux_w_m2)
            || !(1.0..=(climate.geology().terrain().radius_meters() * 0.001).max(1.0))
                .contains(&self.site_radius_meters)
            || !(1.0..=1e12).contains(&self.cycle_seconds)
            || !(0.0..=1.0).contains(&self.active_fraction)
            || (self.kind == ActivityKind::Dormant
                && (self.candidate_count != 0
                    || self.heat_flux_w_m2 != 0.0
                    || self.active_fraction != 0.0))
        {
            return Err(Error::InvalidDefinition);
        }
        let profile = climate.geology().profile();
        let suitable = match self.kind {
            ActivityKind::Dormant => true,
            ActivityKind::Cryovolcanic => profile == GeologyProfile::Ice,
            ActivityKind::Seismic => profile == GeologyProfile::WeatheredRock,
            ActivityKind::Vent | ActivityKind::Volcanic => matches!(
                profile,
                GeologyProfile::DryRock | GeologyProfile::WeatheredRock
            ),
        };
        if !suitable {
            return Err(Error::InvalidDefinition);
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActivitySite {
    pub id: String,
    /// Candidate slot never changes when another candidate is excluded.
    pub slot: u8,
    pub kind: ActivityKind,
    pub direction: [f64; 3],
    pub body_local_position_meters: [f64; 3],
    pub outward_normal: [f64; 3],
    pub substrate: Substrate,
    pub province_id: String,
    pub radius_meters: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlumeTransport {
    BallisticVacuum,
    Atmospheric,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcousticPath {
    ContactOnly,
    AirAndContact,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Plume {
    pub transport: PlumeTransport,
    pub height_meters: f64,
    /// Body-fixed source normal plus atmospheric advection; no airborne wind in vacuum.
    pub velocity_mps: [f64; 3],
}
/// Continuous source levels, not commands to replay a transient on each load/frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityEvent {
    pub site: ActivitySite,
    pub epoch: Epoch,
    pub intensity: f64,
    pub thermal_power_watts: f64,
    pub plume: Option<Plume>,
    pub acoustic_path: AcousticPath,
    pub local_pressure_pascal: f64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ActivityDefinition {
    climate: ClimateDefinition,
    seed: u64,
    inputs: ActivityInputs,
    identity: String,
    sites: Vec<ActivitySite>,
}
impl ActivityDefinition {
    pub fn new(
        climate: ClimateDefinition,
        version: u16,
        seed: u64,
        inputs: ActivityInputs,
    ) -> Result<Self, Error> {
        if version != ACTIVITY_VERSION {
            return Err(Error::UnsupportedVersion);
        }
        inputs.validate(&climate)?;
        let mut d = Self {
            climate,
            seed,
            inputs,
            identity: String::new(),
            sites: Vec::new(),
        };
        d.identity = sha256(&d.to_bytes());
        if inputs.heat_flux_w_m2 > 0.0 && inputs.active_fraction > 0.0 {
            for slot in 0..inputs.candidate_count {
                let z = 2.0 * random(seed, 1000 + u64::from(slot) * 2) - 1.0;
                let angle = TAU * random(seed, 1001 + u64::from(slot) * 2);
                let r = (1.0 - z * z).max(0.0).sqrt();
                let direction = [r * angle.cos(), z, r * angle.sin()];
                let s = d.climate.geology().sample(direction)?;
                let suitable = match inputs.kind {
                    ActivityKind::Dormant => false,
                    ActivityKind::Cryovolcanic => s.geology.substrate == Substrate::WaterIce,
                    ActivityKind::Seismic => s.geology.history.iter().any(|h| {
                        h.process == GeologicalProcess::TectonicDeformation
                            && h.relative_extent > 0.35
                    }),
                    ActivityKind::Vent | ActivityKind::Volcanic => {
                        s.geology.history.iter().any(|h| {
                            h.process == GeologicalProcess::Volcanism && h.relative_extent > 0.35
                        })
                    }
                };
                // Avoid steep unsupported emitter sites; source is anchored to the exact physical surface.
                if suitable && s.surface.slope <= 0.5 {
                    d.sites.push(ActivitySite {
                        id: format!("{}:{slot}", d.identity),
                        slot,
                        kind: inputs.kind,
                        direction,
                        body_local_position_meters: s.surface.body_local_position_meters,
                        outward_normal: s.surface.outward_normal,
                        substrate: s.geology.substrate,
                        province_id: s.geology.province_id,
                        radius_meters: inputs.site_radius_meters,
                    });
                }
            }
        }
        Ok(d)
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn climate(&self) -> &ClimateDefinition {
        &self.climate
    }
    pub fn inputs(&self) -> ActivityInputs {
        self.inputs
    }
    pub fn sites(&self) -> &[ActivitySite] {
        &self.sites
    }
    pub fn compatible_with(&self, identity: &str, climate: &ClimateDefinition) -> bool {
        self.identity == identity && self.climate == *climate
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = b"SLMACT01".to_vec();
        b.extend(ACTIVITY_VERSION.to_le_bytes());
        b.extend(self.seed.to_le_bytes());
        b.extend([self.inputs.kind as u8, self.inputs.candidate_count]);
        for v in self.inputs.values() {
            b.extend(v.to_le_bytes());
        }
        b.extend(self.climate.to_bytes());
        b
    }
    pub fn from_bytes(body: CelestialBody, b: &[u8]) -> Result<Self, Error> {
        if b.len() < 52
            || b.len() > crate::terrain::MAX_DEFINITION_BYTES + 256
            || &b[..8] != b"SLMACT01"
        {
            return Err(Error::InvalidEncoding);
        }
        let version = u16::from_le_bytes([b[8], b[9]]);
        let seed = u64::from_le_bytes(b[10..18].try_into().map_err(|_| Error::InvalidEncoding)?);
        let kind = match b[18] {
            0 => ActivityKind::Dormant,
            1 => ActivityKind::Vent,
            2 => ActivityKind::Volcanic,
            3 => ActivityKind::Seismic,
            4 => ActivityKind::Cryovolcanic,
            _ => return Err(Error::InvalidEncoding),
        };
        let mut v = [0.0; 4];
        for (i, x) in v.iter_mut().enumerate() {
            *x = f64::from_le_bytes(
                b[20 + i * 8..28 + i * 8]
                    .try_into()
                    .map_err(|_| Error::InvalidEncoding)?,
            );
        }
        Self::new(
            ClimateDefinition::from_bytes(body, &b[52..])?,
            version,
            seed,
            ActivityInputs {
                kind,
                candidate_count: b[19],
                heat_flux_w_m2: v[0],
                site_radius_meters: v[1],
                cycle_seconds: v[2],
                active_fraction: v[3],
            },
        )
    }
    pub fn sample(&self, site_id: &str, epoch: Epoch) -> Result<ActivityEvent, Error> {
        let t = seconds(epoch)?;
        let site = self
            .sites
            .iter()
            .find(|s| s.id == site_id)
            .ok_or(Error::IncompatibleDefinition)?;
        let phase = (t.rem_euclid(self.inputs.cycle_seconds) / self.inputs.cycle_seconds
            + random(self.seed, 2000 + u64::from(site.slot)))
        .fract();
        let intensity = if phase < self.inputs.active_fraction {
            (PI * phase / self.inputs.active_fraction).sin().powi(2)
        } else {
            0.0
        };
        let weather = self.climate.sample(site.direction, epoch)?;
        let vacuum = self.climate.inputs().chemistry == Chemistry::Vacuum;
        let plume = if site.kind == ActivityKind::Seismic || intensity == 0.0 {
            None
        } else {
            Some(Plume {
                transport: if vacuum {
                    PlumeTransport::BallisticVacuum
                } else {
                    PlumeTransport::Atmospheric
                },
                height_meters: site.radius_meters * intensity * if vacuum { 10.0 } else { 5.0 },
                velocity_mps: std::array::from_fn(|i| {
                    site.outward_normal[i] * intensity * 20.0 + weather.wind_mps[i]
                }),
            })
        };
        Ok(ActivityEvent {
            site: site.clone(),
            epoch,
            intensity,
            thermal_power_watts: self.inputs.heat_flux_w_m2
                * PI
                * site.radius_meters.powi(2)
                * intensity,
            plume,
            acoustic_path: if vacuum {
                AcousticPath::ContactOnly
            } else {
                AcousticPath::AirAndContact
            },
            local_pressure_pascal: weather.pressure_pascal,
        })
    }
    /// Read-only finite local selection. Eviction cannot create/rekey a source or replay an event.
    pub fn events_within(
        &self,
        body_local_position_meters: [f64; 3],
        range_meters: f64,
        epoch: Epoch,
    ) -> Result<Vec<ActivityEvent>, Error> {
        seconds(epoch)?;
        if !body_local_position_meters.iter().all(|x| x.is_finite())
            || !range_meters.is_finite()
            || range_meters < 0.0
        {
            return Err(Error::InvalidDefinition);
        }
        self.sites
            .iter()
            .filter(|s| {
                let delta = std::array::from_fn::<_, 3, _>(|i| {
                    s.body_local_position_meters[i] - body_local_position_meters[i]
                });
                delta[0].hypot(delta[1]).hypot(delta[2]) <= range_meters
            })
            .map(|s| self.sample(&s.id, epoch))
            .collect()
    }
}
#[cfg(test)]
mod tests;
