"""Freeze a bounded, all-sky nearby Gaia selection and local observed companions.

The 1 GB cap is a maximum, not a request to pad or silently truncate a region.
Parallax selects observations, not error-free geometric distances.
"""
import argparse
import csv
from datetime import datetime, timezone
import gzip
import hashlib
import io
import json
import math
from pathlib import Path
import re
import time
import urllib.error
import urllib.parse
import urllib.request

from acquire_gaia_sources import COLUMNS, ENDPOINT, GOLDENS, decode

ROOT = Path(__file__).resolve().parents[1]
BUDGET = 1_000_000_000


def write_json(path, value):
    pending = path.with_name(path.name + '.pending')
    pending.write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
    pending.replace(path)


def fetch(url):
    for attempt in range(3):
        try:
            with urllib.request.urlopen(url, timeout=45) as response:
                data = response.read(16 * 1024 * 1024 + 1)
                if response.status != 200 or not data or len(data) > 16 * 1024 * 1024:
                    raise ValueError('empty/oversize/failed TAP response')
                return data
        except (urllib.error.URLError, TimeoutError):
            if attempt == 2:
                raise
            time.sleep(attempt + 1)


def csv_bytes(fields, rows):
    stream = io.StringIO(newline='')
    writer = csv.DictWriter(stream, fields, lineterminator='\n')
    writer.writeheader()
    writer.writerows(rows)
    return stream.getvalue().encode()


def artifact(root, name, raw, encoding):
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    data = gzip.compress(raw, mtime=0) if name.endswith('.gz') else raw
    path.write_bytes(data)
    return dict(path=name, bytes=len(data), sha256=hashlib.sha256(data).hexdigest(),
                origin='observed_snapshot', encoding=encoding)


def freeze_gaia(output, parallax_min_mas=10.0, batch_rows=8192, budget=BUDGET,
                resume=False, fetcher=fetch):
    if (not math.isfinite(parallax_min_mas) or parallax_min_mas <= 0
            or not 1 <= batch_rows <= 8192 or not 0 < budget <= BUDGET):
        raise ValueError('positive parallax, 1..8192 batch rows and <=1 GB budget required')
    output.mkdir(parents=True, exist_ok=resume)
    params = dict(parallax_min_mas=parallax_min_mas, batch_rows=batch_rows, budget=budget,
                  order='parallax_desc_source_id_asc')
    if resume:
        state = json.loads((output / 'progress.json').read_text())
        if state['parameters'] != params or (output / 'selection.json').exists():
            raise ValueError('resume parameters differ or snapshot is already complete')
        # Do not trust a stale watermark if any frozen page has changed.
        for item in state['artifacts']:
            data = (output / item['path']).read_bytes()
            if len(data) != item['bytes'] or hashlib.sha256(data).hexdigest() != item['sha256']:
                raise ValueError('resume source artifact changed')
    else:
        state = dict(version=1, retrieved_at=datetime.now(timezone.utc).isoformat(),
                     parameters=params, artifacts=[], pages=[], queries=[], received_response_bytes=0)
    def query(sql, name):
        url = ENDPOINT + '?' + urllib.parse.urlencode(dict(REQUEST='doQuery', LANG='ADQL',
                                                          FORMAT='votable', QUERY=sql, MAXREC=batch_rows + 1))
        raw = fetcher(url)
        state['received_response_bytes'] += len(raw)
        if state['received_response_bytes'] > budget:
            raise ValueError('nearby acquisition exceeds 1 GB response budget; do not truncate')
        fields, rows = decode(raw)
        item = artifact(output, name + '.xml.gz', raw, 'votable_gzip')
        state['artifacts'].append(item)
        state['queries'].append(dict(query=sql, url=url, artifact=item['path'],
                                     raw_bytes=len(raw), raw_sha256=hashlib.sha256(raw).hexdigest()))
        return fields, rows
    predicate = f'parallax >= {parallax_min_mas:.17g}'
    count_sql = f'SELECT COUNT(*) AS n FROM gaiadr3.gaia_source WHERE {predicate}'
    def count(name):
        fields, rows = query(count_sql, name)
        if [f['name'] for f in fields] != ['n'] or len(rows) != 1 or not rows[0]['n'].isascii() or not rows[0]['n'].isdecimal():
            raise ValueError('invalid selected-query count')
        return int(rows[0]['n'])
    if not resume:
        state['expected_rows'] = count('count-start')
        if not state['expected_rows']:
            raise ValueError('empty nearby region')
        write_json(output / 'progress.json', state)
    total = sum(p['rows'] for p in state['pages'])
    last = state['pages'][-1]['last_source_id'] if state['pages'] else 0
    last_parallax = state['pages'][-1]['last_parallax'] if state['pages'] else None
    seen = set()
    for page in state['pages']:
        with gzip.open(output / page['csv'], 'rt', newline='') as stream:
            seen.update(r['source_id'] for r in csv.DictReader(stream))
    def continuation():
        if last_parallax is None:
            return ''
        return f' AND (parallax < {last_parallax} OR (parallax = {last_parallax} AND source_id > {last}))'
    while total < state['expected_rows']:
        index = len(state['pages'])
        sql = f'SELECT TOP {batch_rows} {",".join(COLUMNS)} FROM gaiadr3.gaia_source WHERE {predicate}{continuation()} ORDER BY parallax DESC,source_id'
        fields, rows = query(sql, f'gaia/{index:04}')
        if [f['name'] for f in fields] != COLUMNS or len(rows) != min(batch_rows, state['expected_rows'] - total):
            raise ValueError('changed columns or incomplete keyset page')
        if state.get('fields', fields) != fields:
            raise ValueError('changed source FIELD metadata')
        state['fields'] = fields
        ids = []
        previous_key = (-float(last_parallax), last) if last_parallax is not None else None
        for row in rows:
            key = row['source_id']
            if not key.isascii() or not key.isdecimal() or str(int(key)) != key or not 0 < int(key) < 2**64:
                raise ValueError('noncanonical Gaia source ID')
            source_id = int(key)
            parallax = float(row['parallax'])
            current_key = (-parallax, source_id)
            if key in seen or (previous_key is not None and current_key <= previous_key) or not math.isfinite(parallax) or parallax < parallax_min_mas:
                raise ValueError('duplicate/nonadvancing key or source outside nearby query')
            seen.add(key)
            previous_key = current_key
            ids.append(source_id)
        item = artifact(output, f'gaia/{index:04}.csv.gz', csv_bytes(COLUMNS, rows), 'csv_gzip')
        state['artifacts'].append(item)
        state['pages'].append(dict(index=index, rows=len(rows), first_source_id=ids[0],
                                   last_source_id=ids[-1], last_parallax=rows[-1]['parallax'],
                                   csv=item['path'], xml=f'gaia/{index:04}.xml.gz'))
        total += len(rows)
        last = ids[-1]
        last_parallax = rows[-1]['parallax']
        write_json(output / 'progress.json', state)
        print(json.dumps(dict(gaia_pages=len(state['pages']), received_rows=total,
                              expected_rows=state['expected_rows'])), flush=True)
    _, tail = query(f'SELECT TOP 1 source_id FROM gaiadr3.gaia_source WHERE {predicate}{continuation()} ORDER BY parallax DESC,source_id', 'tail')
    if tail or count('count-end') != state['expected_rows']:
        raise ValueError('selection changed or keyset coverage is incomplete')
    state.update(selected_query_complete=True, full_release_complete=False,
                 scope=dict(all_sky=True, parallax_min_mas=parallax_min_mas,
                            nominal_parsecs=1000 / parallax_min_mas,
                            meaning='Observed parallax cut, not a complete or error-free distance sphere; Gaia ICRS/J2016.0.'))
    if sum(a['bytes'] for a in state['artifacts']) > budget:
        raise ValueError('frozen snapshot exceeds budget')
    write_json(output / 'selection.json', state)
    return state


def prepare(output, observed=ROOT / 'catalogs/sources/observed-526-v1', revision='nearby-524-v1'):
    """Partition exact nearby PS associations with their Gaia host batches."""
    if not re.fullmatch('[A-Za-z0-9._-]{1,128}', revision):
        raise ValueError('explicit safe publication revision required')
    selection = json.loads((output / 'selection.json').read_text())
    radius = selection['scope']['nominal_parsecs']
    id_chunk = {}
    for page in selection['pages']:
        with gzip.open(output / page['csv'], 'rt', newline='') as stream:
            for row in csv.DictReader(stream):
                if row['source_id'] in id_chunk:
                    raise ValueError('duplicate Gaia source while routing planets')
                id_chunk[row['source_id']] = page['index']
    if not set(GOLDENS).issubset(id_chunk):
        raise ValueError('nearby snapshot lacks the independent golden stars')
    original = json.loads((observed / 'manifest.json').read_text())
    ps_source = next(s for s in original['sources'] if s['id'] == 'nasa:exoplanet-ps')
    ps_artifact = next(a for a in ps_source['artifacts'] if a['path'] == 'ps.csv')
    ps_raw = (observed / 'ps.csv').read_bytes()
    if len(ps_raw) != ps_artifact['bytes'] or hashlib.sha256(ps_raw).hexdigest() != ps_artifact['sha256']:
        raise ValueError('original PS snapshot hash changed')
    ps_rows = list(csv.DictReader(io.StringIO(ps_raw.decode())))
    fields = list(ps_rows[0])
    groups, excluded = {}, 0
    for row in ps_rows:
        distance = row['sy_dist']
        if not distance or not math.isfinite(float(distance)) or not 0 < float(distance) <= radius:
            excluded += 1
            continue
        gaia = row['gaia_dr3_id']
        key = id_chunk.get(gaia.removeprefix('Gaia DR3 '), 'extra') if gaia else 'extra'
        groups.setdefault(key, []).append(row)
    ps_source = dict(ps_source, artifacts=[artifact(output, 'observed/ps-full.csv.gz', ps_raw, 'csv_gzip')])
    ps_paths = {}
    for key, rows in groups.items():
        name = f'observed/ps-{key}.csv.gz'
        ps_source['artifacts'].append(artifact(output, name, csv_bytes(fields, rows), 'csv_gzip'))
        ps_paths[key] = name
    gaia_template = json.loads((ROOT / 'catalogs/sources/gaia-524-v1/manifest.json').read_text())['sources'][0]
    gaia_source = dict(gaia_template, artifacts=selection['artifacts'],
                       query='; '.join(q['query'] for q in selection['queries']),
                       coverage=f'Complete all-sky observed parallax >= {selection["scope"]["parallax_min_mas"]:g} mas query: {selection["expected_rows"]} rows before quality selection. Nominal {radius:g} pc, not a complete physical sphere.')
    sources = [gaia_source, ps_source]
    horizons = json.loads((observed / 'build.json').read_text())['cells'][0]['observed']['horizons']
    for source in original['sources']:
        if not source['id'].startswith('jpl:'):
            continue
        entries = []
        for old in source['artifacts']:
            raw = (observed / old['path']).read_bytes()
            if len(raw) != old['bytes'] or hashlib.sha256(raw).hexdigest() != old['sha256']:
                raise ValueError('original JPL snapshot hash changed')
            entries.append(artifact(output, 'observed/' + old['path'], raw, old['encoding']))
        sources.append(dict(source, artifacts=entries))
    write_json(output / 'manifest.json', dict(format='salimon.universe.sources', schema_version=1,
               reviewed_on=selection['retrieved_at'][:10], sources=sources))
    cells = []
    for page in selection['pages']:
        raw = dict(gaia=dict(source='gaia:dr3', artifact=page['csv']))
        if page['index'] in ps_paths:
            raw['exoplanets'] = dict(source='nasa:exoplanet-ps', artifact=ps_paths[page['index']])
        cells.append(dict(key=dict(galactic=[0, 0, 0], shard=page['index']), observed=raw))
    extra = dict(horizons=[dict(h, artifact='observed/' + h['artifact']) for h in horizons])
    if 'extra' in ps_paths:
        extra['exoplanets'] = dict(source='nasa:exoplanet-ps', artifact=ps_paths['extra'])
    cells.append(dict(key=dict(galactic=[0, 0, 0], shard=len(cells)), observed=extra))
    write_json(output / 'build.json', dict(version=1, revision=revision, source_manifest='manifest.json', cells=cells))
    selection['publication_revision'] = revision
    selection['companions'] = dict(ps_nominal_distance_max_pc=radius, selected_planets=sum(map(len, groups.values())),
                                    excluded_ps_rows=excluded, full_ps_rows=len(ps_rows), solar_system_targets=len(horizons),
                                    ps_groups={str(k): len(v) for k, v in groups.items()},
                                    storage_keys='Neutral batch/shard keys, not physical galactic positions; source astrometry/frames are preserved.')
    write_json(output / 'selection.json', selection)
    frozen = sum(p.stat().st_size for p in output.rglob('*') if p.is_file())
    if frozen > selection['parameters']['budget']:
        raise ValueError('combined frozen source snapshot exceeds 1 GB')
    return dict(source_directory=str(output), frozen_bytes=frozen, gaia_rows=selection['expected_rows'],
                companions=selection['companions'], full_release_complete=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--parallax-min-mas', type=float, default=10.0)
    parser.add_argument('--batch-rows', type=int, default=8192)
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--revision', default='nearby-524-v1')
    args = parser.parse_args()
    freeze_gaia(args.output, args.parallax_min_mas, args.batch_rows, resume=args.resume)
    print(json.dumps(prepare(args.output, revision=args.revision), indent=2))


if __name__ == '__main__':
    main()
