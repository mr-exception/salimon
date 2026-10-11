# Temporary wgpu Metal startup backport

`wgpu-hal/` is the crates.io `wgpu-hal` 30.0.1 source, with its original MIT and
Apache licenses. The workspace patch changes only Metal surface visibility:
allow acquisition until the hosting window has first reported visible, then
retain the existing occlusion protection. It backports
[upstream d91a995](https://github.com/gfx-rs/wgpu/commit/d91a995), adapting its
atomic type to 30.0.1's standard-library synchronization.

On macOS 26.2, repeated Salimon launches otherwise return `Occluded` indefinitely
before the first presented frame, including an active, raised native window.
This blocks issue #156 native acceptance. No graphics settings, shaders, timing,
other backends or physics are changed. Remove this patch and vendored source
when the pinned published dependency includes the upstream startup correction.

Published package metadata (`.cargo*`, `Cargo.lock`, `Cargo.toml.orig`) is omitted;
the normalized package manifest, build script, examples and source are retained.
