use super::{effects::*, *};
const REGEN_INTERVAL: Duration = Duration::from_secs(60);

impl Survival {
    pub fn with_rules(config: Config, rules: Rules) -> Result<Self, SaveError> {
        rules.validate()?;
        let mut state = Self::new(config)?;
        state.pipeline.rules = rules;
        Ok(state)
    }
    pub fn set_armor(&mut self, armor: Defense) -> Result<(), SaveError> {
        armor.validate()?;
        if !self.terminal {
            self.pipeline.armor = armor;
        }
        Ok(())
    }
    pub fn active_effects(&self) -> impl Iterator<Item = (EffectId, u16, Option<Duration>)> + '_ {
        self.pipeline
            .active
            .iter()
            .map(|(id, a)| (*id, a.stacks, a.remaining))
    }
    pub fn apply_effect(&mut self, id: EffectId) -> Result<(), SaveError> {
        let rule = self
            .pipeline
            .rules
            .effect(id)
            .ok_or_else(|| invalid("unknown effect"))?;
        if self.terminal {
            return Ok(());
        }
        let remaining = match rule.lifetime {
            Lifetime::Timed(d) => Some(d),
            _ => None,
        };
        let exposed = matches!(rule.lifetime, Lifetime::Exposure { .. });
        self.pipeline
            .active
            .entry(id)
            .and_modify(|a| {
                a.stacks = a.stacks.saturating_add(1).min(rule.max_stacks);
                a.remaining = remaining;
                a.exposed = exposed;
            })
            .or_insert(Active {
                stacks: 1,
                remaining,
                tick: Duration::ZERO,
                exposed,
            });
        Ok(())
    }
    /// Exposure updates are idempotent. Additional zones must be aggregated by
    /// the host before calling; entering one zone does not add a stack per frame.
    pub fn set_exposure(&mut self, id: EffectId, exposed: bool) -> Result<(), SaveError> {
        let rule = self
            .pipeline
            .rules
            .effect(id)
            .ok_or_else(|| invalid("unknown exposure"))?;
        let Lifetime::Exposure { linger } = rule.lifetime else {
            return Err(invalid("effect is not exposure-bound"));
        };
        if self.terminal {
            return Ok(());
        }
        if exposed {
            if !self.pipeline.active.get(&id).is_some_and(|a| a.exposed) {
                self.apply_effect(id)?;
            }
        } else if linger.is_zero() {
            self.pipeline.active.remove(&id);
        } else if let Some(a) = self.pipeline.active.get_mut(&id)
            && a.exposed
        {
            a.exposed = false;
            a.remaining = Some(linger);
        }
        Ok(())
    }
    pub fn remove_effect(&mut self, id: EffectId, treatment: bool) -> Result<bool, SaveError> {
        let rule = self
            .pipeline
            .rules
            .effect(id)
            .ok_or_else(|| invalid("unknown effect"))?;
        if self.terminal || (rule.removal == Removal::TreatmentOnly && !treatment) {
            return Ok(false);
        }
        Ok(self.pipeline.active.remove(&id).is_some())
    }
    pub fn heal(&mut self, event: Heal) -> Result<(), SaveError> {
        event.amount.validate()?;
        if self.terminal {
            return Ok(());
        }
        if event.mode == Healing::Treatment {
            self.pipeline.wounds.subtract(event.amount);
        }
        self.health.add(event.amount);
        self.health.0 = self.health.0.min(FULL - self.pipeline.wounds.0);
        Ok(())
    }
    pub fn damage(&mut self, event: Damage) -> Result<(), SaveError> {
        event.amount.validate()?;
        if self.pipeline.rules.source(event.source).is_none() {
            return Err(invalid("unknown damage source"));
        }
        if !self.terminal {
            self.damage_batch(&[event]);
        }
        Ok(())
    }
    fn modified_damage(&self, event: Damage) -> Points {
        // The source was validated at command/registry admission.
        let rule = self
            .pipeline
            .rules
            .source(event.source)
            .expect("registered damage source");
        let mut amount = event
            .amount
            .0
            .saturating_sub(rule.defense.flat.0)
            .saturating_sub(self.pipeline.armor.flat.0);
        for resistance in [
            rule.defense.resistance_bps,
            self.pipeline.armor.resistance_bps,
        ] {
            amount = ((u128::from(amount) * u128::from(10_000 - resistance)) / 10_000) as u64;
        }
        let amplification: u64 = self
            .pipeline
            .active
            .iter()
            .filter_map(|(id, a)| {
                match self
                    .pipeline
                    .rules
                    .effect(*id)
                    .expect("registered active effect")
                    .behavior
                {
                    Behavior::Amplify {
                        source,
                        basis_points,
                    } if source == event.source => {
                        Some(u64::from(basis_points) * u64::from(a.stacks))
                    }
                    _ => None,
                }
            })
            .sum();
        amount = ((u128::from(amount)
            * u128::from(10_000 + u64::from(rule.amplification_bps) + amplification))
            / 10_000)
            .min(u128::from(rule.damage_cap.0)) as u64;
        Points(amount)
    }
    fn damage_batch(&mut self, events: &[Damage]) {
        let mut cause = None;
        for &event in events {
            let amount = self.modified_damage(event);
            if amount == Points::ZERO {
                continue;
            }
            let priority = self
                .pipeline
                .rules
                .source(event.source)
                .expect("registered source")
                .death_priority;
            cause = cause.max(Some((priority, event.source)));
            let lost = amount.0.min(self.health.0);
            self.health.subtract(amount);
            if event.wound {
                self.pipeline.wounds.0 = (self.pipeline.wounds.0 + lost).min(FULL);
            }
        }
        if self.health == Points::ZERO {
            self.terminal = true;
            self.pipeline.death_cause = cause.map(|(_, s)| s);
        }
    }
    fn regen_eligible(&self) -> bool {
        self.hunger.0 >= 70 * POINT && self.fatigue.0 >= 50 * POINT
    }
    pub(crate) fn advance_pipeline(&mut self, elapsed: Duration) {
        let mut remaining = elapsed.as_nanos();
        while remaining > 0 && !self.terminal {
            let hungry = self.hunger == Points::ZERO;
            let regen = self.regen_eligible();
            if !regen {
                self.pipeline.regeneration = Duration::ZERO;
            }
            let mut step = remaining;
            let hunger_rate = u128::from(SCALE_SECONDS / self.config.hunger_full_seconds);
            let fatigue_rate = u128::from(SCALE_SECONDS / self.config.fatigue_full_seconds);
            if !hungry {
                step = step.min(u128::from(self.hunger.0).div_ceil(hunger_rate));
            }
            if hungry {
                step = step
                    .min((self.config.starvation_interval - self.starvation_elapsed).as_nanos());
            }
            if regen {
                step = step.min((REGEN_INTERVAL - self.pipeline.regeneration).as_nanos());
                // Cross the inclusive eligibility boundary at the first admitted
                // nanosecond below it, never by one render or arbitrary tick.
                step = step.min(u128::from(self.hunger.0 - 70 * POINT) / hunger_rate + 1);
                step = step.min(u128::from(self.fatigue.0 - 50 * POINT) / fatigue_rate + 1);
            }
            for (id, a) in &self.pipeline.active {
                let rule = self
                    .pipeline
                    .rules
                    .effect(*id)
                    .expect("registered active effect");
                if let Some(d) = a.remaining {
                    step = step.min(d.as_nanos());
                }
                if let Some(interval) = rule.interval {
                    step = step.min((interval - a.tick).as_nanos());
                }
            }
            let delta = nanos(step);
            self.drain_meters(step);
            remaining -= step;
            let mut damages = Vec::new();
            let mut heals = Vec::new();
            let mut feeds = Vec::new();
            if hungry {
                self.starvation_elapsed += delta;
                if self.starvation_elapsed == self.config.starvation_interval {
                    self.starvation_elapsed = Duration::ZERO;
                    damages.push(Damage {
                        source: Source::Starvation,
                        amount: Points(POINT),
                        wound: false,
                    });
                }
            }
            if regen {
                self.pipeline.regeneration += delta;
            }
            for (id, a) in &mut self.pipeline.active {
                let rule = self.pipeline.rules.effect(*id).expect("registered effect");
                if let Some(d) = &mut a.remaining {
                    *d = d.saturating_sub(delta);
                }
                if let Some(interval) = rule.interval {
                    a.tick += delta;
                    if a.tick == interval {
                        a.tick = Duration::ZERO;
                        for _ in 0..a.stacks {
                            match rule.behavior {
                                Behavior::Damage {
                                    source,
                                    amount,
                                    wound,
                                } => damages.push(Damage {
                                    source,
                                    amount,
                                    wound,
                                }),
                                Behavior::Feed { amount } => feeds.push(amount),
                                Behavior::Heal { amount, mode } => {
                                    heals.push(Heal { amount, mode })
                                }
                                Behavior::Amplify { .. } => {
                                    unreachable!("amplifiers have no interval")
                                }
                            }
                        }
                    }
                }
            }
            // Damage wins ties with heals/regen; death is terminal for the whole
            // timestamp. Cause uses highest configured priority, then Source Ord.
            damages.sort_by_key(|d| d.source);
            self.damage_batch(&damages);
            if !self.terminal {
                for amount in feeds {
                    self.command(Command::Feed(amount))
                        .expect("validated periodic food");
                }
                for heal in heals {
                    self.heal(heal).expect("validated periodic healing");
                }
                if self.pipeline.regeneration == REGEN_INTERVAL {
                    self.pipeline.regeneration = Duration::ZERO;
                    self.heal(Heal {
                        amount: Points(POINT),
                        mode: Healing::Ordinary,
                    })
                    .expect("constant regeneration");
                }
            } else if self.pipeline.regeneration == REGEN_INTERVAL {
                self.pipeline.regeneration = Duration::ZERO;
            }
            // Final tick and amplification remain active at the expiry timestamp;
            // expiration occurs last, after all timestamp events.
            self.pipeline
                .active
                .retain(|_, a| a.remaining != Some(Duration::ZERO));
        }
    }
}
