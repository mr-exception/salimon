# Expedition Interface references

These 81 PNGs (27 states at each of 960×600, 1280×720, 1920×1080) are
reviewable test references for #831. Appearance approval is **pending owner
review**. They do not approve themselves or change gameplay authority.

Per-width manifests bind browser, fonts, DPR 1, fixed clock and pixel policy to
image SHA-256 hashes. Missing registrations/files, hash drift and pixel changes
fail. No masks or tolerance changes apply to this family. Existing navigation
and panorama references remain separate and unchanged.

The #901 refresh retains the delivered field-marker inventory control (#879),
bottom HUD right-click hint, active Mining/Cancel feedback and expanded,
heading-focused Surface controls Help (#890–#894). Only affected built-world
frames and their manifest hashes change;
gallery, contact and cockpit references, environment bindings and pixel policy
remain unchanged. Dialog frames also retain the updated HUD visible behind their
backdrop. Candidate generation still requires subsequent strict comparisons,
repeat captures and the visual/focus negative checks.

See the [client workflow](../../README.md#cross-view-expedition-interface-visual-regression-831)
for candidate generation, review, comparisons, repeat/negative checks and evidence
storage. Proposals go to ignored `client/test-results/`, never directly here.
Original/expected/repeat/diff captures and scenario JSON remain local/CI evidence;
these versioned images are source fixtures, not delivery reports. The owner
retains reference review and merge authority.
