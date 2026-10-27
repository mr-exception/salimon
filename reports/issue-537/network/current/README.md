# Source-access retry — 2026-10-27

All three original provider URLs now return HTTP 200 using the current managed
environment's inherited proxy/TLS configuration with command network access.
Six retrieved pages are retained as lossless `.html.gz` originals. Decompress
with `gzip -dc FILE.html.gz`; each JSON sidecar records the exact URL, UTC time,
final URL, response byte count/SHA-256 and compressed archive SHA-256. The
`body_sha256` hashes the decompressed original response. No input catalogs changed.

| Evidence | Official original | Retained record |
| --- | --- | --- |
| Gaia license | [ESA](https://www.cosmos.esa.int/web/gaia-users/license) | [metadata](gaia-license.json), [original](gaia-license.html.gz) |
| Gaia credits | [ESA](https://www.cosmos.esa.int/web/gaia-users/credits) | [metadata](gaia-credits.json), [original](gaia-credits.html.gz) |
| ESA archive terms | [ESA](https://www.cosmos.esa.int/web/esdc/terms-and-conditions) | [metadata](esa-archive-terms.json), [original](esa-archive-terms.html.gz) |
| Horizons manual | [JPL](https://ssd.jpl.nasa.gov/horizons/manual.html) | [metadata](jpl-horizons-manual.json), [original](jpl-horizons-manual.html.gz) |
| Linked JPL image policy | [JPL](https://www.jpl.nasa.gov/jpl-image-use-policy/) | [metadata](jpl-use-policy.json), [original](jpl-use-policy.html.gz) |
| Exoplanet acknowledgments | [NASA Exoplanet Archive](https://exoplanetarchive.ipac.caltech.edu/docs/acknowledge.html) | [metadata](exoplanet-acknowledgments.json), [original](exoplanet-acknowledgments.html.gz) |
| Linked CC full legal code | [Creative Commons](https://creativecommons.org/licenses/by-nc/3.0/igo/legalcode) | [HTTP 403 record](gaia-license-legalcode.json); no content obtained |

## Findings to bring to the human review

- Gaia identifies CC BY-NC 3.0 IGO. ESA's archive terms require an authorization
  request before commercial use, including uses directly or indirectly generating
  financial gain, and identify Gaia's credit as ESA, Gaia DPAC. The Gaia credits
  page links release-specific acknowledgments/citations; those linked release
  pages and the failed full legal code are not archived here. Commercial reuse
  cannot be approved from public availability or this retrieval.
- The Horizons manual documents models, limitations, references and contributors.
  Its linked JPL image policy concerns images/video and does not by itself
  establish rights to ephemeris data or contributed scientific material. Exact
  selected source/model/reference review remains required.
- The Exoplanet page requests its standard archive acknowledgment and direct
  acknowledgment of specific literature references. It now identifies Christiansen
  et al. (2025) as replacing Akeson et al. (2013). Selected dataset/original-reference
  requirements must still be checked; an acknowledgment page alone is not a
  blanket reuse grant.

This is retrieved evidence and an AI reading of its contents. No human reviewer,
approval date, production permission, full snapshot or balance acceptance is
inferred. The private-development-only decision and restricted/unreviewed source
statuses remain in force. Complete the [actual decision checklist](../../decisions.md)
against exact selected inputs before production staging. No provider was contacted
to request permission.
