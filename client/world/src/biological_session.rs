//! Atomic biological source debit and physical object publication in the existing owner.
use super::MiningSession;
use crate::{
    WorldPosition,
    environment::harvest::{Catalog, Delta, Journal, Key, Method},
    resources::{FragmentId, RawMaterial, ResourceDeposit, ResourceTransform},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarvestError {
    InvalidSource,
    StaleRevision,
    Occupied,
    TooLarge,
    Exhausted,
}
impl MiningSession {
    pub fn detach_biological(
        &mut self,
        catalog: &Catalog,
        journal: &mut Journal,
        key: &Key,
        expected: u64,
        pose: ResourceTransform,
    ) -> Result<FragmentId, HarvestError> {
        journal
            .validate(catalog)
            .map_err(|_| HarvestError::InvalidSource)?;
        let s = catalog
            .get(key)
            .filter(|s| s.method == Method::Detach)
            .ok_or(HarvestError::InvalidSource)?;
        if journal.revision(key) != expected {
            return Err(HarvestError::StaleRevision);
        }
        if journal.remaining(s) == 0. {
            return Err(HarvestError::Exhausted);
        }
        if self.carried_object().is_some() || self.attached_sphere_id().is_some() {
            return Err(HarvestError::Occupied);
        }
        if !s.carriable() {
            return Err(HarvestError::TooLarge);
        }
        let revision = expected.checked_add(1).ok_or(HarvestError::Exhausted)?;
        let material = RawMaterial::new(s.material, s.initial_mass_kg)
            .map_err(|_| HarvestError::InvalidSource)?;
        let ids = self
            .import_salvage(&[(s.deposit, material, pose)])
            .ok_or(HarvestError::Exhausted)?;
        let id = ids[0];
        self.fragments
            .get_mut(id)
            .expect("published identity")
            .set_natural_shape(s.shape);
        assert!(self.pick_up(id), "preflight reserved empty carry slot");
        journal.write(Delta {
            key: key.clone(),
            revision,
            remaining_kg: 0.,
            detached: Some(id),
        });
        Ok(id)
    }
    /// Existing spherical lifecycle owns capacity, rate, release and allocation.
    pub fn mine_biological(
        &mut self,
        catalog: &Catalog,
        journal: &mut Journal,
        claim: (&Key, u64),
        delta: std::time::Duration,
        config: crate::spherical_mining::MiningConfig,
        pose: ResourceTransform,
    ) -> Result<f64, HarvestError> {
        let (key, expected) = claim;
        journal
            .validate(catalog)
            .map_err(|_| HarvestError::InvalidSource)?;
        let s = catalog
            .get(key)
            .filter(|s| s.method == Method::Embedded)
            .ok_or(HarvestError::InvalidSource)?;
        if journal.revision(key) != expected {
            return Err(HarvestError::StaleRevision);
        }
        if self.carried_object().is_some() {
            return Err(HarvestError::Occupied);
        }
        if self.attached_sphere_id().is_none() && !self.fragments.can_allocate() {
            return Err(HarvestError::Exhausted);
        }
        let remaining = journal.remaining(s);
        if remaining <= 0. {
            self.interrupt_spherical();
            return Err(HarvestError::Exhausted);
        }
        let revision = expected.checked_add(1).ok_or(HarvestError::Exhausted)?;
        let mut deposit = ResourceDeposit::new(
            s.deposit,
            RawMaterial::new(s.material, s.initial_mass_kg)
                .map_err(|_| HarvestError::InvalidSource)?,
            WorldPosition::new(s.center[0], s.center[1], s.center[2]),
            remaining,
        )
        .map_err(|_| HarvestError::InvalidSource)?;
        // A saved or streamed stale source cannot restore already extracted material.
        if self
            .remaining
            .get(&s.deposit)
            .is_some_and(|m| *m != remaining)
        {
            return Err(HarvestError::InvalidSource);
        }
        let before = self.extracted_mass_kg();
        self.tick_spherical(true, Some(&mut deposit), delta, config, pose);
        let added = self.extracted_mass_kg() - before;
        if added > 0. {
            journal.write(Delta {
                key: key.clone(),
                revision,
                remaining_kg: deposit.remaining_mass_kg(),
                detached: None,
            });
        }
        Ok(added)
    }
}
