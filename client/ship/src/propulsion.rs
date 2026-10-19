//! Validated force/mass input. Cargo and energy owners update this cached value.
use std::fmt;

pub const STARTER_DRY_MASS_KG: f64 = 100_000.0;
pub const STARTER_FORWARD_FORCE_NEWTONS: f64 = 4_000_000.0;
pub const STARTER_REVERSE_FORCE_NEWTONS: f64 = 2_000_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidFlightMotion;

impl fmt::Display for InvalidFlightMotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "flight velocity and forces must be finite; mass must be at least the dry hull mass",
        )
    }
}
impl std::error::Error for InvalidFlightMotion {}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Propulsion {
    total_mass_kg: f64,
    forward_force_newtons: f64,
    reverse_force_newtons: f64,
}
impl Default for Propulsion {
    fn default() -> Self {
        Self {
            total_mass_kg: STARTER_DRY_MASS_KG,
            forward_force_newtons: STARTER_FORWARD_FORCE_NEWTONS,
            reverse_force_newtons: STARTER_REVERSE_FORCE_NEWTONS,
        }
    }
}
impl Propulsion {
    pub fn new(
        total_mass_kg: f64,
        forward_force_newtons: f64,
        reverse_force_newtons: f64,
    ) -> Result<Self, InvalidFlightMotion> {
        if !total_mass_kg.is_finite()
            || total_mass_kg < STARTER_DRY_MASS_KG
            || !forward_force_newtons.is_finite()
            || forward_force_newtons < 0.0
            || !reverse_force_newtons.is_finite()
            || reverse_force_newtons < 0.0
        {
            return Err(InvalidFlightMotion);
        }
        Ok(Self {
            total_mass_kg,
            forward_force_newtons,
            reverse_force_newtons,
        })
    }
    pub fn with_total_mass(self, mass_kg: f64) -> Result<Self, InvalidFlightMotion> {
        Self::new(
            mass_kg,
            self.forward_force_newtons,
            self.reverse_force_newtons,
        )
    }

    #[must_use]
    pub fn total_mass_kg(self) -> f64 {
        self.total_mass_kg
    }

    /// Signed fraction of local +X force: negative commands use reverse capacity.
    #[must_use]
    pub fn acceleration_meters_per_second_squared(self, percentage: i8) -> f64 {
        let fraction = f64::from(percentage.clamp(-100, 100)) / 100.0;
        let force = if fraction < 0.0 {
            self.reverse_force_newtons
        } else {
            self.forward_force_newtons
        };
        fraction * force / self.total_mass_kg
    }
}

impl Propulsion {
    pub(crate) fn validate_save(&self) -> Result<(), salimon_persistence::SaveError> {
        Self::new(
            self.total_mass_kg,
            self.forward_force_newtons,
            self.reverse_force_newtons,
        )
        .map(|_| ())
        .map_err(|_| salimon_persistence::invalid("propulsion"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_force_and_cached_mass_determine_acceleration() {
        assert_eq!(
            Propulsion::default().acceleration_meters_per_second_squared(100),
            40.0
        );
        assert_eq!(
            Propulsion::default().acceleration_meters_per_second_squared(-100),
            -20.0
        );
        let loaded = Propulsion::new(200_000.0, 4_000_000.0, 2_000_000.0).unwrap();
        assert_eq!(loaded.acceleration_meters_per_second_squared(100), 20.0);
        assert_eq!(loaded.acceleration_meters_per_second_squared(0), 0.0);
        for mass in [0.0, -1.0, 99_999.0, f64::NAN, f64::INFINITY] {
            assert!(Propulsion::new(mass, 4_000_000.0, 2_000_000.0).is_err());
        }
        assert!(Propulsion::new(STARTER_DRY_MASS_KG, f64::NAN, 0.0).is_err());
    }
}
