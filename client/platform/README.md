# Platform

Reserved for native input and OS-specific adapters. Task 2 uses `winit` inside
the runtime because the bootstrap has no portable gameplay domains and the
architecture permits native integration at the platform/runtime boundary. No
separate platform crate or adapter is implemented yet.

Move behavior here when it becomes genuinely platform-specific or when Windows
and web targets require interchangeable adapters. Portable world, character,
and ship domains must consume typed capabilities and must not depend directly on
`winit`, macOS APIs, or native window handles.
