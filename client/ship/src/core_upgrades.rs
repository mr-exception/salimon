//! Instant direct-material Core exception. No timers, power debit or replacement Core.
use crate::ShipController;
use salimon_persistence::{SaveError, Validate, invalid};
use salimon_world::{
    mining::MiningSession,
    resources::{ResourceFragment, ResourceId},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    pub level: u8,
    pub capacity_joules: u64,
    pub iron_milligrams: u64,
    pub silicate_milligrams: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum InputMode {
    #[default]
    Fuel,
    Upgrade,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applied {
    pub tier: Tier,
    pub receipts: Vec<ResourceFragment>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreUpgrades {
    pub mode: InputMode,
    pub applied: Vec<Applied>,
    pub pending: Vec<ResourceFragment>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeResult {
    Accepted,
    Applied,
    Recovered,
    NoItem,
    WrongMaterial,
    Oversized,
    NotReady,
    Maximum,
    Occupied,
    Invalid,
    StorageFull,
}
/// Integer-milligram credit with exact original receipts. Reject sub-milligram
/// quantities rather than rounding away physical material; 1e-9 kg admits f64 noise only.
fn milligrams(f: ResourceFragment) -> Option<u64> {
    let kg = f.material().mass_kg();
    let mg = (kg * 1_000_000.).round();
    ((1. ..=1_000_000_000.).contains(&mg) && (kg - mg / 1_000_000.).abs() <= 1e-9)
        .then_some(mg as u64)
}
fn totals(receipts: &[ResourceFragment]) -> Option<[u64; 2]> {
    let mut out = [0u64; 2];
    for f in receipts {
        let index = match f.material().resource() {
            ResourceId::IronOre => 0,
            ResourceId::SilicateRock => 1,
            _ => return None,
        };
        out[index] = out[index].checked_add(milligrams(*f)?)?;
    }
    Some(out)
}
pub fn validate_catalog(tiers: &[Tier]) -> Result<(), SaveError> {
    let mut previous = crate::DEFAULT_CORE_ENERGY_CAPACITY_JOULES;
    if tiers.is_empty() || tiers.len() > 8 {
        return Err(invalid("Core upgrade catalog size"));
    }
    for (i, t) in tiers.iter().enumerate() {
        if usize::from(t.level) != i + 1
            || t.capacity_joules <= previous
            || t.capacity_joules > 1_000_000_000_000
            || t.iron_milligrams == 0
            || t.silicate_milligrams == 0
            || t.iron_milligrams > 1_000_000_000
            || t.silicate_milligrams > 1_000_000_000
        {
            return Err(invalid("Core upgrade catalog identity/capacity/cost"));
        }
        previous = t.capacity_joules;
    }
    Ok(())
}
impl Validate for CoreUpgrades {
    fn validate(&self) -> Result<(), SaveError> {
        if self.applied.len() > 8 || self.pending.len() > 128 {
            return Err(invalid("Core receipt budget"));
        }
        let mut ids = std::collections::HashSet::new();
        for receipts in self
            .applied
            .iter()
            .map(|a| a.receipts.as_slice())
            .chain(std::iter::once(self.pending.as_slice()))
        {
            if receipts.len() > 128 || totals(receipts).is_none() {
                return Err(invalid("Core material receipts"));
            }
            for f in receipts {
                if f.id().0 == 0 || !ids.insert(f.id()) {
                    return Err(invalid("Core duplicate receipt identity"));
                }
                let m = salimon_world::resources::RawMaterial::new(
                    f.material().resource(),
                    f.material().mass_kg(),
                )
                .map_err(|_| invalid("Core receipt mass"))?;
                if m.volume_m3() != f.material().volume_m3() {
                    return Err(invalid("Core receipt volume"));
                }
                salimon_world::resources::ResourceTransform::new(
                    f.transform().position(),
                    f.transform().orientation_xyzw(),
                )
                .map_err(|_| invalid("Core receipt pose"))?;
            }
        }
        Ok(())
    }
}
impl CoreUpgrades {
    pub fn next<'a>(&self, catalog: &'a [Tier]) -> Option<&'a Tier> {
        catalog.get(self.applied.len())
    }
    pub fn remaining(&self, catalog: &[Tier]) -> Option<[u64; 2]> {
        let t = self.next(catalog)?;
        let n = totals(&self.pending)?;
        Some([
            t.iron_milligrams.checked_sub(n[0])?,
            t.silicate_milligrams.checked_sub(n[1])?,
        ])
    }
    pub fn mass_kg(&self) -> f64 {
        self.pending
            .iter()
            .chain(self.applied.iter().flat_map(|a| a.receipts.iter()))
            .map(|f| f.material().mass_kg())
            .sum()
    }
    pub fn validate_context(
        &self,
        catalog: &[Tier],
        world: &MiningSession,
        ship: &ShipController,
    ) -> Result<(), SaveError> {
        self.validate()?;
        validate_catalog(catalog)?;
        for (i, a) in self.applied.iter().enumerate() {
            if catalog.get(i) != Some(&a.tier)
                || totals(&a.receipts) != Some([a.tier.iron_milligrams, a.tier.silicate_milligrams])
            {
                return Err(invalid("Core installed tier/material audit"));
            }
        }
        if !self.pending.is_empty() && self.remaining(catalog).is_none() {
            return Err(invalid("Core pending cost exceeded"));
        }
        let next_id = world.save_state().next_id;
        for f in self
            .pending
            .iter()
            .chain(self.applied.iter().flat_map(|a| a.receipts.iter()))
        {
            if f.id().0 > next_id || world.fragment(f.id()).is_some() {
                return Err(invalid(
                    "Core receipt duplicates world or exceeds allocator",
                ));
            }
        }
        if self
            .applied
            .last()
            .is_some_and(|a| ship.energy_config().capacity_joules != a.tier.capacity_joules)
        {
            return Err(invalid("Core installed capacity/source mismatch"));
        }
        Ok(())
    }
    pub fn contribute(&mut self, world: &mut MiningSession, catalog: &[Tier]) -> UpgradeResult {
        if self.mode != InputMode::Upgrade {
            return UpgradeResult::Invalid;
        }
        let Some(left) = self.remaining(catalog) else {
            return UpgradeResult::Maximum;
        };
        if self.pending.len() >= 128 {
            return UpgradeResult::StorageFull;
        }
        let Some(id) = world.carried_id() else {
            return UpgradeResult::NoItem;
        };
        let f = *world
            .fragment(id)
            .expect("world carry references valid source");
        let index = match f.material().resource() {
            ResourceId::IronOre => 0,
            ResourceId::SilicateRock => 1,
            _ => return UpgradeResult::WrongMaterial,
        };
        let Some(mg) = milligrams(f) else {
            return UpgradeResult::Invalid;
        };
        if mg > left[index] {
            return UpgradeResult::Oversized;
        }
        if !world.remove_fragment(id) {
            return UpgradeResult::Invalid;
        }
        self.pending.push(f);
        UpgradeResult::Accepted
    }
    pub fn recover(&mut self, world: &mut MiningSession) -> UpgradeResult {
        if world.carried_object().is_some() {
            return UpgradeResult::Occupied;
        }
        let Some(f) = self.pending.last().copied() else {
            return UpgradeResult::NoItem;
        };
        if !world.return_reserved_fragment(f) {
            return UpgradeResult::Invalid;
        }
        self.pending.pop();
        UpgradeResult::Recovered
    }
    pub fn apply(
        &mut self,
        ship: &mut ShipController,
        world: &MiningSession,
        catalog: &[Tier],
    ) -> UpgradeResult {
        if self.mode != InputMode::Upgrade || self.validate_context(catalog, world, ship).is_err() {
            return UpgradeResult::Invalid;
        }
        let Some(t) = self.next(catalog).cloned() else {
            return UpgradeResult::Maximum;
        };
        if self.remaining(catalog) != Some([0, 0]) {
            return UpgradeResult::NotReady;
        }
        let old = self
            .applied
            .last()
            .map_or(crate::DEFAULT_CORE_ENERGY_CAPACITY_JOULES, |a| {
                a.tier.capacity_joules
            });
        if ship.increase_core_capacity(old, t.capacity_joules).is_err() {
            return UpgradeResult::Invalid;
        }
        self.applied.push(Applied {
            tier: t,
            receipts: std::mem::take(&mut self.pending),
        });
        UpgradeResult::Applied
    }
}
