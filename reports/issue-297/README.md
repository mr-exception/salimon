# Issue #297 — historical alias provenance and corrected reports guide

The missing manifest was not an incomplete upload. Migration `db534caf08cfe3c133d3dce06f9cb6532b4f1247`
created it while moving/deduplicating report images. Owner cleanup `9b0a24526e2da337efdac2beb31c525e7fbb27d1`
(`chore: removed old task reports`, 2026-10-06) deliberately removed the index
and the two report folders. The current reports guide retained a stale claim
that the index/images were still present.

Corrected the guide to distinguish historical aliases from live files and point
to immutable historical Git objects. No deleted image folders, duplicate bytes
or dangling local alias targets are restored. Existing evidence is unchanged. The broader link scan also found one pre-existing
scout-guide link to the intentionally removed #80 report; it now points to the
verified pre-cleanup revision rather than a missing local file.

| Historical removed copy | Historical retained image | Verified SHA-256 |
| --- | --- | --- |
| `issue-29/failure.png` | `issue-29/landed-cabin.png` | `dd9387150c216de7176d46b23475f73978b3250a3ff08f0cde192af8c9dc1baf` |
| `issue-34/step-018-cargo-route-cockpit-17.png` | `issue-34/step-123-cargo-route-cockpit-120.png` | `d573ff63d1a2eea55266b1f7417ee0945743ff619bd8ec9a9c6c3afe409cee1f` |

Verification resolves both original copies from pre-migration revision
`b926083cdaf44047cd669e7767c755d7ebb42616` (`docs/issue-*` paths), both retained copies from pre-cleanup
`7e8946ee75d7f9be1158118109b26d47f96445ec` (`reports/issue-*` paths), and hashes all six Git blobs.
Every original pair and retained copy matches its recorded digest.
[Machine-readable paths, revisions, sizes and digests](evidence-provenance.json).

Historical references:
- [Original alias manifest](https://github.com/mr-exception/salimon/blob/7e8946ee75d7f9be1158118109b26d47f96445ec/reports/evidence-aliases.json)
- [Retained #29 image](https://github.com/mr-exception/salimon/blob/7e8946ee75d7f9be1158118109b26d47f96445ec/reports/issue-29/landed-cabin.png)
- [Retained #34 image](https://github.com/mr-exception/salimon/blob/7e8946ee75d7f9be1158118109b26d47f96445ec/reports/issue-34/step-123-cargo-route-cockpit-120.png)
- [Intentional cleanup](https://github.com/mr-exception/salimon/commit/9b0a24526e2da337efdac2beb31c525e7fbb27d1)

Validation: exact `git show <revision>:<path>` blob reads and Python SHA-256
comparisons passed; current-tree absence verified. Changed Markdown relative
links/anchors and `git diff --check` passed. This is documentation/provenance
only; Rust/model/native checks are not required for this incremental commit.
The parent stack passed 675 Rust tests (22 opt-in ignored), 99 model tests
(2 optional skips), build, Clippy and source/export checks; no new native claim.
