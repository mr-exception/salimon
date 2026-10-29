//! Published environmental definitions and stateless body-fixed epoch queries.
//! Authoritative metadata only; presentation never owns weather, sites or ecology.
pub mod activity;
pub mod atmosphere;
pub mod climate;
pub mod ecology;
pub mod landmarks;
pub mod organism_placement;
pub mod organisms;
pub mod signature_sites;
pub mod weather_visuals;

use crate::terrain::TerrainError;
use salimon_universe::{Epoch, TimeScale};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentError {
    InvalidDefinition,
    InvalidEncoding,
    UnsupportedVersion,
    IncompatibleDefinition,
    InvalidEpoch,
    Terrain(TerrainError),
}
impl From<TerrainError> for EnvironmentError {
    fn from(error: TerrainError) -> Self {
        Self::Terrain(error)
    }
}
/// Uniform TT time relative to J2000 noon. Never use load time or convert UTC implicitly.
pub(crate) fn seconds(epoch: Epoch) -> Result<f64, EnvironmentError> {
    let origin =
        Epoch::calendar([2000, 1, 1], [12, 0, 0, 0], TimeScale::Tt).expect("valid fixed TT epoch");
    epoch
        .seconds_since(origin)
        .map_err(|_| EnvironmentError::InvalidEpoch)
}
pub(crate) fn random(seed: u64, domain: u64) -> f64 {
    let mut x = seed ^ domain.wrapping_mul(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((x ^ (x >> 31)) >> 11) as f64 / (1_u64 << 53) as f64
}
#[cfg(test)]
pub(crate) fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = v[0].hypot(v[1]).hypot(v[2]);
    v.map(|x| x / n)
}
