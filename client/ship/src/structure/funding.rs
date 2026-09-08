//! Exact project material ownership; timed work is supplied by the construction owner.
use super::placement::Requirement;
use salimon_world::{
    mining::MiningSession,
    resources::{ResourceFragment, ResourceId},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const COMPLETE: u32 = 1_000_000;
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Funding {
    pub receipts: Vec<ResourceFragment>,
    pub progress_ppm: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FundingError {
    Unapproved,
    NoItem,
    WrongMaterial,
    Oversized,
    Invalid,
    Complete,
    Occupied,
}
pub fn milligrams(f: ResourceFragment) -> Result<u64, FundingError> {
    let kg = f.material().mass_kg();
    let mg = (kg * 1e6).round();
    if !(1. ..=1_000_000_000.).contains(&mg) || (kg - mg / 1e6).abs() > 1e-9 {
        return Err(FundingError::Invalid);
    }
    Ok(mg as u64)
}
impl Funding {
    pub fn delivered(&self, resource: ResourceId) -> u64 {
        self.receipts
            .iter()
            .filter(|f| f.material().resource() == resource)
            .map(|f| milligrams(*f).expect("validated material receipt"))
            .sum()
    }
    pub fn validate(&self, requirements: &[Requirement]) -> Result<(), FundingError> {
        if self.receipts.len() > 128 || self.progress_ppm > COMPLETE {
            return Err(FundingError::Invalid);
        }
        let mut ids = BTreeSet::new();
        for f in &self.receipts {
            milligrams(*f)?;
            if f.id().0 == 0
                || !ids.insert(f.id().0)
                || !requirements
                    .iter()
                    .any(|r| r.resource == f.material().resource())
                || salimon_world::resources::RawMaterial::new(
                    f.material().resource(),
                    f.material().mass_kg(),
                )
                .map_err(|_| FundingError::Invalid)?
                    != f.material()
            {
                return Err(FundingError::Invalid);
            }
        }
        if requirements
            .iter()
            .any(|r| self.delivered(r.resource) > r.milligrams)
            || (self.progress_ppm > 0 && !self.funded(requirements))
        {
            return Err(FundingError::Invalid);
        }
        Ok(())
    }
    pub fn funded(&self, requirements: &[Requirement]) -> bool {
        requirements
            .iter()
            .all(|r| self.delivered(r.resource) == r.milligrams)
    }
    pub fn contribute(
        &mut self,
        world: &mut MiningSession,
        requirements: &[Requirement],
        approved: bool,
    ) -> Result<(), FundingError> {
        if !approved {
            return Err(FundingError::Unapproved);
        }
        self.validate(requirements)?;
        if self.progress_ppm != 0 || self.receipts.len() == 128 {
            return Err(FundingError::Invalid);
        }
        let id = world.carried_id().ok_or(FundingError::NoItem)?;
        let f = *world.fragment(id).ok_or(FundingError::Invalid)?;
        let r = requirements
            .iter()
            .find(|r| r.resource == f.material().resource())
            .ok_or(FundingError::WrongMaterial)?;
        if milligrams(f)? > r.milligrams - self.delivered(r.resource) {
            return Err(FundingError::Oversized);
        }
        if !world.remove_fragment(id) {
            return Err(FundingError::Invalid);
        }
        self.receipts.push(f);
        Ok(())
    }
    /// Caller must supply funded authoritative progress, never a UI percentage.
    pub fn set_progress(
        &mut self,
        ppm: u32,
        requirements: &[Requirement],
    ) -> Result<(), FundingError> {
        if ppm < self.progress_ppm || ppm > COMPLETE || !self.funded(requirements) {
            return Err(FundingError::Invalid);
        }
        self.progress_ppm = ppm;
        Ok(())
    }
    /// Floor once per resource, then distribute by original receipt order. Integer
    /// rounding is consumed, never created; original IDs/source survive partial returns.
    pub fn refunds(
        &self,
        requirements: &[Requirement],
    ) -> Result<Vec<ResourceFragment>, FundingError> {
        self.validate(requirements)?;
        if self.progress_ppm == COMPLETE {
            return Err(FundingError::Complete);
        }
        let mut out = Vec::new();
        for r in requirements {
            let mut remaining = (u128::from(self.delivered(r.resource))
                * u128::from(COMPLETE - self.progress_ppm)
                / u128::from(COMPLETE)) as u64;
            for f in self
                .receipts
                .iter()
                .filter(|f| f.material().resource() == r.resource)
            {
                let mg = remaining.min(milligrams(*f)?);
                if mg > 0 {
                    let material =
                        salimon_world::resources::RawMaterial::new(r.resource, mg as f64 / 1e6)
                            .map_err(|_| FundingError::Invalid)?;
                    out.push(if mg == milligrams(*f)? {
                        *f
                    } else {
                        ResourceFragment::new(f.id(), f.source(), material, f.transform())
                    });
                    remaining -= mg;
                }
            }
        }
        Ok(out)
    }
    pub fn mass_kg(&self) -> f64 {
        self.receipts.iter().map(|f| f.material().mass_kg()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::resources::*;
    fn funding() -> (Funding, Vec<Requirement>) {
        let requirements = vec![
            Requirement {
                resource: ResourceId::IronOre,
                milligrams: 101,
            },
            Requirement {
                resource: ResourceId::SilicateRock,
                milligrams: 203,
            },
        ];
        let receipts = [
            (1, ResourceId::IronOre, 51),
            (2, ResourceId::IronOre, 50),
            (3, ResourceId::SilicateRock, 203),
        ]
        .into_iter()
        .map(|(id, r, mg)| {
            ResourceFragment::new(
                FragmentId(id),
                DepositId {
                    body: salimon_world::CelestialBodyId::Earth,
                    local: 1,
                },
                RawMaterial::new(r, mg as f64 / 1e6).unwrap(),
                ResourceTransform::new(
                    salimon_world::WorldPosition::new(0., 0., 0.),
                    [0., 0., 0., 1.],
                )
                .unwrap(),
            )
        })
        .collect();
        (
            Funding {
                receipts,
                progress_ppm: 0,
            },
            requirements,
        )
    }
    #[test]
    fn resource_total_rounding_and_nonreused_original_receipts_at_all_boundaries() {
        for percent in [0, 25, 50, 75, 99, 100] {
            let (mut f, r) = funding();
            f.set_progress(percent * 10_000, &r).unwrap();
            if percent == 100 {
                assert_eq!(f.refunds(&r), Err(FundingError::Complete));
                continue;
            }
            let returned = f.refunds(&r).unwrap();
            for req in &r {
                let mg: u64 = returned
                    .iter()
                    .filter(|p| p.material().resource() == req.resource)
                    .map(|p| milligrams(*p).unwrap())
                    .sum();
                assert_eq!(mg, req.milligrams * u64::from(100 - percent) / 100);
            }
            assert!(returned.iter().all(|p| {
                f.receipts
                    .iter()
                    .any(|old| old.id() == p.id() && old.source() == p.source())
            }));
        }
    }
    #[test]
    fn underfunded_refunds_all_and_invalid_or_backward_progress_is_atomic() {
        let (mut f, r) = funding();
        f.receipts.pop();
        assert_eq!(f.refunds(&r).unwrap(), f.receipts);
        assert_eq!(f.set_progress(1, &r), Err(FundingError::Invalid));
        f.receipts.push(f.receipts[0]);
        assert_eq!(f.validate(&r), Err(FundingError::Invalid));
        let (mut f, r) = funding();
        f.set_progress(500_000, &r).unwrap();
        assert_eq!(f.set_progress(250_000, &r), Err(FundingError::Invalid));
        assert_eq!(f.progress_ppm, 500_000);
    }
}
