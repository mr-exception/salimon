//! Intentional fictional Core propulsion: funded joules become impulse, not kinetic-energy billing.
use std::fmt;
pub const STARTER_DRY_MASS_KG: f64 = 100_000.0;
pub const STARTER_IMPULSE_NS_PER_JOULE: f64 = 1.0;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidFlightMotion;
impl fmt::Display for InvalidFlightMotion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "finite velocity, mass >= dry hull, and impulse coefficient in 0..=1000 N.s/J required",
        )
    }
}
impl std::error::Error for InvalidFlightMotion {}
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Propulsion {
    #[serde(skip, default = "saved_dry_mass")]
    total_mass_kg: f64,
    impulse_ns_per_joule: f64,
}
fn saved_dry_mass() -> f64 {
    STARTER_DRY_MASS_KG
}
impl Default for Propulsion {
    fn default() -> Self {
        Self {
            total_mass_kg: STARTER_DRY_MASS_KG,
            impulse_ns_per_joule: STARTER_IMPULSE_NS_PER_JOULE,
        }
    }
}
impl Propulsion {
    /// Numeric coefficient bound keeps finite funded-energy / mass arithmetic
    /// finite even for an f64::MAX input. This is not a velocity limit or upgrade.
    pub fn new(total_mass_kg: f64, impulse_ns_per_joule: f64) -> Result<Self, InvalidFlightMotion> {
        if !total_mass_kg.is_finite()
            || total_mass_kg < STARTER_DRY_MASS_KG
            || !impulse_ns_per_joule.is_finite()
            || !(0.0..=1000.0).contains(&impulse_ns_per_joule)
        {
            return Err(InvalidFlightMotion);
        }
        Ok(Self {
            total_mass_kg,
            impulse_ns_per_joule,
        })
    }
    pub fn with_total_mass(self, mass_kg: f64) -> Result<Self, InvalidFlightMotion> {
        Self::new(mass_kg, self.impulse_ns_per_joule)
    }
    pub fn total_mass_kg(self) -> f64 {
        self.total_mass_kg
    }
    /// Signed funded propulsion energy only; callers own allocation and direction.
    pub fn delta_velocity_meters_per_second(self, signed_funded_joules: f64) -> f64 {
        (signed_funded_joules / self.total_mass_kg) * self.impulse_ns_per_joule
    }
    /// F=kappa*P, a=F/m. Power comes from the Core configuration, never a second force capacity.
    pub fn acceleration_meters_per_second_squared(self, signed_watts: f64) -> f64 {
        self.delta_velocity_meters_per_second(signed_watts)
    }
    pub(crate) fn validate_save(&self) -> Result<(), salimon_persistence::SaveError> {
        Self::new(self.total_mass_kg, self.impulse_ns_per_joule)
            .map(|_| ())
            .map_err(|_| salimon_persistence::invalid("propulsion coefficient/mass"))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn funded_impulse_and_cached_mass_are_the_only_velocity_inputs() {
        let p = Propulsion::default();
        assert_eq!(p.delta_velocity_meters_per_second(100e6), 1000.0);
        assert_eq!(p.delta_velocity_meters_per_second(-50e6), -500.0);
        assert_eq!(p.delta_velocity_meters_per_second(1e9), 10_000.0);
        assert_eq!(
            p.with_total_mass(200_000.0)
                .unwrap()
                .delta_velocity_meters_per_second(100e6),
            500.0
        );
        for mass in [0.0, 99_999.0, f64::NAN, f64::INFINITY] {
            assert!(Propulsion::new(mass, 1.0).is_err());
        }
        for k in [-1.0, 1001.0, f64::NAN, f64::INFINITY] {
            assert!(Propulsion::new(100_000.0, k).is_err());
        }
    }
}
