# #304 — Elapsed-time fragment damping

Linear/angular retention now uses `0.995^(dt/0.008)` / `0.98^(dt/0.008)`.
The reference is the ordinary 16 ms tick's two 8 ms subdivisions: normal
motion retains its existing loss, while small/fast unrelated objects no longer
multiply the loss by imposing additional steps. Unpowered floors remain undamped.
Factors are evaluated once per substep, outside the body loop. Contact, rest,
sleep and cutoff thresholds are unchanged.

Regressions cover a realistic 10 kg iron sphere, 1/4/8/16/40/160 ms partitions,
restarting calls, and a distant 5 mm fast residual that changes scheduling by
more than 10× without changing the subject's linear/angular retention. The
2e-14 velocity tolerance covers repeated floating-point exponential products;
semi-implicit position/gravity integration is not claimed partition invariant.
Existing contact/sleep/conservation/zero-g regressions remain required.

Local Rust checks could not run: this Linux container has no cargo/rustc and
cannot reach the Rust distribution host. GitHub Ubuntu verification of the
complete stack, including this change, is recorded in the #306 report.
`git diff --check` passed. Native M1 motion/performance/visual verification remains
pending in [#313](https://github.com/mr-exception/salimon/issues/313).
