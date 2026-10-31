"""Freeze release-specific Gaia DR3 inputs for private scientific/resource checks.

The bounded random-index sample is a measurement sample, not full-release delivery.
Original VOTable bytes, queries, hashes and field metadata are retained alongside
losslessly transcribed CSV. Public/commercial source review remains #659.
"""
import argparse
import csv
from datetime import datetime, timezone
import hashlib
import io
import json
from pathlib import Path
import urllib.parse
import urllib.request
import xml.etree.ElementTree as ET

ENDPOINT = 'https://gaia.aip.de/tap/sync'
COLUMNS = ['source_id', 'random_index', 'ref_epoch', 'ra', 'ra_error', 'dec', 'dec_error',
           'parallax', 'parallax_error', 'pmra', 'pmra_error', 'pmdec', 'pmdec_error',
           'radial_velocity', 'radial_velocity_error', 'ruwe', 'duplicated_source',
           'astrometric_params_solved', 'ra_dec_corr', 'ra_parallax_corr', 'ra_pmra_corr',
           'ra_pmdec_corr', 'dec_parallax_corr', 'dec_pmra_corr', 'dec_pmdec_corr',
           'parallax_pmra_corr', 'parallax_pmdec_corr', 'pmra_pmdec_corr', 'phot_g_mean_mag']
GOLDENS = {'4472832130942575872': 'Barnard\u2019s Star', '5853498713190525696': 'Proxima Centauri'}
LIMIT = 16 * 1024 * 1024


def decode(data):
    root = ET.fromstring(data)
    infos = root.findall('.//{*}INFO')
    statuses = [i.attrib.get('value') for i in infos if i.attrib.get('name') == 'QUERY_STATUS']
    if not statuses or any(s != 'OK' for s in statuses):
        raise ValueError('TAP query failed or truncated: ' + str(statuses))
    tables = root.findall('.//{*}TABLE')
    if len(tables) != 1:
        raise ValueError('expected one TABLE')
    fields = [dict(f.attrib) for f in tables[0].findall('{*}FIELD')]
    names = [f['name'] for f in fields]
    if len(names) != len(set(names)):
        raise ValueError('duplicate fields')
    rows = []
    for row in tables[0].findall('.//{*}TR'):
        values = [c.text or '' for c in row.findall('{*}TD')]
        if len(values) != len(names):
            raise ValueError('VOTable row width')
        rows.append(dict(zip(names, values)))
    return fields, rows


def fetch(url):
    with urllib.request.urlopen(url, timeout=45) as response:
        data = response.read(LIMIT + 1)
        if response.status != 200 or not data or len(data) > LIMIT:
            raise ValueError('empty/oversize/failed TAP response')
        return data


def freeze(output, fetcher=fetch, sample_rows=8192):
    if not 1 <= sample_rows <= 16384:
        raise ValueError('sample_rows must be 1..16384')
    output.mkdir(parents=True, exist_ok=False)
    timestamp = datetime.now(timezone.utc).isoformat()
    acquisitions, artifacts = [], []
    datasets = [('sample', f'select {",".join(COLUMNS)} from gaiadr3.gaia_source where random_index < {sample_rows} order by source_id'),
                ('golden', f'select {",".join(COLUMNS)} from gaiadr3.gaia_source where source_id in ({",".join(GOLDENS)}) order by source_id')]
    for name, query in datasets:
        url = ENDPOINT + '?' + urllib.parse.urlencode(dict(REQUEST='doQuery', LANG='ADQL', FORMAT='votable', QUERY=query, MAXREC=sample_rows + 2))
        raw = fetcher(url)
        (output / f'{name}.xml').write_bytes(raw)
        fields, rows = decode(raw)
        if [f['name'] for f in fields] != COLUMNS or len(rows) != (sample_rows if name == 'sample' else len(GOLDENS)):
            raise ValueError('TAP fields/count differ from exact selection')
        ids = [row['source_id'] for row in rows]
        if len(set(ids)) != len(ids) or any(not i.isdecimal() or str(int(i)) != i for i in ids):
            raise ValueError('duplicate or non-exact source IDs')
        if name == 'golden' and set(ids) != set(GOLDENS):
            raise ValueError('golden IDs missing')
        stream = io.StringIO(newline='')
        writer = csv.DictWriter(stream, COLUMNS, lineterminator='\n')
        writer.writeheader()
        writer.writerows(rows)
        data = stream.getvalue().encode()
        (output / f'{name}.csv').write_bytes(data)
        for suffix, payload, encoding in [('xml', raw, 'votable'), ('csv', data, 'csv')]:
            artifacts.append(dict(path=f'{name}.{suffix}', bytes=len(payload), sha256=hashlib.sha256(payload).hexdigest(), origin='observed_snapshot', encoding=encoding))
        acquisitions.append(dict(name=name, url=url, query=query, rows=len(rows), fields=fields,
                                 csv_transcription='TABLEDATA strings in source FIELD order; missing TD remains empty; no numeric reformatting'))
    source = dict(id='gaia:dr3', release='Gaia DR3 (gaiadr3.gaia_source; reference epoch J2016.0)', endpoint=ENDPOINT,
                  query='; '.join(q for _, q in datasets), documentation=['https://www.cosmos.esa.int/web/gaia/dr3', 'https://doi.org/10.17876/gaia/dr.3/1'],
                  license=dict(status='unreviewed', policy_url='https://www.cosmos.esa.int/web/gaia/credits', attribution='ESA Gaia mission; Gaia DPAC; Gaia Collaboration (2023), A&A 674 A1; AIP Gaia Archive mirror.', review_issue='https://github.com/mr-exception/salimon/issues/659'),
                  coverage=f'Exact deterministic random_index < {sample_rows} sample and two explicit golden IDs; not a full Gaia release.',
                  uncertainty='Published astrometric standard errors/correlations and missing values retained; no distance or unmeasured phase space invented.', artifacts=artifacts)
    write = lambda name, value: (output / name).write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
    write('manifest.json', dict(format='salimon.universe.sources', schema_version=1, reviewed_on=timestamp[:10], sources=[source]))
    write('selection.json', dict(version=1, retrieved_at=timestamp, acquisitions=acquisitions, golden_identifications=GOLDENS,
                                private_authorization='https://github.com/mr-exception/salimon/issues/524', full_release_complete=False))
    # Keep sample/golden builds separate: a golden may also occur in the sample.
    for name, _ in datasets:
        matches = [[f'gaia_dr3:{key}', f'stellar-golden:{"barnard" if "Barnard" in value else "proxima"}'] for key, value in GOLDENS.items()] if name == 'golden' else []
        write(f'build-{name}.json', dict(version=1, revision=f'gaia-524-{name}-v1', source_manifest='manifest.json', cells=[dict(key=dict(galactic=[0, 0, 0], shard=0), observed=dict(gaia=dict(source='gaia:dr3', artifact=f'{name}.csv'), stellar_cross_matches=matches))]))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--sample-rows', type=int, default=8192)
    args = parser.parse_args()
    freeze(args.output, sample_rows=args.sample_rows)
    print(json.dumps(dict(snapshot=str(args.output), sampled_rows=args.sample_rows, full_release_complete=False)))


if __name__ == '__main__':
    main()
