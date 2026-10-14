//! Mechanical one-fragment intake and automatic joule conversion.
use crate::{InvalidEnergy, ShipController};
use salimon_world::{mining::MiningSession, resources::ResourceId};
use std::time::Duration;

/// Explicit material-keyed tuning; defaults come from the world resource catalog.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConversionConfig {
    pub joules_per_second: f64,
    pub iron_joules_per_kg: f64,
    pub silicate_joules_per_kg: f64,
    pub ice_joules_per_kg: f64,
}
impl Default for ConversionConfig {
    fn default() -> Self {
        Self {
            joules_per_second: 200_000_000.0,
            iron_joules_per_kg: ResourceId::IronOre.definition().energy_joules_per_kg,
            silicate_joules_per_kg: ResourceId::SilicateRock.definition().energy_joules_per_kg,
            ice_joules_per_kg: ResourceId::WaterIce.definition().energy_joules_per_kg,
        }
    }
}
impl ConversionConfig {
    fn yield_per_kg(self, material: ResourceId) -> f64 {
        match material {
            ResourceId::IronOre => self.iron_joules_per_kg,
            ResourceId::SilicateRock => self.silicate_joules_per_kg,
            ResourceId::WaterIce => self.ice_joules_per_kg,
        }
    }
    fn validate(self) -> Result<Self, InvalidEnergy> {
        if !self.joules_per_second.is_finite()
            || self.joules_per_second <= 0.0
            || [
                self.iron_joules_per_kg,
                self.silicate_joules_per_kg,
                self.ice_joules_per_kg,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
        {
            return Err(InvalidEnergy);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Conversion {
    config: ConversionConfig,
    remaining_joules: f64,
}
impl Conversion {
    pub(crate) fn remaining_duration(&self) -> Duration {
        Duration::try_from_secs_f64(self.remaining_joules / self.config.joules_per_second)
            .unwrap_or(Duration::MAX)
    }
    pub(crate) fn advance(&mut self, delta: Duration) -> f64 {
        let converted = if delta >= self.remaining_duration() {
            self.remaining_joules
        } else {
            self.remaining_joules
                .min(self.config.joules_per_second * delta.as_secs_f64())
        };
        self.remaining_joules = (self.remaining_joules - converted).max(0.0);
        converted
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntakeResult {
    Accepted,
    Busy,
    NoCarriedFragment,
    InvalidYield,
}
impl ShipController {
    /// Caller supplies the normal near-Core interaction gate. Intake is mechanical
    /// and never requires cockpit ownership, artificial gravity or bus electricity.
    pub fn intake_carried_fragment(&mut self, session: &mut MiningSession) -> IntakeResult {
        if self.conversion.remaining_joules > 0.0 {
            return IntakeResult::Busy;
        }
        let Some(id) = session.carried_id() else {
            return IntakeResult::NoCarriedFragment;
        };
        let Some(fragment) = session.fragments().iter().find(|f| f.id() == id) else {
            return IntakeResult::NoCarriedFragment;
        };
        let material = fragment.material();
        let joules = material.mass_kg() * self.conversion.config.yield_per_kg(material.resource());
        if !joules.is_finite() {
            return IntakeResult::InvalidYield;
        }
        if !session.remove_fragment(id) {
            return IntakeResult::NoCarriedFragment;
        }
        self.conversion.remaining_joules = joules;
        IntakeResult::Accepted
    }
    pub fn conversion_remaining_joules(&self) -> f64 {
        self.conversion.remaining_joules
    }
    /// Capture a fragment's yield at intake; tuning updates apply to future intakes.
    pub fn configure_conversion(&mut self, config: ConversionConfig) -> Result<(), InvalidEnergy> {
        self.conversion.config = config.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::{
        CelestialBodyId, WorldPosition,
        resources::{DepositId, RawMaterial, ResourceDeposit},
    };
    fn carried(resource: ResourceId, mass: f64) -> MiningSession {
        let mut session = MiningSession::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(resource, mass).unwrap(),
            WorldPosition::new(0.0, 0.0, 0.0),
            mass,
        )
        .unwrap();
        session.extract(&mut deposit, Duration::from_secs(10));
        assert!(session.pick_up(session.fragments()[0].id()));
        session
    }
    #[test]
    fn intake_consumes_once_and_rejects_second_until_completion() {
        for resource in [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ] {
            for mass in [0.125, 1.0, 2.0] {
                let mut ship = ShipController::default();
                let mut session = carried(resource, mass);
                assert_eq!(
                    ship.intake_carried_fragment(&mut session),
                    IntakeResult::Accepted
                );
                assert!(session.fragments().is_empty());
                assert_eq!(session.carried_id(), None);
                let expected = mass * resource.definition().energy_joules_per_kg;
                assert_eq!(ship.conversion_remaining_joules(), expected);
                assert_eq!(
                    ship.intake_carried_fragment(&mut session),
                    IntakeResult::Busy
                );
                ship.advance(Duration::from_secs(100));
                assert_eq!(ship.conversion_remaining_joules(), 0.0);
                assert_eq!(
                    ship.intake_carried_fragment(&mut session),
                    IntakeResult::NoCarriedFragment
                );
            }
        }
    }
    #[test]
    fn dead_core_bootstraps_and_full_core_keeps_processing_with_discharge() {
        let mut ship = ShipController::default();
        ship.set_stored_core_energy_joules(0);
        let mut session = carried(ResourceId::IronOre, 2.0);
        assert_eq!(
            ship.intake_carried_fragment(&mut session),
            IntakeResult::Accepted
        );
        ship.advance(Duration::from_secs(1));
        assert!(ship.snapshot().power_available);
        assert_eq!(ship.snapshot().energy_core.stored_joules, 199_890_000);
        ship.set_stored_core_energy_joules(u64::MAX);
        ship.advance(Duration::from_secs(9));
        assert_eq!(ship.conversion_remaining_joules(), 0.0);
        assert_eq!(
            ship.snapshot().energy_core.stored_joules,
            ship.snapshot().energy_core.capacity_joules
        );
    }
    #[test]
    fn zero_yield_is_consumed_and_invalid_config_preserves_state() {
        let mut ship = ShipController::default();
        ship.configure_conversion(ConversionConfig {
            iron_joules_per_kg: 0.0,
            ..Default::default()
        })
        .unwrap();
        assert_eq!(
            ship.intake_carried_fragment(&mut carried(ResourceId::IronOre, 1.0)),
            IntakeResult::Accepted
        );
        assert_eq!(ship.conversion_remaining_joules(), 0.0);
        assert!(
            ship.configure_conversion(ConversionConfig {
                joules_per_second: f64::NAN,
                ..Default::default()
            })
            .is_err()
        );
    }
}
