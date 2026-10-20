# #313 — native sphere validation (recollection in progress)

**Draft: production staging provenance was invalidated.** The first production staging build reused the temporary constant-damping ablation library through a shared Cargo target directory. Its measurements/captures are retained under [rejected-cache](rejected-cache/STATUS.md) and are not current-physics acceptance.

The source fixture change, debug workspace tests and strict Clippy results remain valid; production physics source was never reverted. A fresh `target-production` directory is now building the actual production source, followed by release damping regressions and complete affected native recollection. PR #456 remains draft until this correction is complete.
