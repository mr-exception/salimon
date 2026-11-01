"""Independent offline coverage/transcription/association checks for nearby sources."""
import argparse
import csv
import gzip
import hashlib
import io
import json
import math
from pathlib import Path
import xml.etree.ElementTree as ET

from acquire_gaia_sources import COLUMNS, GOLDENS, decode
from import_universe_sources import verify as verify_manifest
from verify_observed_sources import read_blob as read_pack_blob

ROOT = Path(__file__).resolve().parents[1]


def inflated(path):
    with gzip.open(path, 'rb') as stream:
        raw = stream.read(16 * 1024 * 1024 + 1)
    if len(raw) > 16 * 1024 * 1024:
        raise ValueError('source decode exceeds bounded acquisition response size')
    return raw


def rows(path):
    return list(csv.DictReader(io.StringIO(inflated(path).decode())))


def pack_checks(root, pack, selection, id_chunk):
    """Compare every published observation's source fields to the frozen inputs."""
    def blob(item):
        return read_pack_blob(pack, item)
    with (pack / 'manifest.json').open('rb') as stream:
        raw = stream.read(64 * 1024 * 1024 + 1)
    if len(raw) > 64 * 1024 * 1024:
        raise ValueError('oversize nearby manifest')
    manifest = json.loads(raw)
    if manifest['format'] != 'salimon.universe.private-pack' or manifest['version'] != 2 or manifest['revision'] != selection['publication_revision'] or manifest['production_eligible']:
        raise ValueError('wrong pack scope/revision/version')
    if blob(manifest['source_manifest']) != (root / 'manifest.json').read_bytes():
        raise ValueError('pack not bound to exact nearby source manifest')
    pages = selection['pages']
    if [c['key'] for c in manifest['cells']] != [dict(galactic=[0, 0, 0], shard=i) for i in range(len(pages) + 1)]:
        raise ValueError('missing/extra regional storage batch')
    accepted = rejected = published_planets = 0
    for index, cell in enumerate(manifest['cells']):
        sidecars = cell['sidecars']
        if index < len(pages):
            original = rows(root / pages[index]['csv'])
            expected = {}
            for row in original:
                quality = (row['ruwe'] and float(row['ruwe']) > 1.4) or row['duplicated_source'] in ('true', 'True', '1')
                coords = (row['ra'] and row['dec'] and row['ref_epoch']
                          and 0 <= float(row['ra']) < 360 and -90 <= float(row['dec']) <= 90
                          and 1 <= float(row['ref_epoch']) < 10000)
                if quality or not coords:
                    rejected += 1
                else:
                    expected[row['source_id']] = row
            actual = json.loads(blob(sidecars['gaia_observations']))
            observed = [o for group in actual.values() for o in group]
            by_id = {o['source_fields']['source_id']: o for o in observed}
            if len(by_id) != len(observed) or by_id.keys() != expected.keys():
                raise ValueError('selected Gaia sources missing, duplicated or added in pack')
            for key, observation in by_id.items():
                if observation['source'] != 'gaia_dr3:' + key or observation['source_fields'] != expected[key]:
                    raise ValueError('published Gaia original fields/uncertainties changed')
            summary = json.loads(blob(sidecars['gaia_selection']))
            if summary['policy'] != dict(max_ruwe=1.4, reject_duplicated_source=True) or summary['summary']['selected_rows'] != len(expected):
                raise ValueError('published selection policy/count changed')
            accepted += len(expected)
        if 'planet_observations' in sidecars:
            group = index if index < len(pages) else 'extra'
            expected = {r['pl_name']: r for r in rows(root / f'observed/ps-{group}.csv.gz')}
            actual = json.loads(blob(sidecars['planet_observations']))
            by_name = {o['source_fields']['pl_name']: o for o in actual}
            if len(by_name) != len(actual) or by_name.keys() != expected.keys() or any(o['source_fields'] != expected[k] for k, o in by_name.items()):
                raise ValueError('published planet source fields/uncertainties changed')
            published_planets += len(actual)
    if accepted + rejected != len(id_chunk) or published_planets != selection['companions']['selected_planets']:
        raise ValueError('published coverage differs from nearby selection')
    installed = (pack / 'manifest.json').stat().st_size + sum(p.stat().st_size for p in (pack / 'blobs').iterdir())
    frozen = sum(p.stat().st_size for p in root.rglob('*') if p.is_file())
    if installed + frozen > selection['parameters']['budget']:
        raise ValueError('combined source snapshot and installed pack exceed 1 GB')
    return dict(gaia_selected_rows=accepted, gaia_rejected_rows=rejected, published_ps_planets=published_planets,
                installed_pack_bytes=installed, combined_snapshot_and_pack_bytes=installed + frozen,
                every_published_source_field_matches_original=True)


def verify(root, observed=ROOT / 'catalogs/sources/observed-526-v1', pack=None):
    registered_manifest, _ = verify_manifest(root / 'manifest.json')
    selection = json.loads((root / 'selection.json').read_text())
    if not selection['selected_query_complete'] or selection['full_release_complete']:
        raise ValueError('wrong completion scope')
    minimum = selection['parameters']['parallax_min_mas']
    if not math.isfinite(minimum) or minimum <= 0 or selection['parameters']['order'] != 'parallax_desc_source_id_asc':
        raise ValueError('invalid region/order policy')
    radius = 1000 / minimum
    if selection['scope']['parallax_min_mas'] != minimum or selection['scope']['nominal_parsecs'] != radius or selection['scope']['all_sky'] is not True:
        raise ValueError('scope differs from selected observation boundary')
    pinned = json.loads((ROOT / 'catalogs/sources/gaia-524-v1/selection.json').read_text())['acquisitions'][0]['fields']
    signature = lambda fields: [(f['name'], f.get('unit'), f.get('datatype')) for f in fields]
    if signature(selection['fields']) != signature(pinned):
        raise ValueError('Gaia release units/datatypes changed')
    by_query = {q['artifact']: q for q in selection['queries']}
    registered = {a['path'] for s in registered_manifest['sources'] for a in s['artifacts']}
    referenced = set(by_query) | {p[k] for p in selection['pages'] for k in ('xml', 'csv')}
    if not referenced.issubset(registered):
        raise ValueError('selection references an artifact outside the verified source manifest')
    response_bytes = 0
    for name, q in by_query.items():
        raw = inflated(root / name)
        if len(raw) != q['raw_bytes'] or hashlib.sha256(raw).hexdigest() != q['raw_sha256']:
            raise ValueError('original TAP response hash changed')
        tree = ET.fromstring(raw)
        published = [i.attrib.get('value') for i in tree.findall('.//{*}INFO') if i.attrib.get('name') == 'QUERY']
        if published != [q['query']]:
            raise ValueError('original TAP query differs from acquisition record')
        response_bytes += len(raw)
    for name in ('count-start.xml.gz', 'count-end.xml.gz'):
        if by_query[name]['query'] != f'SELECT COUNT(*) AS n FROM gaiadr3.gaia_source WHERE parallax >= {minimum:.17g}':
            raise ValueError('count query does not measure the documented region')
        fields, count_rows = decode(inflated(root / name))
        if [f['name'] for f in fields] != ['n'] or count_rows != [{'n': str(selection['expected_rows'])}]:
            raise ValueError('changed selected-query counts')
    if decode(inflated(root / 'tail.xml.gz'))[1]:
        raise ValueError('unexpected remaining keyset rows')
    id_chunk, golden = {}, {}
    last_key = None
    last_parallax = None
    last_source_id = 0
    for page in selection['pages']:
        continuation = '' if last_parallax is None else f' AND (parallax < {last_parallax} OR (parallax = {last_parallax} AND source_id > {last_source_id}))'
        expected_query = f'SELECT TOP {selection["parameters"]["batch_rows"]} {",".join(COLUMNS)} FROM gaiadr3.gaia_source WHERE parallax >= {minimum:.17g}{continuation} ORDER BY parallax DESC,source_id'
        if by_query[page['xml']]['query'] != expected_query:
            raise ValueError('page query does not measure the documented region/continuation')
        fields, original = decode(inflated(root / page['xml']))
        transcribed = rows(root / page['csv'])
        if [f['name'] for f in fields] != COLUMNS or fields != selection['fields'] or original != transcribed or len(original) != page['rows']:
            raise ValueError('source columns/metadata/transcription/count changed')
        for row in transcribed:
            key = row['source_id']
            if not key.isascii() or not key.isdecimal() or str(int(key)) != key or not 0 < int(key) < 2**64 or key in id_chunk:
                raise ValueError('source IDs missing, duplicated or noncanonical')
            parallax = float(row['parallax'])
            if not math.isfinite(parallax) or parallax < minimum:
                raise ValueError('source outside explicit observed parallax cut')
            current_key = (-parallax, int(key))
            if last_key is not None and current_key <= last_key:
                raise ValueError('nonadvancing parallax/source key')
            last_key = current_key
            id_chunk[key] = page['index']
            if key in GOLDENS:
                golden[key] = row
        if int(original[0]['source_id']) != page['first_source_id'] or int(original[-1]['source_id']) != page['last_source_id'] or original[-1]['parallax'] != page['last_parallax']:
            raise ValueError('keyset page bounds differ from original rows')
        last_source_id = page['last_source_id']
        last_parallax = page['last_parallax']
    tail_query = f'SELECT TOP 1 source_id FROM gaiadr3.gaia_source WHERE parallax >= {minimum:.17g} AND (parallax < {last_parallax} OR (parallax = {last_parallax} AND source_id > {last_source_id})) ORDER BY parallax DESC,source_id'
    if by_query['tail.xml.gz']['query'] != tail_query:
        raise ValueError('tail query does not measure the documented region/continuation')
    if len(id_chunk) != selection['expected_rows'] or len(golden) != len(GOLDENS):
        raise ValueError('regional query or independent golden coverage incomplete')
    ps_raw = inflated(root / 'observed/ps-full.csv.gz')
    if ps_raw != (observed / 'ps.csv').read_bytes():
        raise ValueError('original NASA PS snapshot changed')
    ps_all = list(csv.DictReader(io.StringIO(ps_raw.decode())))
    groups = {}
    for row in ps_all:
        value = row['sy_dist']
        if value and math.isfinite(float(value)) and 0 < float(value) <= radius:
            gaia = row['gaia_dr3_id']
            group = id_chunk.get(gaia.removeprefix('Gaia DR3 '), 'extra') if gaia else 'extra'
            groups.setdefault(group, []).append(row)
    for key, expected in groups.items():
        if rows(root / f'observed/ps-{key}.csv.gz') != expected:
            raise ValueError('nearby PS rows or exact host batch association changed')
    if sum(map(len, groups.values())) != selection['companions']['selected_planets']:
        raise ValueError('nearby planet count mismatch')
    ranges = {'4472832130942575872': (546, 548), '5853498713190525696': (767, 769)}
    associations = []
    for key, row in golden.items():
        lo, hi = ranges[key]
        expected_host = "Barnard's star" if 'Barnard' in GOLDENS[key] else 'Proxima Cen'
        hosts = sorted({r['hostname'] for r in ps_all if r['gaia_dr3_id'] == 'Gaia DR3 ' + key})
        if hosts != [expected_host] or not lo < float(row['parallax']) < hi or not float(row['parallax_error']) > 0 or float(row['ref_epoch']) != 2016:
            raise ValueError('independent exact-ID/epoch/parallax association failed')
        associations.append(dict(source_id=key, ps_hosts=hosts, parallax_mas=row['parallax'], parallax_error_mas=row['parallax_error']))
    source_manifest = json.loads((root / 'manifest.json').read_text())
    original_manifest = json.loads((observed / 'manifest.json').read_text())
    original_jpl = {s['id']: s for s in original_manifest['sources'] if s['id'].startswith('jpl:')}
    for source in source_manifest['sources']:
        if source['id'] in original_jpl:
            for old, new in zip(original_jpl[source['id']]['artifacts'], source['artifacts'], strict=True):
                if (root / new['path']).read_bytes() != (observed / old['path']).read_bytes():
                    raise ValueError('original Solar System source bytes changed')
    frozen_bytes = sum(p.stat().st_size for p in root.rglob('*') if p.is_file())
    if frozen_bytes > selection['parameters']['budget'] or response_bytes != selection['received_response_bytes'] or response_bytes > selection['parameters']['budget']:
        raise ValueError('frozen snapshot or successful response bytes exceed budget')
    result = dict(status='passed', selected_query_complete=True, full_release_complete=False,
                gaia_input_rows=len(id_chunk), pages=len(selection['pages']), all_sky=True,
                parallax_min_mas=minimum, nominal_parsecs=radius,
                selected_ps_planets=sum(map(len, groups.values())), solar_system_targets=len(original_jpl),
                response_bytes=response_bytes, frozen_source_bytes=frozen_bytes,
                exact_golden_associations=associations,
                source_manifest_sha256=hashlib.sha256((root / 'manifest.json').read_bytes()).hexdigest())
    if pack is not None:
        result['production_pack'] = pack_checks(root, pack, selection, id_chunk)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', required=True, type=Path)
    parser.add_argument('--pack', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.sources, pack=args.pack), indent=2))


if __name__ == '__main__':
    main()
