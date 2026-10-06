# Platform

Reserved for native input and OS-specific adapters. The runtime owns current
`winit` integration and translates native events into typed commands for the
portable gameplay domains. No
separate platform crate or adapter is implemented yet.

Move behavior here when it becomes genuinely platform-specific or when Windows
and web targets require interchangeable adapters. Portable world, character,
and ship domains must consume typed capabilities and must not depend directly on
`winit`, macOS APIs, or native window handles.
