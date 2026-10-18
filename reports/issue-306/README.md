# #306 — Fragment scratch and semantic mutation work

The runtime retains snapshot/state/ID buffers and moves selected states into the
solver instead of cloning/reallocating populations. The solver retains six
per-body arrays across substeps and stateful updates. This is capacity reuse,
not a claim that the entire contact/activation pipeline is allocation-free.

Exact identical validated carried/loose poses no longer journal redundant
updates. Pickup/drop still emit semantic events. Velocity, wake/growth, Core
removal and ship-local anchor updates keep their routes. Supporting-frame motion
still writes absolute poses. A changed containment frame reclassifies retained
centers once, including stationary unanchored objects crossing a moving ship;
repeated stationary redraws/ticks avoid both journal and mass work. This required
correctness work is explicitly O(retained count) on changing frames.

Regressions exercise exact fresh/reused-array parity through growing/shrinking
populations, scratch capacity retention, a settled tick with zero journal/mass
visits, moving anchored contents, stationary world boundary crossing and removal.
Existing interpolation, growth, carrying, streaming, Core and sleep/wake tests
remain the portable behavior gates. The benchmark independently replays the
solver from the same snapshots and checks velocities and position parity.

Portable validation and corrected-fixture before/after CPU/allocation evidence
are being collected on GitHub Ubuntu; the editing container has no Rust toolchain.
Native Apple M1 visuals/motion/FPS remain pending in
[#313](https://github.com/mr-exception/salimon/issues/313), not claimed here.
