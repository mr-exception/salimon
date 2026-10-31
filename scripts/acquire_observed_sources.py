"""Freeze the explicit private JPL/PS selection for #526; no runtime downloads.

Writes a NEW directory only. The output retains original bytes and exact queries;
source terms stay unreviewed for public release (#659).
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

PS_ENDPOINT = 'https://exoplanetarchive.ipac.caltech.edu/TAP/sync'
JPL_ENDPOINT = 'https://ssd.jpl.nasa.gov/api/horizons.api'
# Exact body centers, never barycenters masquerading as planets.
TARGETS = [
    (10, 'Sun', 'star', 'system:solar'),
    (199, 'Mercury', 'planet', 'jpl:10'), (299, 'Venus', 'planet', 'jpl:10'),
    (399, 'Earth', 'planet', 'jpl:10'), (301, 'Moon', 'moon', 'jpl:399'),
    (499, 'Mars', 'planet', 'jpl:10'), (599, 'Jupiter', 'planet', 'jpl:10'),
    (501, 'Io', 'moon', 'jpl:599'), (502, 'Europa', 'moon', 'jpl:599'),
    (503, 'Ganymede', 'moon', 'jpl:599'), (504, 'Callisto', 'moon', 'jpl:599'),
    (699, 'Saturn', 'planet', 'jpl:10'), (606, 'Titan', 'moon', 'jpl:699'),
    (799, 'Uranus', 'planet', 'jpl:10'), (899, 'Neptune', 'planet', 'jpl:10'),
    (801, 'Triton', 'moon', 'jpl:899'), (999, 'Pluto', 'minor_body', 'jpl:10'),
    (901, 'Charon', 'moon', 'jpl:999'),
]
PARAMETERS = ['sy_dist', 'st_mass', 'st_rad', 'pl_massj', 'pl_msinij', 'pl_radj',
              'pl_orbper', 'pl_orbsmax', 'pl_orbeccen', 'pl_orbincl', 'pl_orblper', 'pl_orbtper']


def queries():
    columns = ['pl_name', 'hostname', 'default_flag', 'gaia_dr3_id', 'pl_refname',
               'sy_snum', 'sy_pnum', 'pl_bmassprov']
    for key in PARAMETERS:
        columns += [key, key + 'err1', key + 'err2']
        if key != 'sy_dist':
            columns.append(key + 'lim')
    ps = PS_ENDPOINT + '?' + urllib.parse.urlencode({
        'query': 'select ' + ','.join(columns) + ' from ps where default_flag=1 order by hostname,pl_name',
        'format': 'csv'})
    jpl = {}
    for code, *_ in TARGETS:
        jpl[code] = JPL_ENDPOINT + '?' + urllib.parse.urlencode({
            'format': 'json', 'COMMAND': f"'{code}'", 'CENTER': "'500@0'", 'EPHEM_TYPE': "'VECTORS'",
            'START_TIME': "'2000-01-01'", 'STOP_TIME': "'2000-01-02'", 'STEP_SIZE': "'12h'",
            'OUT_UNITS': "'KM-S'", 'REF_PLANE': "'FRAME'", 'VEC_TABLE': "'2'"})
    return ps, jpl


def count_query():
    return PS_ENDPOINT + '?' + urllib.parse.urlencode({
        'query': 'select count(*) as n from ps where default_flag=1', 'format': 'csv'})


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')


def source(source_id, endpoint, query, path, data, scope, timestamp):
    jpl = endpoint == JPL_ENDPOINT
    return dict(id=source_id, release=f'Private observed selection retrieved {timestamp}',
                endpoint=endpoint, query=query,
                documentation=(["https://ssd.jpl.nasa.gov/horizons/manual.html",
                                "https://ssd-api.jpl.nasa.gov/doc/horizons.html"] if jpl else
                               ["https://exoplanetarchive.ipac.caltech.edu/docs/API_PS_columns.html",
                                "https://exoplanetarchive.ipac.caltech.edu/docs/acknowledge.html"]),
                license=dict(status='unreviewed',
                             policy_url=('https://ssd.jpl.nasa.gov/horizons/manual.html' if jpl else
                                         'https://exoplanetarchive.ipac.caltech.edu/docs/acknowledge.html'),
                             attribution=('NASA/JPL Solar System Dynamics / Horizons; exact source model and references retained.' if jpl else
                                          'NASA Exoplanet Archive, Caltech, NASA Exoplanet Exploration Program; Christiansen et al. (2025); all original pl_refname references retained.'),
                             review_issue='https://github.com/mr-exception/salimon/issues/659'),
                coverage=scope, uncertainty='Original fields, source errors, missing values and references retained; no complete-state covariance or unmeasured quantity invented.',
                artifacts=[dict(path=path, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                                origin='observed_snapshot', encoding='json' if jpl else 'csv')])


def freeze(output, fetch):
    output.mkdir(parents=True, exist_ok=False)
    timestamp = datetime.now(timezone.utc).isoformat()
    ps, jpl = queries()
    sources, requests, acquisitions = [], [], []

    def get(url, name):
        data = fetch(url)
        if not data or len(data) > 8 * 1024 * 1024:
            raise ValueError('source response empty or exceeds selected batch bound')
        (output / name).write_bytes(data)
        acquisitions.append(dict(url=url, path=name, sha256=hashlib.sha256(data).hexdigest(), bytes=len(data)))
        return data

    count_data = get(count_query(), 'ps-count.csv')
    counts = list(csv.DictReader(io.StringIO(count_data.decode('utf-8'))))
    if len(counts) != 1 or set(counts[0]) != {'n'}:
        raise ValueError('invalid PS row-count response')
    expected_rows = int(counts[0]['n'])
    data = get(ps, 'ps.csv')
    rows = list(csv.DictReader(io.StringIO(data.decode('utf-8'))))
    if not rows or len(rows) != expected_rows or any(row.get('default_flag') != '1' for row in rows):
        raise ValueError('PS response must contain only nonempty default-row CSV')
    if len({r['pl_name'] for r in rows}) != len(rows):
        raise ValueError('PS snapshot has duplicate planet keys')
    sources.append(source('nasa:exoplanet-ps', PS_ENDPOINT, ps, 'ps.csv', data,
                          'All PS default_flag=1 rows at retrieval; selected science/error/limit/identity/reference columns; not PSCompPars or complete phase-space orbits.', timestamp))
    sources[0]['artifacts'].append(dict(path='ps-count.csv', bytes=len(count_data),
                                         sha256=hashlib.sha256(count_data).hexdigest(),
                                         origin='observed_snapshot', encoding='csv'))
    for code, name, kind, parent in TARGETS:
        path = f'jpl-{code}.json'
        data = get(jpl[code], path)
        response = json.loads(data)
        if response.get('signature') != dict(source='NASA/JPL Horizons API', version='1.2') or '$$SOE' not in response.get('result', ''):
            raise ValueError(f'unsupported/missing Horizons result for {code}')
        source_id = f'jpl:horizons-{code}-j2000'
        sources.append(source(source_id, JPL_ENDPOINT, jpl[code], path, data,
                              f'{name} center {code}; explicit parent {parent}; 3 geometric ICRF/SSB KM-S TDB samples, 2000-01-01 to 2000-01-02 every 12h. Selected inventory, not all Solar System small bodies or complete model validity.', timestamp))
        requests.append(dict(source=source_id, artifact=path, system='system:solar',
                             parent=parent, target=f'jpl:{code}', object_kind=kind))
    write_json(output / 'manifest.json', dict(format='salimon.universe.sources', schema_version=1,
                                            reviewed_on=timestamp[:10], sources=sources))
    write_json(output / 'selection.json', dict(version=1, retrieved_at=timestamp, targets=TARGETS,
                                              ps_rows=len(rows), acquisition=acquisitions,
                                              private_authorization='https://github.com/mr-exception/salimon/issues/659'))
    # One bounded source cell. No invented spatial position for hosts without astrometry.
    write_json(output / 'build.json', dict(version=1, revision='observed-526-v1', source_manifest='manifest.json',
                                          cells=[dict(key=dict(galactic=[0, 0, 0], shard=0),
                                                      observed=dict(exoplanets=dict(source='nasa:exoplanet-ps', artifact='ps.csv'),
                                                                    horizons=requests))]))


def fetch(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        if response.status != 200:
            raise ValueError(f'HTTP status {response.status}')
        return response.read(8 * 1024 * 1024 + 1)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True, help='new snapshot directory')
    args = parser.parse_args()
    freeze(args.output, fetch)
    print(json.dumps(dict(snapshot=str(args.output), targets=len(TARGETS))))


if __name__ == '__main__':
    main()
