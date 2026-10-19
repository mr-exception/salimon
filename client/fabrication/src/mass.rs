use crate::FabricationError;

/// Positive integer milligrams. Balances may be zero and use plain u64.
/// Quantize world kilograms once, nearest milligram (half rounds upward).
/// Sub-half-milligram or overflowing input is rejected without consuming it.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct Mass(u64);

impl Mass {
    pub fn from_milligrams(milligrams: u64) -> Result<Self, FabricationError> {
        if milligrams == 0 {
            return Err(FabricationError::InvalidMass);
        }
        Ok(Self(milligrams))
    }

    pub fn from_kg(kg: f64) -> Result<Self, FabricationError> {
        let mg = (kg * 1_000_000.0).round();
        // Strict upper bound avoids the f64 rounding of u64::MAX to 2^64.
        if !kg.is_finite() || kg <= 0.0 || mg < 1.0 || mg >= u64::MAX as f64 {
            return Err(FabricationError::InvalidMass);
        }
        Ok(Self(mg as u64))
    }

    pub const fn milligrams(self) -> u64 {
        self.0
    }

    pub fn kg(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}
