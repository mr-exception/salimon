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

Portable runtime selection/adapter/solver attribution and raw index evidence
are collected with the final #306 stack; see that report. Range selection keeps
its existing exact scan/frame semantics pending measured justification for a
second membership structure. No inactive state is pruned.

Local `git diff --check` passed. Rust/format/lint gates run on GitHub Ubuntu
because this editing container lacks cargo/rustc. Native M1 acceptance remains
pending in [#313](https://github.com/mr-exception/salimon/issues/313).
