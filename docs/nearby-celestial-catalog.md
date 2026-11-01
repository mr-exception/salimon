# Nearby celestial catalog (#524)

The owner chose a **1,000,000,000-byte maximum** nearby dataset on 2026-11-01,
replacing #524's previous all-usable-DR3 delivery. The first publication is
`nearby-524-v1`. Its [coverage record](../catalogs/nearby-524-v1.json) records
measured counts, bytes and hashes; the [delivery report](../reports/issue-524/README.md)
contains commands, original numerical checks and actual limitations.

## Area and contents

| Source | Exact selection | Meaning and limits |
| --- | --- | --- |
| Gaia DR3 `gaiadr3.gaia_source` | All sky, observed `parallax >= 10` mas | Nominal 100 pc / about 326 light-years around the Sun. An observational cut, not a geometrically complete sphere or certain inverse-parallax distance. |
| NASA Exoplanet Archive frozen PS default rows | `0 < sy_dist <= 100` pc | Published nominal host distance, with original error/limit fields. Missing/outside distances are excluded; this cut can disagree with Gaia parallax. |
| Existing JPL Horizons snapshots | 18 explicitly selected Solar System targets from #526 | Sun, eight planets, Moon, selected giant-planet moons, Pluto and Charon. Not a complete asteroid/comet/moon inventory. |

The Gaia source query has **574,531 rows before quality selection**. All original
selected columns survive in frozen inputs; the production importer uses its
existing RUWE <=1.4 / reject-duplicated-source policy and coordinate validity
checks. Missing RUWE remains a separately counted unknown. Rejected input rows
remain in the frozen sources rather than being silently lost. Final accepted,
rejected, missing-measurement and planet/body counts are in the coverage record.

Missing/nonpositive or smaller parallaxes and Gaia non-detections are outside
this query. Quality-rejected input rows remain available as raw data but do not
become accepted Gaia observations. PS contains published known planets with a
usable nominal host distance, not inferred undiscovered planets. These source
limits matter when expanding coverage; increasing the nominal radius alone does
not make every nearby celestial body known.

The 29 Gaia fields include exact decimal DR3 IDs, reference epoch, RA/Dec and
their errors, parallax/error, proper motion/errors, radial velocity/error,
quality flags, all ten astrometric correlations and G magnitude. Source strings,
units and missing values are preserved. RA/Dec are source ICRS astrometry and the
reference Julian year is J2016.0; no implicit galactic-frame/time-scale conversion,
inverse-parallax position or missing-velocity zero is introduced. JPL retains its
own declared centers/frames/epochs. [Gaia ingestion](universe-gaia-ingestion.md)
and [observed sources](observed-source-coverage.md) own the scientific adapters.

PS rows join a Gaia batch only by an explicit published DR3 ID. Hosts absent
from the raw Gaia selection go to a separate dependency-closed batch; no name or
nearest-neighbor merge is performed. Independent Proxima/Barnard exact-ID and
parallax/error/epoch checks retain their source associations. Full original PS
rows are retained locally for provenance, while only the distance-selected rows
are published in this nearby pack.

## Location and reproducibility

Bulk files are private development artifacts in this workspace:

- `artifacts/nearby-524-v1/sources/`: original TAP VOTables and string-preserving
  CSVs, deterministically gzip-archived; count/tail responses, exact queries/URLs,
  field metadata, hashes, reused PS/JPL snapshots, selection and build request.
- `artifacts/nearby-524-v1/pack/`: production content-addressed offline pack and
  its manifest. Source snapshot plus this one installed pack must fit the 1 GB
  cap; unused capacity is left available for a future larger selection.
- `artifacts/nearby-524-v1.tar`: local portable bundle of both, when produced by
  the delivery commands. Repository coverage metadata binds the retained inputs
  and output; large source rows/blobs are not redistributed through this public
  repository. Private source use is authorized; #659 still owns any future
  public/commercial dataset distribution review.

Reacquire into a **new** directory; server-generated raw XML bytes can differ on
a later run even for the same source rows. For exact replay, use the frozen
bundle and its hashes instead of fetching new bytes.

```sh
python scripts/acquire_nearby_sources.py \
  --output artifacts/nearby-524-v1/sources --revision nearby-524-v1
# On an interrupted, incomplete acquisition, repeat with identical arguments plus --resume.
python scripts/verify_nearby_sources.py --sources artifacts/nearby-524-v1/sources
cargo run -p salimon-universe --example nearby_verify --release --locked -- \
  artifacts/nearby-524-v1/sources artifacts/nearby-524-v1/pack
python scripts/verify_nearby_sources.py \
  --sources artifacts/nearby-524-v1/sources --pack artifacts/nearby-524-v1/pack
```

Acquisition pages by `(parallax DESC, source_id ASC)`, retaining exact source
parallax text in the continuation query. Initial/final regional counts, strict
advancing keys, distinct IDs and an empty final page establish query completeness;
TAP ERROR/OVERFLOW, changed units/columns, incomplete counts and budget overrun
fail rather than silently truncate the region. The initial source-ID-only plan
hit the server's statement timeout; parallax-indexed paging retains the same
selection. Source gzip hashes bind stored bytes, and recorded original-response
hashes bind exact inflated bytes. The independent verifier compares every CSV
field against original VOTable strings and every published observation against
the frozen source row.

## Pack and runtime boundaries

The [v2 pack](universe-pack.md) losslessly compresses sidecars and reads legacy
raw v1 packs. Its root manifest admits at most 64 MiB for this bounded regional
catalog; v1 keeps its original 16 MiB bound. Per-cell raw catalogs remain 16 MiB,
raw observed batches remain 8 MiB after gzip decoding, and there are at most
4096 cells. Builder identity checks are incremental rather than repeatedly
cloning the growing catalog; final publication is fully validated and atomic.

The filesystem reader exercises every cell/sidecar and exact-ID goldens. It still
holds the bounded root/ID index separately from its catalog cache. This is not
a billion-source storage design. Cell `[0,0,0]` keys and shard numbers are neutral
offline batch addresses, **not physical star locations**. Source astrometry is
retained for a later explicit coordinate/inference adapter. Downloading this
catalog does not replace the ordinary playable world, generate planets, reveal
player navigation, grant backend persistence or establish scientific/hardware
qualification outside the recorded source checks.

## Expanding later

1. Preserve `nearby-524-v1`, its exact inputs/coverage record and bundle hashes.
   Choose an explicit new boundary and publication revision, for example a lower
   parallax threshold for a larger nominal region. Keep source IDs stable.
2. Count/measure the proposed selection and choose a reviewed data budget before
   acquisition. The current tool enforces a 1 GB maximum for successful Gaia
   response bytes and frozen sources; independent final verification additionally
   caps the source snapshot plus installed pack together. A larger selection
   must fit or receive a new explicit budget/scope decision, never row truncation.
3. Acquire to a separate directory with `--parallax-min-mas` and `--revision`.
   Keep exact queries, original bytes/hashes, units/errors/correlations, missing
   values, quality decisions and exact-ID cross-match evidence. A new epoch,
   Gaia release or different selection must not silently reuse an old coverage claim.
4. Rebuild/reverify through production code, measure actual installed bytes,
   decoder throughput and RSS, and compare preserved IDs/observations against
   the previous publication. Update this guide, coverage metadata and report in
   the same reviewed PR. Use explicit migrations if identities deliberately change.
5. If the 64 MiB root index, per-cell limits, memory or overall budget are reached,
   implement/verify a scalable paged catalog/index instead of claiming that raising
   one cap solves full-Gaia storage. The earlier full-release scope and 2–5/10 GB
   constraints remain historical future-expansion requirements, not passes from
   this 100 pc publication. Public/commercial redistribution still needs #659.
