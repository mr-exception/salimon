# #313 — native sphere validation (recollection in progress)

**Draft: production staging provenance was invalidated.** The first production staging build reused the temporary constant-damping ablation library through a shared Cargo target directory. Its measurements/captures are retained under [rejected-cache](rejected-cache/STATUS.md) and are not current-physics acceptance.

The source fixture change, debug workspace tests and strict Clippy results remain valid; production physics source was never reverted. A fresh `target-production` directory is now building the actual production source, followed by release damping regressions and complete affected native recollection. PR #456 remains draft until this correction is complete.

The isolated production build (SHA-256 `572e70b5…`) and release physics gates now
pass; corrected native pools contain 187/117 completed moving cap intervals for
500/1,000 objects. A source-owned input-isolation switch is being added to
fixed-step runner children after real desktop events changed initial camera/holds
in two functional captures. Direct OS probes retain normal native delivery.
