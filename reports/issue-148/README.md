# Issue 148 — fragment geometry and pile contacts

Work in progress: validation and native evidence will be recorded before completion.

Fragments use a 5× density-derived linear scale (half the previous 10×), while
all deposits retain their 10× scale and mining mass/yield is unchanged. CPU
vertices from the immutable authored GLBs define cached convex envelopes.
Spheres are broad-phase/clearance only; separating-axis narrow phase includes
face normals and crossed edge directions. Contact impulses include mass,
friction and angular response; orientation is written back to the existing
session and rendered, carried, targeted and preserved in the ship frame.

No authored assets or engine dependencies are changed. Convex envelopes bridge
small concavities (particularly the clustered ice variant); they do not model
individual disconnected shards. Adaptive substeps are bounded rather than full
continuous collision detection. Disk persistence remains outside the prototype.
