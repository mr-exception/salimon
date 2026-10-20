//! Validated item-specific recovery data. Item storage/transactions belong to the host.
use crate::{Command, Points, Survival, effects::*};
use salimon_persistence::{SaveError, Validate, invalid};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Category {
    Food,
    Medical,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Recovery {
    Hunger,
    Health(Healing),
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Benefit {
    Instant {
        recovery: Recovery,
        milli_points: u32,
    },
    OverTime {
        id: EffectId,
        recovery: Recovery,
        milli_points: u32,
        interval: Duration,
        duration: Duration,
        max_stacks: u16,
    },
}
impl Benefit {
    fn rule(&self) -> Option<EffectRule> {
        match *self {
            Self::OverTime {
                id,
                recovery,
                milli_points,
                interval,
                duration,
                max_stacks,
            } => {
                // Definition validation checks this numeric conversion before use.
                let amount = Points::milli(milli_points).ok()?;
                Some(EffectRule {
                    id,
                    behavior: match recovery {
                        Recovery::Hunger => Behavior::Feed { amount },
                        Recovery::Health(mode) => Behavior::Heal { amount, mode },
                    },
                    lifetime: Lifetime::Timed(duration),
                    interval: Some(interval),
                    max_stacks,
                    removal: Removal::Dispellable,
                })
            }
            Self::Instant { .. } => None,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub item_id: String,
    pub category: Category,
    pub benefits: Vec<Benefit>,
}
impl Validate for Definition {
    fn validate(&self) -> Result<(), SaveError> {
        let id = &self.item_id;
        if id.is_empty()
            || id.len() > 128
            || id.starts_with('-')
            || id.ends_with('-')
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || self.benefits.is_empty()
            || self.benefits.len() > 16
        {
            return Err(invalid("invalid consumable definition"));
        }
        for benefit in &self.benefits {
            let (milli_points, durations) = match benefit {
                Benefit::Instant { milli_points, .. } => (*milli_points, None),
                Benefit::OverTime {
                    milli_points,
                    interval,
                    duration,
                    ..
                } => (*milli_points, Some((*interval, *duration))),
            };
            Points::milli(milli_points)?;
            if milli_points == 0
                || durations.is_some_and(|(i, d)| i > d || d.as_nanos() % i.as_nanos().max(1) != 0)
            {
                return Err(invalid("empty or incomplete consumable recovery"));
            }
        }
        Rules {
            effects: self.benefits.iter().filter_map(Benefit::rule).collect(),
            ..Default::default()
        }
        .validate()
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Catalog(Vec<Definition>);
impl Catalog {
    pub fn new(definitions: Vec<Definition>) -> Result<Self, SaveError> {
        let value = Self(definitions);
        value.validate()?;
        Ok(value)
    }
    pub fn get(&self, item_id: &str) -> Option<&Definition> {
        self.0.iter().find(|d| d.item_id == item_id)
    }
    pub fn starter() -> Self {
        Self::new(vec![
            Definition {
                item_id: "food-ration".into(),
                category: Category::Food,
                benefits: vec![Benefit::Instant {
                    recovery: Recovery::Hunger,
                    milli_points: 40_000,
                }],
            },
            // Explicit provisional balancing: +25 ordinary health over 10 active seconds.
            Definition {
                item_id: "medkit".into(),
                category: Category::Medical,
                benefits: vec![Benefit::OverTime {
                    id: EffectId(10_000),
                    recovery: Recovery::Health(Healing::Ordinary),
                    milli_points: 2500,
                    interval: Duration::from_secs(1),
                    duration: Duration::from_secs(10),
                    max_stacks: 1,
                }],
            },
        ])
        .expect("validated starter consumable tuning")
    }
}
impl Validate for Catalog {
    fn validate(&self) -> Result<(), SaveError> {
        if self.0.len() > 256 {
            return Err(invalid("consumable catalog limit"));
        }
        let mut ids = std::collections::BTreeSet::new();
        let mut effects = Vec::new();
        for definition in &self.0 {
            definition.validate()?;
            if !ids.insert(&definition.item_id) {
                return Err(invalid("duplicate consumable item"));
            }
            effects.extend(definition.benefits.iter().filter_map(Benefit::rule));
        }
        // Effect IDs are globally unique across consumable definitions.
        Rules {
            effects,
            ..Default::default()
        }
        .validate()
    }
}
impl Survival {
    /// Prepare every effect on a clone before publishing any benefit. The host
    /// removes the selected physical item only after this transaction succeeds.
    pub fn apply_consumable(&mut self, definition: &Definition) -> Result<(), SaveError> {
        definition.validate()?;
        if self.snapshot().terminal {
            return Err(invalid("terminal run cannot consume"));
        }
        let mut next = self.clone();
        for rule in definition.benefits.iter().filter_map(Benefit::rule) {
            match next.pipeline.rules.effects.iter().find(|r| r.id == rule.id) {
                Some(existing) if *existing != rule => {
                    return Err(invalid("consumable effect ID conflict"));
                }
                Some(_) => {}
                None => next.pipeline.rules.effects.push(rule),
            }
        }
        next.pipeline.rules.validate()?;
        for benefit in &definition.benefits {
            match *benefit {
                Benefit::Instant {
                    recovery,
                    milli_points,
                } => {
                    let amount = Points::milli(milli_points)?;
                    match recovery {
                        Recovery::Hunger => next.command(Command::Feed(amount))?,
                        Recovery::Health(mode) => next.heal(Heal { amount, mode })?,
                    }
                }
                Benefit::OverTime { id, .. } => next.apply_effect(id)?,
            }
        }
        next.validate()?;
        *self = next;
        Ok(())
    }
}
