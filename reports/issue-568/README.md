# Corpus, experience and balance verification (#568)

Sixth change in the requested stack, based on #600. Native capture now restores
the actual window and polls only readback status, avoiding repeated world snapshot
construction. Failure handling preserves the original error, checkpoints and last
state without executing the `native` marker or reusing a timed-out protocol stream.
The optional inspection scope remains compatible with older executables. Existing
E2E guards and focus, occlusion and display admission remain authoritative.

Verification completed in two separate contexts. Each captured 128 original images
across nine fixtures and passed all 86 temporal comparisons. These are within-lane
checks, not cross-platform pixel equivalence or human appearance approval.

| Evidence | Clean main Linux | Current stack native Metal |
| --- | --- | --- |
| Source checkpoint | `e863ded7e6eb60f7da53483d52ac6a73874a09be` | `88703d8372efe231c1647229b9438a821a43dafc` |
| Platform | Linux aarch64 container, software Vulkan/llvmpipe | macOS 26.2, Apple M1/Metal |
| Physical capture size / DPI scale | 1280×800 / 1 | 1280×800 / 2 |
| Compiler | Rust 1.99.1, LLVM 23.1.1 | Rust 1.89.0, LLVM 20.1.7 |
| Corpus | 128 images, nine fixtures | 128 images, nine fixtures |
| Temporal comparisons | 86 passed | 86 passed |
| Experience matrix | 12 passed, zero failed, 14 blocked | 12 passed, zero failed, 14 blocked |
| Portable survival balance | Six passed | Six passed |

Main was the merged #596 checkpoint and had clean tracked files before and after
execution. Metal includes all five preceding changes and the capture fix at a clean
source checkpoint. Subsequent commits add reports and evidence. Exact executable
SHA-256 values, compiler output and image identity are in
[build context](build-context.json), [Linux context](linux-context.json),
[Linux summary](linux-summary.json) and [Metal summary](metal-summary.json).
Linux used Mesa 25.2.8 llvmpipe (LLVM 20.1.2, 128 bits); Metal did not return a
meaningful driver-version string, so none is inferred.

## Checks and results

The [Linux step ledger](linux-status.jsonl) records every command and exit code.
Formatting, workspace Clippy with warnings denied, release build, all workspace
unit/doc tests, exact GLB processing, three candidate importer tests, three headless
resource tests and two extended persistence tests passed. Runtime had 360 passed /
13 ignored; renderer 75 passed / 14 ignored. Model tests ran 99 with three expected
Blender skips; script tests passed 118. Blender was not installed in this container;
preceding asset PRs retain their actual Blender 4.5.3 authoring checks.

All 39 graphical baseline scenario cases and 47 screenshot-evidence scenario cases
passed on Linux. The packaged Linux executable passed the virtual X11 input smoke:
window creation, renderer ready, presentation, F2 PrecisionTour, F2 Gameplay and
continued playability. See [packaged smoke result](packaged-smoke-result.json).
Virtual input is not a physical keyboard/display validation.

On the current stack, the new scoped RPC regression passed, runner tests passed
21, all script tests passed 120, and Clippy, release build, formatting and the
documentation gate passed. The prior stack's workspace checks are recorded in
#600. Actual native Metal captures cover the new capture path. Original logs live
under [validation](validation), with complete game protocol logs in the archives.
The test log's printed synthetic timeout is a runner failure-fixture assertion;
the unittest result is successful.

GitHub workflow dispatch was attempted and denied HTTP 403 (admin rights required).
The [original response](hosted-dispatch-error.log) is retained. The local Docker
nightly-equivalent run completed; this is not a hosted periodic CI pass. Source
build checks for #597–#600 passed at their recorded heads in
[preceding PR checks](preceding-pr-checks.json).

## Pending decisions and measured limits

The reviewed appearance command correctly returned pending (exit 3). The older
approved software Vulkan v1 baseline has incompatible context/fixtures; software
v2 is a candidate and Metal v1 awaits review. Neither is an eligible approved
comparison for these captures. See both baseline audits:
[Linux](linux-baseline-audit.json), [Metal](metal-baseline-audit.json).
No mask, tolerance or baseline approval was changed.

All #469–#475 implementation issues remain open in the
[dependency snapshot](dependencies.json). The version-1 experience matrix retains
all 14 explicitly blocked rows alongside its 12 passing Scout rows. See
[Linux matrix](experience-linux.json) and [Metal matrix](experience-metal.json).
Whole-matrix coverage remains pending; backend/account/completion state is not
simulated to make these rows pass.

Balance output is provisional evidence, not tuning approval or a measured physical
route. Baseline first ration is 86.304 s; slow setup 431.304 s; low-health/fatigue
221.384 s; the intentionally net-negative route is 941.304 s with a −12.2947 margin.
The recipe remains 0.4 kg biomass + 0.1 kg ice, 10 s at 10 kW, with a 720 s
break-even assumption. The 200 m/16-seed availability sample found both materials
on Earth in 16/16 seeds and on Mercury, Venus, Moon and Mars in 0/16; this is not a
global availability guarantee. Full values and assumptions are in
[Linux balance](linux-balance.json), [Metal balance](metal-balance.json) and the
corresponding availability files. The [physical worksheet](physical-trial-template.json)
remains blank. #537 owns source rights and physical route/balance decisions;
#561 owns visual/audio/baseline approval, #557 physical platform/input and #563
hardware budgets. The natural-settling and realtime performance limits measured
in #600 remain unresolved; this corpus does not override those measurements.

## Original evidence and integrity

The [Linux archive index](linux-archive-index.json) lists all 17 Linux archives;
[Metal evidence](metal-stack-evidence.tar.xz) preserves its original capture tree.
Each archive has a small `.inventory.json` summary and a full gzip JSON inventory
of original paths, sizes and SHA-256 hashes. XZ compression and TAR hardlinks only
deduplicate identical bytes; no image was resized or recompressed. Representative
unaltered originals: [Linux core](images/linux-core.png),
[Metal core](images/metal-core.png), [Linux cockpit](images/linux-cockpit.png).

[Coverage](original-coverage.json) checks the inventories against every Linux
artifact path. [File verification](archive-file-verification.json) streamed every
archived regular file and hardlink against its original SHA-256 manifest.
[Wrapper verification](archive-wrapper-verification.json) verifies that converting
the original gzip wrappers to XZ preserved the entire uncompressed TAR byte stream.
The compiled Linux executable is retained locally and recorded by digest instead
of committing the ELF binary; the archive inventory explicitly identifies this
single omission. Original screenshots, logs, corpus/context and comparison results
are preserved, including pending/failure evidence.

## Reproduction

Use a self-contained disposable clone: mounting a Git worktree alone would omit
its external Git metadata. From the repository containing this report:

```sh
task_repo=$(pwd)
task_scratch=$(mktemp -d)
git clone --no-hardlinks "$task_repo" "$task_scratch/main"
git -C "$task_scratch/main" checkout e863ded7e6eb60f7da53483d52ac6a73874a09be
mkdir "$task_scratch/target"
docker build -t salimon-verification-568 -f "$task_repo/reports/issue-568/Dockerfile" "$task_repo/reports/issue-568"
docker run --rm -e EXPECTED_REVISION=e863ded7e6eb60f7da53483d52ac6a73874a09be \
  -e CARGO_TARGET_DIR=/target \
  -v "$task_scratch/main:/source" -v "$task_scratch/target:/target" \
  -v "$task_repo/reports/issue-568/run-linux.sh:/run-verification.sh:ro" \
  salimon-verification-568 bash /run-verification.sh
```

[Dockerfile-measured](Dockerfile-measured) and [run-linux-measured.sh](run-linux-measured.sh)
retain the exact executed recipes. The reproducer pins the observed Rust 1.99.1
instead of re-resolving `stable`; base image digests and installed package versions
are recorded in validation logs. The final runner additionally asserts the empty
tracked-status result that was manually verified for the measured run. Appearance
and experience exit 3 remain pending, while any other failed step fails the runner.
For Metal, use the source checkpoint above, build the release client and run
`scripts/model_visual_tests.py capture --binary target/release/salimon-client`
then `scripts/model_visual_tests.py compare --temporal-only` on the native display.
No PR is merged by this task.
