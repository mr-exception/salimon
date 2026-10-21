# #314 — Metal candidate review corpus

Collected on the reference Apple M1 on 2026-10-20/21; final corrected corpus captured 2026-10-21. Delivered after [#456](https://github.com/mr-exception/salimon/pull/456). **Candidates remain unapproved.** Human baseline/mask/tolerance approval and true live interpolation review transfer to [#477](https://github.com/mr-exception/salimon/issues/477). Closing #314 records this delivery and transfer, not a visual/interpolation signoff.

## Current captures

[Candidate corpus](candidates/corpus.json): 128 production PNGs, 1280×800 physical pixels, scale 2, Apple M1/Metal, macOS 26.2 arm64, deterministic fixture seeds/steps. Collection checkout `97a7a38db5de920dc19e682253af860ed1fb4477`; executable SHA-256 `1703da38f7109332542a28acde9d896df1eae9af2f18fd20ba69619bf4f48762`, runtime source `c691e6b`. Rust 1.89.0, Python 3.14.2. Actual device is iMac21,1, 16 GiB, AC; built-in nominal59.995 Hz. Adapter driver text is the API's unavailable `" / "`, not an invented version.

The capture runner uses explicit E2E desktop-input isolation introduced by #313. Each checkpoint uses production gameplay/rendering and direct native GPU capture, while external keyboard/pointer/focus cannot change scripted aim/holds. Fixed-step E2E disables live interpolation. Neither repeat stability nor these stepped flight views certify temporal normal-play appearance.

Review sheets cover all 37 distinct PNG contents: [1](review-sheets/sheet-1.jpg), [2](review-sheets/sheet-2.jpg), [3](review-sheets/sheet-3.jpg), [4](review-sheets/sheet-4.jpg). Full resolution examples: [lower port cockpit](candidates/cockpit/lower-port-landed-repeat-00.png), [lower starboard cockpit](candidates/cockpit/lower-starboard-landed-repeat-00.png), [Core](candidates/core/core-restart-repeat-00.png), [deployed ramp](candidates/ramp/deployed-exterior-repeat-00.png), [tool/deposit](candidates/resources/equipped-mining-tool-repeat-00.png). Agent inspection of these sheets found the intended geometry/materials and readable contextual labels at the captured viewpoints; this is not repository-owner image/mask/tolerance approval. Current HUD/toolbar occupies a substantial lower-screen band; acceptance of that art/layout remains the owner's decision.

The corpus covers cabin/exterior/glazing, seated and standing canopy/lower views, thrust/landing/takeoff poses, Core, tool/deposit/mining, spherical pile/floor, closed/lowering/deployed door/ramp and prompts. [Supplemental 71-step reversal scenario](door-reversal.json) passes on this exact binary; its raw state and images are in [#313's isolated functional evidence](../issue-313/isolated-functional/full/door-reversal/). It includes opening, extension, close/reopen reversal and downward ramp views, exercising the merged #333/#334 sources. Fixed checkpoint views and traversal assertions are bounded coverage, not proof of every live in-flight seam or temporal transition. Dedicated Core access/outage coverage and dark-cabin limitation are in [#299](../issue-299/README.md), with remaining contrast decision #451.

## Comparisons and defects

- **Passed:** all 86 frozen-repeat comparisons, zero changed pixels, existing tolerance8/fraction0.001, no new masks. [Per-check results](temporal.json).
- **Diagnostic only:** all 128 images match the original valid `a25fb874…` production corpus within existing comparison tolerances. [Inventory/results](review-sheets/inventory.json). This independently helped reject the intermediate cache-contaminated corpus, which differed in contact views. The original captures are not approved baselines.
- **Passed comparator self-test:** injected persistent silhouette, material and layout defects each fail appearance comparison against a current candidate, while repeating the wrong image passes temporal comparison. [Results/diffs](injected/), [reproducer](inject_defects.py). This demonstrates the distinction between stability and appearance; it does not approve the reference image.
- **Pending, not passed:** reviewed appearance comparison exits3, `human image/mask/tolerance approval pending`. [Status](appearance.json). No baseline images, metadata approval, masks or tolerances were promoted/relaxed. The pending `metal-option-a-v1` descriptor specifies scale1; actual candidate scale2 requires deliberate review in #477. Software-Vulkan approval is independent and unchanged.

## Native display/input boundary

Attached displays: built-in mode60 Hz, nominal59.9952 Hz; external mode75 Hz, nominal74.9726 Hz. [Observed modes](display-modes.json). Physical120/144 Hz unavailable. Existing synthetic high-refresh/interpolation CPU regressions pass in #313's workspace checks, and remain separate from physical-display acceptance.

The bounded [75 Hz probe](window75.py) uses actual native `Game` input (isolation off), existing macOS Accessibility/CGEvent PID helper [source](input.swift), direct GPU captures and AX window movement. It confirms external scale1 and restores primary scale2. **Native steering and mouse-delivery checks fail**; [raw observations/screenshots](external75/), [summary](314-isolated-final-75hz.log). Focus helper completion and successful window movement do not prove gameplay capture/delivery. Fixed-step E2E still disables interpolation, so this probe grants no live interpolation/FPS acceptance. Remaining input delivery coordinates #453; live review is #477.

Earlier software/direct capture candidates, failed native-input attempts and rejected contaminated-cache stage remain in ignored artifacts or #313's explicitly rejected archive; they are not relabeled as this final corpus. A first isolated capture completed image generation while the report commit was still changing, and correctly failed clean-tracked-revision admission; the final corpus was recaptured from clean `97a7a38`.

## Reproduction and checks

```sh
RUST_LOG=info python3 scripts/model_visual_tests.py capture --binary artifacts/native-20261020/build-isolated/salimon-client --captures artifacts/native-20261020/314/isolated-final-candidates
python3 scripts/model_visual_tests.py compare --temporal-only --captures artifacts/native-20261020/314/isolated-final-candidates --output artifacts/native-20261020/314/isolated-final-temporal
python3 scripts/model_visual_tests.py compare --reviewed --captures artifacts/native-20261020/314/isolated-final-candidates --baselines scenarios/visual/baselines/metal-option-a-v1 --output artifacts/native-20261020/314/isolated-final-review
python3 reports/issue-314/inject_defects.py
RUST_LOG=info python3 reports/issue-314/window75.py --binary artifacts/native-20261020/build-isolated/salimon-client --helper artifacts/native-20261020/input-314 --output artifacts/native-20261020/314/isolated-final-75hz
```

Capture/temporal/defect commands pass; reviewed comparison exits3; external probe exits2/partial. This PR changes report evidence/helper scripts and the maintained follow-up link; it changes no runtime renderer, assets, CI or baseline policy. Source checks and 25 functional routes for the same binary are linked in [#313](../issue-313/README.md). Evidence hashes: [manifest](sha256.json). Large JSON/logs are losslessly gzipped; their recorded original paths are historical. Human appearance approval, live normal-play interpolation and external-display input checks remain pending/failed in #477/#453.
