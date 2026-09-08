# Renderer

Reserved for the custom `wgpu` renderer, starting in task 2. This boundary will
own GPU resources, render passes, materials, and presentation. Consume typed
render data without owning or mutating authoritative domain state. No renderer
or GPU dependency is implemented in task 1.
