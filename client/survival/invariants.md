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
