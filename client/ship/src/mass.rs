//! Event-driven physical mass ledger. Location/handling never creates a second identity.
use crate::{InvalidFlightMotion, STARTER_DRY_MASS_KG};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MassObjectId {
    Fragment(u64),
    Equipment(u64),
}

#[derive(Clone, Debug)]
pub struct ShipMass {
    objects: BTreeMap<MassObjectId, f64>,
    total_kg: f64,
}
impl Default for ShipMass {
    fn default() -> Self {
        Self {
            objects: BTreeMap::new(),
            total_kg: STARTER_DRY_MASS_KG,
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
        let next = self.total_kg - old.unwrap_or(0.0) + mass_kg.unwrap_or(0.0);
        if !next.is_finite() || next < STARTER_DRY_MASS_KG {
            return Err(InvalidFlightMotion);
        }
        if let Some(mass) = mass_kg {
            self.objects.insert(id, mass);
        } else {
            self.objects.remove(&id);
        }
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
#[cfg(test)]
mod tests {
    use super::*;
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
