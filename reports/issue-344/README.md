# Issue 344 — Deliberate output tray collection

Completion displays the existing authored stationary package once, keyed by the
Fabricator's `(job, unit)` tray observation. Rations have a colored band; medkits
a medical cross. Ship geometry/pose carries it through translation and rotation,
without world gravity, loose physics, auto-transfer or a carrying step. Blackout
does not hide output. The existing module collision envelope is preserved.

E targets a separate tray face under shared interior, range, front-face and
occlusion checks. First-vacant typed food/medical receiver acceptance precedes
removing the single output under exclusive session ownership. A full category
reports feedback and preserves output, mass and queue block. Repeated empty
collection cannot duplicate an item. Only tray output contributes ship mass;
collected inventory intentionally contributes none.

The portable receiver foundation (three food/two medical, one each, category
and resident identity checks) is necessary for this transaction. #345 completes
its consumption API, slot-order/category tests and M/N production input. HUD and
native scenario/signoff remain #346/#347/#348 respectively.

Validation on Linux x86_64, Rust 1.99.1:

- `cargo test -p salimon-client -p salimon-fabrication -p salimon-renderer --locked`:
  passed, including domain FIFO/power/tray regressions, runtime integration and
  renderer WGSL validation of conditional category visuals.
- `cargo test -p salimon-client --locked tray`: 4 passed, including final new
  regressions for both categories, full/vacant slots, blocked blackout, repeated
  collection, mass changes, unchanged carrying/world identities and moved anchors.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`, formatting and `git diff --check`: passed.

Native GPU visuals/OS input were not run: no display/Vulkan driver on this host.
Actual medical/ration appearance, readability, room interaction and frame pacing
remain explicitly tracked by [#348](https://github.com/mr-exception/salimon/issues/348).
CPU/portable checks do not constitute native visual signoff.
