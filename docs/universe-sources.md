# Universe source manifests and offline staging — U06 (#407)

The frozen [manifest](../catalogs/sources/manifest-v1.json) records official Gaia
DR3, JPL Horizons and NASA Exoplanet Archive PS sources, exact acquisition queries,
release/model context, coverage, uncertainty, credits/reuse policy references and
SHA256/byte counts. Checked on 2026-10-21. It is a development fixture manifest,
not a complete catalog download or production redistribution approval.

| Source | Pinned fixture | Limits and reuse |
| --- | --- | --- |
| Gaia DR3 / `gaiadr3.gaia_source` | Two explicitly synthetic CSV rows, J2016.0; no real Gaia bytes committed | Negative/missing parallax and missing radial motion remain explicit. Official [license](https://www.cosmos.esa.int/web/gaia-users/license) index states CC BY-NC 3.0 IGO. Full Cosmos pages returned 451 here. [Credits](https://www.cosmos.esa.int/web/gaia-users/credits) and release papers must be reviewed; production restricted pending #513. |
| JPL Horizons | Actual frozen Earth-399 barycentric ICRF vector API JSON, two dates, KM-S/TDB; response signature/header retained | [Manual](https://ssd.jpl.nasa.gov/horizons/manual.html) defines frames, epochs, validity and citation. No full covariance, invented certainty, orbit integration or complete SPK pack. Source-specific production reuse review is uncompleted. |
| NASA Exoplanet Archive PS | Actual frozen default-row Proxima Cen b CSV with Gaia identity, asymmetric errors and original literature reference | [PS columns](https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html) distinguish self-consistent rows from composite data. [Acknowledgment policy](https://exoplanetarchive.ipac.caltech.edu/docs/acknowledge.html) requests Archive credit, Christiansen et al. (2025) and original references. Public data access does not establish blanket rights to contributed material. |

Gaia [bulk/TAP access](https://www.cosmos.esa.int/web/gaia-users/archive/extract-data)
uses release-specific tables. Preserve original source IDs, error/quality columns,
missing velocities and selection coverage. U07/U09 must not equate inverse noisy
parallax with certain distance. JPL [API parameters](https://ssd-api.jpl.nasa.gov/doc/horizons.html)
are frozen in the query URL; retain returned service/model/header/epoch/frame.
Exoplanet [TAP access](https://exoplanetarchive.ipac.caltech.edu/docs/TAP/usingTAP.html)
is a live evolving table: the archived exact response digest, query and date pin
this input, not a claim that rerunning a live endpoint yields identical bytes.
Errors/nulls/source references are preserved without normalization by U06.

## Offline import

Run from the repository root:

```sh
python scripts/import_universe_sources.py --output artifacts/universe-sources-u06
python -m unittest discover -s scripts -p test_import_universe_sources.py -v
cargo test -p salimon-universe --locked
```

Choose a new output path on each run. The standard-library CLI reads only local
manifest/artifacts, streams SHA256/size verification, rejects duplicate/unsafe/
escaping paths, stages into an adjacent temporary directory, reverifies copied
bytes and renames only a complete validated bundle. Existing output is never
replaced. No downloader, socket, credentials, runtime hook or scientific conversion
is included. Tests disable sockets and compare two complete builds byte-for-byte.
Rust `sources.rs` validates the shared typed metadata with independent fixtures.
U07 now implements the [Gaia fixture adapter](universe-gaia-ingestion.md);
U08 now implements the [JPL/exoplanet fixture adapters](universe-observed-ingestion.md). Binary codecs remain U04.

`--production` fails on the checked-in manifest: restricted/unreviewed sources
and synthetic fixtures cannot be published. There is no opt-in override flag.
[Follow-up #513](https://github.com/mr-exception/salimon/issues/513) owns source-use
review/permissions and actual full input snapshot approval before production Gaia
work or pack distribution. Code licensing does not relicense third-party data.
Fixture source bytes retain their provider's terms and references. U06 closure
records manifest/tooling delivery, not approval of production use or science.

U07 now provides [Gaia scientific fixture ingestion](universe-gaia-ingestion.md);
raw staging remains separate and production gates above still apply.

## Private-development decision and production evidence (#513)

On 2026-10-22 the owner selected **private development only** in the task
conversation. [Decision and exact input inventory](../reports/issue-513/private-development-decision.json)
bind that scope to the current manifest. Keep synthetic Gaia and the existing
small JPL/PS development inputs; no actual Gaia release, commercial distribution,
production source approval or alternative scientific source is authorized by
that choice. Source-specific restrictions and unreviewed statuses remain intact.

Production staging now additionally requires `--production-review <file>`.
A separate schema-1 `salimon.universe.production-review` JSON record must bind
`manifest_sha256` to the **exact input manifest bytes**, identify `intended_use`,
`distribution`, human `reviewer`, ISO `reviewed_on` and `approval_evidence`, and
contain exactly one `sources` entry for every source ID. Each entry records:

- `id`, `terms_url`, ISO `terms_checked_on`, `terms_evidence` identifying archived
  full terms, exact `credit_text` and source-specific `references`;
- `decision`: `terms_permit_intended_use` or `permission_obtained`, with a nonempty
  `permission_record` for the latter;
- `snapshot_scope: full_production_selection`, exact `acquisition`, `epoch`,
  `frame`, and explicit `snapshot_approval_evidence`.

The bound manifest already freezes query/release and every input size/hash.
Changing it invalidates review. Stage preserves manifest bytes and includes the
review record, then revalidates both before atomic publication. Status/origin
flag edits alone cannot promote development snapshots. Reserved output metadata
paths cannot be artifact names. Rust `SourceManifest` remains the unchanged
metadata validator; the offline Python staging boundary enforces the additional
human review sidecar. No runtime consumer or production packaging is added.

These checks establish record completeness and binding; they cannot verify the
truth or legal sufficiency of a human decision. Owner source-use review must
inspect actual full current terms/permissions and actual complete selected data.
No full current terms were fetched in this restricted environment. Existing
2026-10-21 references are historical guidance, not refreshed legal approval.
U07 and U12 publication remain gated until that review; private fixture science
and implementation may continue. See the [report](../reports/issue-513/README.md).
