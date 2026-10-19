# Issue 343 — Fabricator screen and interaction

Delivered keyboard crafting modal attached to the authored wall monitor. Separate
ray/face regions disambiguate the universal input port and monitor. Both enforce
interior, 3 m range, front face and cabin occlusion. Opening clears held controls;
modal input captures movement, mouse look, toolbar and mining while work continues.
Focus/lifecycle clearing and leaving range close it. Native repeats/synthetic
presses are rejected by the existing input policy.

Up/Down browses supported/locked definitions, Left/Right selects quantity, Enter
adds, Tab inspects FIFO jobs, Delete cancels without refund, E/Escape closes. Raw
bins show stored/20 kg; recipes show fixed ingredients/duration/watts; queue rows
show quantities and remaining time. Shortages report exact required and stored
material quantities. Power pause and occupied output are separate statuses. The
centered high-contrast panel uses 3x glyphs and 66-character wrapping; CPU tests
verify its default 1280x800 fit. All definitions/jobs are browsable without drawing
the complete 64-job technical queue over gameplay. The physical monitor uses a
fifth independent instrument-atlas region for live status/job count/active time.

Validation on Linux x86_64, Rust 1.99.1:

- `cargo test --workspace --locked`: passed after correcting atlas test helper
  byte stride; renderer's 63 CPU tests include WGSL/import/cache regressions.
- `cargo test -p salimon-client --locked crafting`: 4 passed (includes the
  final locked-recipe/departure regression added after the workspace run).
- `cargo build --workspace --locked`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.

Native smoke/capture was not run: this host has no display/Vulkan driver. CPU
raster size/color checks and WGSL validation do not establish visual fidelity,
OS accessibility integration or reference performance. Native legibility,
interaction/playability and screenshots remain explicitly tracked by [#348](https://github.com/mr-exception/salimon/issues/348).
No survival, progression, persistence or upgrade behavior is introduced.
