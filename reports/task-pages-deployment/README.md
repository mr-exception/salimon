# Pages deployment skip fix

The first periodic nightly run [38907136946](https://github.com/mr-exception/salimon/actions/runs/38907136946) passed all verification and reporting steps, but its dashboard job was skipped. No Pages deployment was produced.

The dashboard condition lacked a status-check function, leaving GitHub's implicit success check across its dependency chain. Nightly runs intentionally skip weekly lanes (and weekly runs skip nightly). Override inherited skips with `always()`, reject cancelled runs, and explicitly require `needs.reports.result == 'success'` while retaining the repository-variable opt-in.

Updated the operations guide with 404/skipped-job troubleshooting.

Validation: actionlint 1.7.7 passes the complete modified workflow. Reviewed gating: successful reports plus opt-in publishes despite inactive lanes; failed/skipped reports, cancellation, or disabled opt-in do not publish. No game code changed.

Limitations: live deployment requires merging and starting a new default-branch run. Repository variable and Pages administration settings are not readable through the connector, so their configured values were not verified. No repository settings changed. Re-running the old run uses its original workflow revision.
