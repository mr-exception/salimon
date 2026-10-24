# Corpus, experience and balance verification (#568)

Sixth change in the requested stack. This report retains a clean tracked main
Linux reproduction separately from the current stack's native/portable checks.
GitHub workflow dispatch returned HTTP 403 (admin rights required); no hosted
periodic CI pass is claimed. Docker local Linux/software Vulkan supplies the
fallback using the actual resolved compiler, ICD and source hashes.

The native capture runner now restores the real window and polls a small scoped
inspection rather than rebuilding the world snapshot repeatedly. The optional
scope remains compatible with old executables. Native failure handling preserves
its original error/checkpoints/last state and never executes the `native` marker
or sends another request into a potentially timed-out protocol stream. E2E,
focus/occlusion and physical display admission remain intact.

Source: [Dockerfile](Dockerfile), [Linux runner](run-linux.sh), maintained runtime
and runner guides. Final original captures, corpus.json, context, command logs,
temporal results, source/binary hashes and all failures are recorded below.
No candidate, mask or tolerance is promoted to an approved baseline.

All #469–#475 implementation issues remain open in the retained
[dependency snapshot](dependencies.json). The version-1 experience matrix keeps
its twelve Scout portable tests and fourteen explicitly blocked future rows.
No fake backend/account/completion or navigation state is introduced. Whole-matrix
coverage remains pending; #537 owns source rights and physical route/balance
choices, #561 visual/audio/baseline approval, #557 physical platform/input and
#563 hardware budgets. The [physical trial worksheet](physical-trial-template.json)
is unfilled and is not evidence of a physical route or approved tuning.

Execution is in progress; final results and original inventories will be appended
before this PR is opened. No PR is merged by this task.
