# #305 — Stable fragment ID index

FragmentOutput now maintains an ID-to-slot lookup beside its canonical ordered
Vec. Lookup/writeback is expected O(1); removal repairs shifted slots in O(n)
to preserve deterministic creation/solver order. Growth retains slots, both
legacy and sphere creation register IDs, and removal cannot recycle IDs.
MiningSession exposes a read-only identity query; pickup and release use it.

Tests cover head/middle/tail/repeated removal, growth, exact index/Vec agreement,
monotonic replacement identities and ordered coalesced deletion journal events.
Existing production lifecycle, pickup/drop, consumption and streaming restoration
tests exercise the same index. A focused ignored release benchmark compares the
historical linear lookup oracle to indexed lookup on identical validated 10 kg
iron populations, 500/1,000 active plus 0/10,000/50,000 inactive entries. It retains
20 raw samples and index capacity (unchanged during measured lookups).

## Portable lookup evidence

[120 raw release samples](lookup-samples.csv), 20 per case, from Ubuntu 24.04,
Rust 1.99.1, the same-process historical-linear oracle and indexed implementation.
[Runner evidence](https://github.com/mr-exception/salimon/actions/runs/39136864104).
The index benchmark passed; that run later failed only while extracting a
zstd allocation trace for the separate runtime comparison. Extraction was fixed.

| Active queries | Inactive retained | Linear median µs | Indexed median µs |
| --- | --- | --- | --- |
| 500 | 0 | 104.60 | 12.75 |
| 500 | 10000 | 5488.98 | 22.36 |
| 500 | 50000 | 28742.24 | 63.07 |
| 1000 | 0 | 353.27 | 17.37 |
| 1000 | 10000 | 15050.32 | 64.25 |
| 1000 | 50000 | 77748.46 | 107.64 |

Doubling the queried population at zero history grew the linear cost about
3.4× versus about 1.4× indexed in this sample. Adding retained history no longer
adds a linear search to every writeback. Cache locality still affects lookup
latency; expected constant-time lookup is not a constant nanosecond guarantee.
Index capacity remained unchanged during queries. This is structural capacity
and wall-clock evidence, not a global allocator interception count.

Portable runtime selection/adapter/solver and allocation attribution is in the
[stack report](https://github.com/mr-exception/salimon/blob/issue-306/reports/issue-306/README.md).
 Range selection keeps
its existing exact scan/frame semantics pending measured justification for a
second membership structure. No inactive state is pruned.

Local `git diff --check` passed. The first complete stack verification passed 507 Rust tests (21 opt-in ignored),
workspace build and Clippy with warnings denied, plus 92 Python tooling tests.
[Verification run](https://github.com/mr-exception/salimon/actions/runs/39136204385).
The editing container lacks cargo/rustc; those gates ran on GitHub Ubuntu. Native M1 acceptance remains
pending in [#313](https://github.com/mr-exception/salimon/issues/313).
