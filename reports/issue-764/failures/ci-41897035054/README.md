# CI failure at first published head

Run: https://github.com/mr-exception/salimon/actions/runs/41897035054
Head: `9620d82d8939fc4cd1abef35367ded507259a234`
Tested merge: `4772c92d9ee3032209e7e9f46155b22d5439a91b`
Artifact: https://github.com/mr-exception/salimon/actions/runs/41897035054/artifacts/13356117823

Domain/build passed; 29 browser cases passed, PLANET-1 near-colony replay failed
with 3 pixels over the unchanged policy threshold and 5,009 raw changed pixels.
The paired recorded player/camera transforms are identical. Original PNGs,
observations, metrics and policy diff are retained here. The diff's affected
pixels include high-contrast illustrated crate edge coverage. The downstream
negative/repeat checks and baseline approval were not run/skipped after failure.
This run is not acceptance evidence for a repaired head.

The repair requests geometric precision from the SVG adapter and captures the
full 1280×800 viewport, preserving the viewer pixel grid and UI rather than a
fractional element crop. No comparator policy, baseline reference or mask changes.
Repeated local and current-head CI results must validate that repair.
