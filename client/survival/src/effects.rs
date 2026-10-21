//! Data definitions are numeric; all effect behavior remains Rust-owned.
use crate::{Points, SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, time::Duration};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Source {
    Starvation,
    Radiation,
    Burning,
    Fall,
    Environment,
    Registered(u16),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Defense {
    pub flat: Points,
    pub resistance_bps: u16,
}
impl Default for Defense {
    fn default() -> Self {
        Self {
            flat: Points::ZERO,
            resistance_bps: 0,
        }
    }
}
impl Validate for Defense {
    fn validate(&self) -> Result<(), SaveError> {
        self.flat.validate()?;
        if self.resistance_bps > 10_000 {
            return Err(invalid("resistance above 100 percent"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRule {
    pub source: Source,
    pub defense: Defense,
    pub amplification_bps: u16,
    pub damage_cap: Points,
    pub death_priority: u16,
}
impl SourceRule {
    pub fn unmodified(source: Source, death_priority: u16) -> Self {
        Self {
            source,
            defense: Defense::default(),
            amplification_bps: 0,
            damage_cap: Points::FULL,
            death_priority,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Healing {
    Ordinary,
    Treatment,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Damage {
    pub source: Source,
    pub amount: Points,
    pub wound: bool,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Heal {
    pub amount: Points,
    pub mode: Healing,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(pub u16);
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Modifier {
    FatigueDrain,
    Regeneration,
    FatigueRelief,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Behavior {
    Modifier {
        kind: Modifier,
        basis_points: u16,
    },
    Feed {
        amount: Points,
    },
    Damage {
        source: Source,
        amount: Points,
        wound: bool,
    },
    Heal {
        amount: Points,
        mode: Healing,
    },
    Amplify {
        source: Source,
        basis_points: u16,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Lifetime {
    Timed(Duration),
    Persistent,
    /// Radiation uses zero linger; burns may linger after the last exposure ends.
    Exposure {
        linger: Duration,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Removal {
    Dispellable,
    TreatmentOnly,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRule {
    pub id: EffectId,
    pub behavior: Behavior,
    pub lifetime: Lifetime,
    /// Required for damage/healing; amplification is continuous and has none.
    pub interval: Option<Duration>,
    pub max_stacks: u16,
    pub removal: Removal,
}
/// Observational view of a registered active effect; cannot mutate authority.
#[derive(Clone, Copy, Debug)]
pub struct EffectSnapshot<'a> {
    pub rule: &'a EffectRule,
    pub stacks: u16,
    pub remaining: Option<Duration>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    pub sources: Vec<SourceRule>,
    pub effects: Vec<EffectRule>,
}
impl Default for Rules {
    fn default() -> Self {
        Self {
            sources: [
                Source::Starvation,
                Source::Radiation,
                Source::Burning,
                Source::Fall,
                Source::Environment,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, s)| SourceRule::unmodified(s, i as u16))
            .collect(),
            effects: Vec::new(),
        }
    }
}
impl Rules {
    pub(crate) fn source(&self, source: Source) -> Option<&SourceRule> {
        self.sources.iter().find(|r| r.source == source)
    }
    pub(crate) fn effect(&self, id: EffectId) -> Option<&EffectRule> {
        self.effects.iter().find(|r| r.id == id)
    }
}
impl Validate for Rules {
    fn validate(&self) -> Result<(), SaveError> {
        if self.sources.len() > 256 || self.effects.len() > 256 {
            return Err(invalid("survival registry limit"));
        }
        let mut sources = std::collections::BTreeSet::new();
        for rule in &self.sources {
            if !sources.insert(rule.source) {
                return Err(invalid("duplicate damage source"));
            }
            rule.defense.validate()?;
            rule.damage_cap.validate()?;
            if rule.amplification_bps > 50_000 {
                return Err(invalid("source amplification limit"));
            }
        }
        for source in [Source::Starvation, Source::Environment] {
            if !sources.contains(&source) {
                return Err(invalid("missing built-in damage source"));
            }
        }
        let mut ids = std::collections::BTreeSet::new();
        for rule in &self.effects {
            if !ids.insert(rule.id) || rule.max_stacks == 0 || rule.max_stacks > 100 {
                return Err(invalid("effect identity/stack limit"));
            }
            match rule.lifetime {
                Lifetime::Timed(d) if d.is_zero() || d > Duration::from_secs(86400) => {
                    return Err(invalid("effect duration"));
                }
                Lifetime::Exposure { linger } if linger > Duration::from_secs(86400) => {
                    return Err(invalid("effect linger"));
                }
                _ => {}
            }
            match &rule.behavior {
                Behavior::Modifier { kind, basis_points } => {
                    let valid = match kind {
                        Modifier::FatigueDrain => *basis_points <= 10_000,
                        Modifier::Regeneration => (10_000..=50_000).contains(basis_points),
                        Modifier::FatigueRelief => *basis_points <= 10_000,
                    };
                    if !valid || rule.interval.is_some() || rule.max_stacks != 1 {
                        return Err(invalid("invalid continuous food modifier"));
                    }
                }
                Behavior::Damage { source, amount, .. } => {
                    amount.validate()?;
                    if !sources.contains(source) {
                        return Err(invalid("unregistered effect source"));
                    }
                }
                Behavior::Feed { amount } | Behavior::Heal { amount, .. } => amount.validate()?,
                Behavior::Amplify {
                    source,
                    basis_points,
                } => {
                    if !sources.contains(source)
                        || *basis_points > 50_000
                        || rule.interval.is_some()
                    {
                        return Err(invalid("amplifier definition"));
                    }
                }
            }
            if !matches!(
                rule.behavior,
                Behavior::Amplify { .. } | Behavior::Modifier { .. }
            ) && !matches!(rule.interval, Some(d) if d >= Duration::from_millis(1) && d <= Duration::from_secs(86400))
            {
                return Err(invalid("periodic effect interval outside [1ms,1day]"));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Active {
    pub(crate) stacks: u16,
    pub(crate) remaining: Option<Duration>,
    pub(crate) tick: Duration,
    pub(crate) exposed: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Pipeline {
    pub(crate) rules: Rules,
    pub(crate) armor: Defense,
    pub(crate) wounds: Points,
    pub(crate) regeneration: Duration,
    pub(crate) active: BTreeMap<EffectId, Active>,
    pub(crate) death_cause: Option<Source>,
}
impl Default for Pipeline {
    fn default() -> Self {
        Self {
            rules: Rules::default(),
            armor: Defense::default(),
            wounds: Points::ZERO,
            regeneration: Duration::ZERO,
            active: BTreeMap::new(),
            death_cause: None,
        }
    }
}
impl Validate for Pipeline {
    fn validate(&self) -> Result<(), SaveError> {
        self.rules.validate()?;
        self.armor.validate()?;
        self.wounds.validate()?;
        if self.regeneration >= Duration::from_secs(60) {
            return Err(invalid("regen accumulator"));
        }
        if self
            .death_cause
            .is_some_and(|s| self.rules.source(s).is_none())
        {
            return Err(invalid("death source"));
        }
        for (id, active) in &self.active {
            let rule = self
                .rules
                .effect(*id)
                .ok_or_else(|| invalid("unknown active effect"))?;
            if active.stacks == 0
                || active.stacks > rule.max_stacks
                || rule.interval.is_some_and(|d| active.tick >= d)
                || (rule.interval.is_none() && !active.tick.is_zero())
            {
                return Err(invalid("active effect stacks/tick"));
            }
            match rule.lifetime {
                Lifetime::Timed(d)
                    if !active.exposed
                        && matches!(active.remaining, Some(r) if !r.is_zero() && r <= d) => {}
                Lifetime::Persistent if !active.exposed && active.remaining.is_none() => {}
                Lifetime::Exposure { linger }
                    if (active.exposed && active.remaining.is_none())
                        || (!active.exposed
                            && matches!(active.remaining, Some(r) if !r.is_zero() && r <= linger)) =>
                    {}
                _ => return Err(invalid("active effect lifetime")),
            }
        }
        Ok(())
    }
}
