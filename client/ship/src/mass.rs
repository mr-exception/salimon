//! Event-driven physical mass ledger. Location/handling never creates a second identity.
use crate::{InvalidFlightMotion, STARTER_DRY_MASS_KG};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MassObjectId {
    Fragment(u64),
    Equipment(u64),
    FabricatorReserve,
    FabricatorTray,
}

#[derive(Clone, Debug)]
pub struct ShipMass {
    objects: BTreeMap<MassObjectId, f64>,
    total_kg: f64,
    cargo_partials: Vec<f64>,
}
impl Default for ShipMass {
    fn default() -> Self {
        Self {
            objects: BTreeMap::new(),
            total_kg: STARTER_DRY_MASS_KG,
            cargo_partials: Vec::new(),
        }
    }
}
impl ShipMass {
    pub fn total_kg(&self) -> f64 {
        self.total_kg
    }

    /// Some means physically onboard (including installed, stored or carried);
    /// None means boundary exit, despawn or consumption. Repeated events are idempotent.
    pub fn update(
        &mut self,
        id: MassObjectId,
        mass_kg: Option<f64>,
    ) -> Result<bool, InvalidFlightMotion> {
        if mass_kg.is_some_and(|m| !m.is_finite() || m <= 0.0) {
            return Err(InvalidFlightMotion);
        }
        let old = self.objects.get(&id).copied();
        if old == mass_kg {
            return Ok(false);
        }
        // Keep cargo separate from the much larger hull. Nonoverlapping partials
        // retain low bits through replacement/removal, including tiny residuals.
        // Work depends on floating-point exponent range, not the object count.
        let mut cargo = self.cargo_partials.clone();
        add_partial(&mut cargo, -old.unwrap_or(0.0))?;
        add_partial(&mut cargo, mass_kg.unwrap_or(0.0))?;
        let empty = mass_kg.is_none() && old.is_some() && self.objects.len() == 1;
        if empty {
            cargo.clear();
        }
        let cargo_kg: f64 = cargo.iter().sum();
        let next = STARTER_DRY_MASS_KG + cargo_kg;
        if !next.is_finite() || cargo_kg < 0.0 {
            return Err(InvalidFlightMotion);
        }
        if let Some(mass) = mass_kg {
            self.objects.insert(id, mass);
        } else {
            self.objects.remove(&id);
        }
        self.cargo_partials = cargo;
        self.total_kg = next;
        Ok(true)
    }

    /// Explicit diagnostic/import reconciliation, never called by flight ticks.
    pub fn reconciles(&self, objects: impl IntoIterator<Item = (MassObjectId, f64)>) -> bool {
        let mut expected = Self::default();
        for (id, mass) in objects {
            if expected.objects.contains_key(&id) || expected.update(id, Some(mass)).is_err() {
                return false;
            }
        }
        self.objects == expected.objects && (self.total_kg - expected.total_kg).abs() <= 1e-6
    }
}
/// Error-free two-sum expansion: each roundoff survives as a smaller partial.
/// Inputs/previous totals are finite; overflow rejects the entire ledger update.
fn add_partial(partials: &mut Vec<f64>, mut value: f64) -> Result<(), InvalidFlightMotion> {
    let mut retained = 0;
    for index in 0..partials.len() {
        let mut other = partials[index];
        if value.abs() < other.abs() {
            std::mem::swap(&mut value, &mut other);
        }
        let sum = value + other;
        if !sum.is_finite() {
            return Err(InvalidFlightMotion);
        }
        let error = other - (sum - value);
        if error != 0.0 {
            partials[retained] = error;
            retained += 1;
        }
        value = sum;
    }
    partials.truncate(retained);
    if value != 0.0 {
        partials.push(value);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_reported_removal_sequence_returns_to_dry_mass() {
        let mut ledger = ShipMass::default();
        let masses = [8.88252139878047, 6.8409055493844075, 8.003999726694019];
        for (id, mass) in masses.into_iter().enumerate() {
            ledger
                .update(MassObjectId::Fragment(id as u64), Some(mass))
                .unwrap();
        }
        for id in (0..3).rev() {
            ledger.update(MassObjectId::Fragment(id), None).unwrap();
        }
        assert_eq!(ledger.total_kg(), STARTER_DRY_MASS_KG);
        assert!(ledger.reconciles([]));
    }

    #[test]
    fn shuffled_replacements_and_transfers_preserve_tiny_residuals() {
        let mut ledger = ShipMass::default();
        let huge = MassObjectId::Equipment(9);
        let residual = MassObjectId::Fragment(9);
        for _ in 0..1000 {
            ledger.update(huge, Some(1e20)).unwrap();
            ledger.update(residual, Some(0.000_003)).unwrap();
            ledger.update(huge, Some(8.88252139878047)).unwrap();
            ledger.update(huge, None).unwrap();
            assert_eq!(ledger.total_kg(), STARTER_DRY_MASS_KG + 0.000_003);
            assert!(ledger.reconciles([(residual, 0.000_003)]));
            ledger.update(residual, None).unwrap();
            assert_eq!(ledger.total_kg(), STARTER_DRY_MASS_KG);
        }
        for order in [[0, 1, 2], [2, 0, 1], [1, 2, 0]] {
            let masses = [0.000_003, 10.0, 6.8409055493844075];
            for id in order {
                ledger
                    .update(MassObjectId::Fragment(id as u64), Some(masses[id]))
                    .unwrap();
            }
            for id in order.into_iter().rev() {
                ledger
                    .update(MassObjectId::Fragment(id as u64), None)
                    .unwrap();
            }
            assert_eq!(ledger.total_kg(), STARTER_DRY_MASS_KG);
        }
    }

    #[test]
    fn overflow_and_invalid_updates_leave_ledger_unchanged() {
        let mut ledger = ShipMass::default();
        let id = MassObjectId::Equipment(1);
        ledger.update(id, Some(f64::MAX)).unwrap();
        assert!(
            ledger
                .update(MassObjectId::Equipment(2), Some(f64::MAX))
                .is_err()
        );
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(ledger.update(id, Some(invalid)).is_err());
        }
        assert!(ledger.reconciles([(id, f64::MAX)]));
        ledger.update(id, None).unwrap();
        assert_eq!(ledger.total_kg(), STARTER_DRY_MASS_KG);
    }

    #[test]
    fn five_hundred_objects_transfers_consumption_equipment_and_validation() {
        let mut mass = ShipMass::default();
        mass.update(MassObjectId::Equipment(1), Some(100.0))
            .unwrap();
        for id in 0..500 {
            mass.update(MassObjectId::Fragment(id), Some(2.0)).unwrap();
            assert!(!mass.update(MassObjectId::Fragment(id), Some(2.0)).unwrap());
        }
        assert_eq!(mass.total_kg(), 101_100.0);
        assert!(
            mass.reconciles(
                std::iter::once((MassObjectId::Equipment(1), 100.0))
                    .chain((0..500).map(|id| (MassObjectId::Fragment(id), 2.0)))
            )
        );
        for id in 0..500 {
            mass.update(MassObjectId::Fragment(id), None).unwrap();
        }
        assert_eq!(mass.total_kg(), 100_100.0);
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                mass.update(MassObjectId::Equipment(1), Some(invalid))
                    .is_err()
            );
            assert_eq!(mass.total_kg(), 100_100.0);
        }
    }
}
