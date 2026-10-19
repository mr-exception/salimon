//! Schema-1 sparse world source state. Derived indexes and mutation journals rebuild.
use super::MiningSession;
use crate::resources::{DepositId, FragmentId, RawMaterial, ResourceFragment, ResourceTransform};
use salimon_persistence::{SaveError, Validate, invalid};
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MiningSaveState {
    pub remaining: Vec<(DepositId, f64)>,
    pub extracted_mass_kg: f64,
    pub fragments: Vec<ResourceFragment>,
    pub next_id: u64,
    pub sealed: Vec<FragmentId>,
    pub carried: Option<FragmentId>,
}
impl Validate for MiningSaveState {
    fn validate(&self) -> Result<(), SaveError> {
        let mut deposits = std::collections::HashSet::new();
        if !self.extracted_mass_kg.is_finite() || self.extracted_mass_kg < 0. {
            return Err(invalid("extracted mass"));
        }
        for (id, m) in &self.remaining {
            if !deposits.insert(*id) || !m.is_finite() || *m < 0. {
                return Err(invalid("depletion journal"));
            }
        }
        let mut ids = std::collections::HashSet::new();
        for f in &self.fragments {
            if f.id().0 == 0 || f.id().0 > self.next_id || !ids.insert(f.id()) {
                return Err(invalid("fragment IDs/allocator"));
            }
            let material = RawMaterial::new(f.material().resource(), f.material().mass_kg())
                .map_err(|_| invalid("fragment material"))?;
            if material.volume_m3() != f.material().volume_m3() {
                return Err(invalid("fragment density/volume"));
            }
            ResourceTransform::new(f.transform().position(), f.transform().orientation_xyzw())
                .map_err(|_| invalid("fragment pose"))?;
        }
        let mut sealed = std::collections::HashSet::new();
        if self
            .sealed
            .iter()
            .any(|id| !ids.contains(id) || !sealed.insert(*id))
            || self.carried.is_some_and(|id| !ids.contains(&id))
        {
            return Err(invalid("sealed/carried reference"));
        }
        Ok(())
    }
}
impl MiningSession {
    pub fn save_state(&self) -> MiningSaveState {
        let (fragments, next_id, sealed) = self.fragments.save_parts();
        let mut remaining: Vec<_> = self.remaining.iter().map(|(k, v)| (*k, *v)).collect();
        // Stable body/catalog order and local identity, independent of hash seed.
        remaining.sort_by_key(|(id, _)| {
            (
                crate::CELESTIAL_BODIES.iter().position(|b| b.id == id.body),
                id.local,
            )
        });
        MiningSaveState {
            remaining,
            extracted_mass_kg: self.extracted_mass_kg,
            fragments,
            next_id,
            sealed,
            carried: self.carried_id(),
        }
    }
    pub fn from_save(state: MiningSaveState) -> Result<Self, SaveError> {
        state.validate()?;
        let mut session = Self {
            remaining: state.remaining.into_iter().collect(),
            extracted_mass_kg: state.extracted_mass_kg,
            fragments: crate::resource_fragments::FragmentOutput::from_save_parts(
                state.fragments,
                state.next_id,
                state.sealed,
            ),
            ..Self::default()
        };
        if let Some(id) = state.carried {
            session
                .carry
                .pick_up(crate::carrying::WorldObjectId::ResourceFragment(id));
        }
        Ok(session)
    }
}
