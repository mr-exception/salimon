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

## Corrected-fixture CPU comparison

[Before raw populations and samples](before-timings.json.gz),
[after raw populations and samples](after-timings.json.gz),
[all phase medians](timing-medians.csv), and [machine/compiler/ref metadata](cpu-metadata.json).
The paired matrix used one AMD EPYC 7763 Ubuntu 24.04 runner, Rust 1.99.1 release,
20 cases (500/1,000 active × 0/10,000 inactive × five modes), five measured samples
per case. Both refs have #304 damping; before has linear identity lookup and
fresh arrays, after includes indexing/scratch/semantic mutation work.
[Timing run](https://github.com/mr-exception/salimon/actions/runs/39137894454).

The 1,000 active + 10,000 inactive rows (complete production update median):

| Mode | Before ms | After ms | Mass ledger visits after |
| --- | --- | --- | --- |
| Asleep | 15.695 | 1.634 | 0 |
| Active | 24.621 | 7.999 | 1000 |
| Mixed | 1311.510 | 1279.853 | 500 |
| Moving ship | 32.297 | 2.443 | 11000 |
| Zero-g | 45.047 | 10.617 | 1000 |

All 20 cases passed population/material/identity conservation, actual asleep
admission and independent solver-replay assertions. A separate raw-data check
confirmed identical initial inventories across refs and five samples in each
row. The mixed case remains overwhelmingly solver-bound; five samples do not
establish a statistically significant improvement there or any tail budget.
There is no FPS pass. Complete-update timing excludes the separate solver replay
and its setup/assertions, and includes history capture, physics, frame and mass
synchronization. Isolated selection is allocating read-only selection; it is
not silently counted as an additional production pass.

The after 1,000/10,000 asleep selection median was 0.580 ms; no second spatial
membership index was adopted. Existing exact range/frame semantics and inactive
retention remain, with linear selection cost visible in the raw report. A
moving frame deliberately visits all 11,000 retained centers for correct boundary
membership; that correctness cost is included in the measured 2.443 ms update.

The editing container lacks a Rust toolchain; portable checks ran on GitHub Ubuntu.
Native Apple M1 visuals/motion/FPS remain pending in
[#313](https://github.com/mr-exception/salimon/issues/313), not claimed here.

`client/world/tests/fragment_journal_cost.rs` isolates no-op pose/writeback and
journal drain from solver/adapter/mass timing. Its checked 0.032 kg current iron
spheres use production one-tick depleted extraction; it records 20 raw samples
for 500/1,000 writes plus independently 0/10,000 inactive identities. The same
integration test runs at the #304 baseline and completed stack, with expected
event counts supplied observationally by environment.

## Validation history and limits

- Full first stack: `cargo test --workspace --locked` **507 passed, 21 opt-in
  ignored**; workspace build, Clippy with warnings denied and 92 Python tooling
  tests **passed** ([run](https://github.com/mr-exception/salimon/actions/runs/39136204385)).
- The final domain checks include the strengthened contact oracle and production
  10 kg journal fixture. The standalone #304 baseline's 69 physics tests passed
  before indexing/scratch.
- Initial allocation extraction **failed** because the runner selected zstd
  rather than gzip. The runner now discovers the emitted trace extension;
  the subsequent paired CPU/asleep allocation collection passed.
- The first isolated journal after-probe **failed**: a shared Cargo target reused
  a baseline world artifact in a checkout with older mtimes. Its pipeline through
  `tee` also hid the failure from the job conclusion. That result is not a pass
  and is not used for journal acceptance. Independent per-ref target directories
  and explicit shell pipe failure propagation were added before rerunning.
- Native M1 motion, presentation/interpolation visuals and hardware performance
  remain **not run**, owned by [#313](https://github.com/mr-exception/salimon/issues/313).
  These CPU results do not close or waive that native follow-up.

## Per-mode allocation and journal evidence

[Allocation summary](allocation-summary.csv) and [ref/fixture/machine metadata](allocation-metadata.json)
come from [the final allocation/journal run](https://github.com/mr-exception/salimon/actions/runs/39138778339).
Each mode ran as its own five-sample 500-object Heaptrack process for each ref.
Counts include production fixture construction, applicable 80-tick preparation,
replay/setup/assertions and the Rust test harness; they are not per-update counts.
Compressed per-mode stack summaries and fixture/sample captures are beside
this report; raw zstd traces are in that run's `fragment-stack-attribution` artifact.

| Mode | Allocation calls before | After | Peak heap before | After |
| --- | --- | --- | --- | --- |
| asleep | 222,668 | 214,484 | 6.17M | 6.47M |
| active | 130,406 | 130,265 | 5.05M | 5.29M |
| mixed | 2,525,855 | 2,514,822 | 42.49M | 42.63M |
| moving-ship | 418,995 | 410,624 | 6.23M | 6.52M |
| zero-g | 97,683 | 97,362 | 5.25M | 5.49M |

Allocation calls fell in these captures, while retaining buffers increased peak
heap by 0.14–0.30 MB. The small active/zero-g call-count changes are not presented
as large improvements. Activation graph/manifold allocations remain substantial,
especially for mixed sizes; six-array reuse does not make the pipeline allocation-free.

The isolated journal rerun used independent Cargo target directories for each
ref and explicit pipeline error propagation. **Both probes passed**, 80 raw
samples each: [before](journal-before.csv), [after](journal-after.csv). Every
optimized no-op sample emitted zero events; every baseline sample emitted one
per requested write. For 1,000 writes + 10,000 inactive entries, median pose/journal
write time fell from **5045.96 µs to 19.49 µs**; drain time fell from **5.83 µs to
0.02 µs**. This isolates no-op pose/journal work from solver and mass timing.

Independent report integrity checks passed: 40 CPU rows/200 measured samples,
unique IDs, actual mass/density-derived radii and asleep-state counts; ten
per-mode allocation fixtures match across refs; all 160 journal sample event
counts match their expected ref behavior.

Final `cargo test -p salimon-physics -p salimon-world --locked`, workspace build,
Clippy with warnings denied and tooling contracts passed in the final run.
Runner rustfmt output was applied and local `git diff --check` passed. The final
report/CSV/gzip additions require no graphical launch. No production workflow
change is included in the three delivering PRs.
