//! Bind the portable lifecycle to stable physical identities and streaming deltas.
use super::MiningSession;
use crate::resources::{FragmentId, ResourceDeposit, ResourceTransform};
use crate::spherical_mining::MiningConfig;
use std::time::Duration;

impl MiningSession {
    pub const fn attached_sphere_id(&self) -> Option<FragmentId> {
        self.attached_sphere_id
    }

    /// Detach immediately, retaining its physical mass/pose/identity in the session.
    pub fn interrupt_spherical(&mut self) -> Option<FragmentId> {
        self.spherical.interrupt();
        self.attached_sphere_id.take()
    }

    /// Advance a single bounded tick. Returns a newly released physical identity.
    /// Attached mass is already represented by a physical fragment, but cannot be
    /// picked up or moved by loose physics before release. No mass is duplicated.
    pub fn tick_spherical(
        &mut self,
        held: bool,
        mut target: Option<&mut ResourceDeposit>,
        delta: Duration,
        config: MiningConfig,
        attachment: ResourceTransform,
    ) -> Option<FragmentId> {
        if let Some(source) = target.as_deref_mut() {
            self.restore(source);
        }
        let result = self
            .spherical
            .tick(held, target.as_deref_mut(), delta, config);
        if result.added_kg > 0.0 {
            let sphere = self
                .spherical
                .attached()
                .or(result.released.map(|r| r.sphere))
                .expect("positive extraction has attached or released material");
            self.attached_sphere_id = Some(self.fragments.upsert_sphere(
                self.attached_sphere_id,
                sphere,
                attachment,
            ));
            if let Some(source) = target {
                self.remaining
                    .insert(source.id(), source.remaining_mass_kg());
            }
            self.extracted_mass_kg += result.added_kg;
        } else if result.released.is_none()
            && let (Some(id), Some(sphere)) = (self.attached_sphere_id, self.spherical.attached())
        {
            self.fragments.upsert_sphere(Some(id), sphere, attachment);
        }
        if result.released.is_some() {
            self.attached_sphere_id.take()
        } else {
            None
        }
    }
}
