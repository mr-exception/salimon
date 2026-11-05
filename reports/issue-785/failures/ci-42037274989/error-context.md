# Preserved CI failure context (compact)

- Run: https://github.com/mr-exception/salimon/actions/runs/42037274989
- Original artifact: https://github.com/mr-exception/salimon/actions/runs/42037274989/artifacts/13418953373
- Test: TD-1/4 one scene, stable layers, visible interior and identical fresh-run baseline captures
- Failure: `expect(received).toEqual(expected)` (deep equality), `tests/top-down.spec.ts:46:23`.
- Failed expression: `expect(captures[1]).toEqual(captures[0])`.
- Result: 36 browser scenarios passed, 1 failed; baseline approval was skipped after failure.
- Actual difference: seven RGBA pixels at the bottom-left rounded viewport corner.

The original PNG captures, failure screenshot, comparator diff and exact per-pixel
RGBA values are retained beside this file. The verbose textual expansion of PNG
Buffer bytes is omitted here to avoid duplicating the preserved binary originals.
The linked original artifact contains the complete generated error context/trace.
No assertion or failure result is removed or relabelled as passing.
