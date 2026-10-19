# Survival invariants

- All three authoritative meters remain in 0..100. Invalid decoded state must
  pass `Validate` before publication; unknown fields reject.
- No floating point accumulation, frame counts, wall clock calls or IO.
- Equal admitted durations and commands give identical state, including fractional
  starvation and sub-point drain. Starvation starts after hunger reaches zero.
- Health zero is terminal. Future commands/time/offline recovery cannot revive it.
- Only fatigue changes offline; health, hunger and active duration remain frozen.
- Launch recovery uses a persistent UTC high watermark and one transient receipt.
  The host commits credited state before exposure; see architecture for crash policy.
- Full elapsed admission and authoritative world death freeze are composed by #371,
  not duplicated inside this standalone domain.
- Starvation, external damage and periodic effects share one modifier pipeline.
- Damage at a timestamp precedes all healing; terminal damage cannot be healed.
- Wound debt is at most missing health and ordinary regeneration cannot erase it.
- Stable Source/EffectId ordering and bounded stacks are independent of host order.
- Timed final ticks occur before expiry; exposure exits with zero linger remove
  immediately. Offline time advances no regeneration/effect interval.
- #369 threshold intervals, multiplicative rates and typed permission policy have
  one owner in `restrictions.rs`; see the architecture contract. Zero fatigue
  cannot deny doors, basic menus or food/medicine solely because of exhaustion.
- Reduced interaction work never reduces ship, physics, Core or machine-job time.
  Derived percentages/cooldowns do not substitute for #371's saved vital state.
