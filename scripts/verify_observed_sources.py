"""Independently compare frozen JPL/PS bytes to a built private pack (#526).

No network. Every source, blob, raw field, vector component and PS numeric/error/
limit conversion is checked. Numerical budgets measure decoder arithmetic only,
not provider accuracy, interpolation, catalog completeness or release permission.
"""
import argparse
from collections import Counter
import csv
from datetime import datetime
import hashlib
import io
import json
import math
from pathlib import Path
import re
import statistics
import struct
import zlib

from acquire_observed_sources import PARAMETERS, TARGETS, count_query

CONVERSIONS = {
    'sy_dist': ('host_distance_meters', 3.085677581491367e16),
    'st_mass': ('host_mass_kg', 1.98847e30), 'st_rad': ('host_radius_meters', 695700000.),
    'pl_massj': ('planet_mass_kg', 1.89813e27), 'pl_msinij': ('minimum_mass_kg', 1.89813e27),
    'pl_radj': ('planet_radius_meters', 71492000.), 'pl_orbper': ('orbital_period_seconds', 86400.),
    'pl_orbsmax': ('semi_major_axis_meters', 149597870700.), 'pl_orbeccen': ('eccentricity', 1.),
    'pl_orbincl': ('inclination_radians', math.pi / 180.),
    'pl_orblper': ('argument_of_periastron_radians', math.pi / 180.),
    'pl_orbtper': ('periastron_time_source_days', 1.),
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def number_equal(actual, expected, context):
    require(math.isfinite(actual) and math.isclose(actual, expected, rel_tol=2e-15, abs_tol=1e-12), context)


def read_blob(pack, blob):
    require(re.fullmatch('[0-9a-f]{64}', blob['sha256']) is not None, 'invalid blob path')
    data = (pack / 'blobs' / blob['sha256']).read_bytes()
    require(len(data) == blob['bytes'] and digest(data) == blob['sha256'], 'blob size/hash mismatch')
    return data


def catalog_records(data):
    require(data[:8] == b'SALUCELL', 'catalog magic')
    version, length, total, crc, reserved = struct.unpack('<IIQII', data[8:32])
    require(version == 1 and total == len(data) and reserved == 0, 'catalog header')
    index_bytes = data[32:32 + length]
    require(zlib.crc32(index_bytes) == crc, 'catalog index CRC')
    index = json.loads(index_bytes)
    records = []
    for cell in index['cells']:
        offset = 32 + length + cell['offset']
        compressed = data[offset:offset + cell['compressed_bytes']]
        decoder = zlib.decompressobj()
        raw = decoder.decompress(compressed) + decoder.flush()
        require(decoder.eof and not decoder.unused_data, 'catalog zlib extent')
        require(len(raw) == cell['raw_bytes'] and zlib.crc32(compressed) == cell['crc32'], 'catalog payload CRC')
        records.extend(dict(kind=r['kind'], **r['record']) for r in json.loads(raw)['records'])
    return records


def verify(snapshot, pack):
    source_bytes = (snapshot / 'manifest.json').read_bytes()
    sources = json.loads(source_bytes)
    selection = json.loads((snapshot / 'selection.json').read_bytes())
    manifest_bytes = (pack / 'manifest.json').read_bytes()
    manifest = json.loads(manifest_bytes)
    require(manifest['production_eligible'] is False, 'private scope required')
    require(read_blob(pack, manifest['source_manifest']) == source_bytes, 'source manifest binding')
    source_sizes = {}
    for source in sources['sources']:
        require(source['license']['status'] == 'unreviewed', 'release review must remain pending')
        for artifact in source['artifacts']:
            path = Path(artifact['path'])
            require(not path.is_absolute() and len(path.parts) == 1, 'source path')
            data = (snapshot / path).read_bytes()
            require(digest(data) == artifact['sha256'] and len(data) == artifact['bytes'], 'raw source hash/size')
            source_sizes[str(path)] = len(data)
    acquisitions = {a['path']: a for a in selection['acquisition']}
    for source in sources['sources']:
        for artifact in source['artifacts']:
            a = acquisitions[artifact['path']]
            expected_query = count_query() if artifact['path'] == 'ps-count.csv' else source['query']
            require(a['url'] == expected_query and a['sha256'] == artifact['sha256'] and a['bytes'] == artifact['bytes'], 'acquisition binding')
    require(len(manifest['cells']) == 1, 'selected source inventory cell')
    cell = manifest['cells'][0]
    records = catalog_records(read_blob(pack, cell['catalog']))
    by_id = {r['header']['id']: r for r in records}
    require(len(records) == len(by_id) and sorted(by_id) == cell['record_ids'], 'complete unique catalog index')
    sidecars = {k: json.loads(read_blob(pack, b)) for k, b in cell['sidecars'].items()}
    rows = list(csv.DictReader(io.StringIO((snapshot / 'ps.csv').read_text())))
    counts = list(csv.DictReader(io.StringIO((snapshot / 'ps-count.csv').read_text())))
    require(len(counts) == 1 and int(counts[0]['n']) == len(rows), 'PS count query differs from snapshot')
    observations = sidecars['planet_observations']
    require(len(rows) == len(observations) == selection['ps_rows'], 'PS full selected row coverage')
    observed = {o['source_fields']['pl_name']: o for o in observations}
    require(len(observed) == len(rows), 'PS unique keys')
    missing, limits, max_relative = Counter(), Counter(), Counter()
    hosts = Counter()
    distributions = {key: dict(values=[], relative_minus=[], relative_plus=[]) for key in PARAMETERS}
    for row in rows:
        o = observed[row['pl_name']]
        require(o['source_fields'] == row, 'PS raw columns must survive exactly')
        require(o['literature_reference'] == row['pl_refname'] and bool(row['pl_refname']), 'PS reference')
        require(row['default_flag'] == '1', 'PS default row')
        planet = 'nea_planet:' + row['pl_name'].encode().hex()
        host = ('gaia_dr3:' + row['gaia_dr3_id'].removeprefix('Gaia DR3 ') if row['gaia_dr3_id'] else
                'nea_host:' + row['hostname'].encode().hex())
        require(o['planet'] == planet and o['host'] == host, 'PS exact source identity')
        require(by_id[planet]['parent'] == host, 'PS canonical parent')
        require(by_id[planet]['kind'] == 'body' and by_id[planet]['body_kind'] == 'planet', 'PS body classification')
        require(by_id[host]['kind'] == 'star' and by_id[planet]['system'] == by_id[host]['system'], 'PS host system')
        for source_field, canonical_field in [('planet_mass_kg', 'mass_kg'), ('planet_radius_meters', 'radius_meters')]:
            reported = o[source_field]
            promoted = reported is not None and reported['limit'] == 'Measurement' and (reported['error_minus_si'] is None or reported['error_minus_si'] < reported['quantity']['value'])
            require(by_id[planet][canonical_field] == (reported['quantity'] if promoted else None), 'PS canonical quantity promotion')
        hosts[host] += 1
        for key in PARAMETERS:
            field, factor = CONVERSIONS[key]
            q = o[field]
            if not row[key]:
                require(q is None, 'PS missing quantity fabricated')
                missing[key] += 1
                continue
            expected = float(row[key]) * factor
            number_equal(q['quantity']['value'], expected, f'PS conversion {key}')
            distributions[key]['values'].append(expected)
            relative = abs(q['quantity']['value'] - expected) / max(abs(expected), 1e-300)
            max_relative[key] = max(max_relative[key], relative)
            require(q['quantity']['provenance'] == dict(kind='observed', dataset='nasa:exoplanet-ps', record_key=row['pl_name']), 'PS provenance')
            require(q['quantity']['validity'] is None, 'PS physical validity invented')
            for side, sign, output in [('err1', 1, 'error_plus_si'), ('err2', -1, 'error_minus_si')]:
                value = row[key + side]
                if value:
                    number_equal(q[output], float(value) * factor * sign, f'PS signed error {key}')
                    if expected != 0:
                        distributions[key]['relative_plus' if sign == 1 else 'relative_minus'].append(q[output] / abs(expected))
                else:
                    require(q[output] is None, 'PS absent error fabricated')
            expected_limit = {'': 'Unspecified', '0': 'Measurement', '1': 'Upper', '-1': 'Lower'}[row.get(key + 'lim', '')]
            require(q['limit'] == expected_limit, 'PS limit changed')
            limits[key + ':' + expected_limit] += 1
            if row[key + 'err1'] and row[key + 'err2']:
                require(q['quantity']['uncertainty']['kind'] == 'bounds', 'PS asymmetric bounds lost')
                number_equal(q['quantity']['uncertainty']['plus'], q['error_plus_si'], 'PS upper bound')
                number_equal(q['quantity']['uncertainty']['minus'], q['error_minus_si'], 'PS lower bound')
            else:
                require(q['quantity']['uncertainty'] == dict(kind='unknown'), 'PS unknown error fabricated')
    ephemerides = {e['target']: e for e in sidecars['ephemerides']}
    require(set(ephemerides) == {f'jpl:{code}' for code, *_ in TARGETS}, 'JPL selected target coverage')
    vectors, max_position_error, max_velocity_error = 0, 0., 0.
    physical_missing, source_models = [], {}
    for code, name, kind, parent in TARGETS:
        e = ephemerides[f'jpl:{code}']
        raw = json.loads((snapshot / f'jpl-{code}.json').read_bytes())['result']
        require(e['raw_response'] == raw, 'JPL original model/reference/header text changed')
        body = by_id[e['target']]
        require(body['kind'] == ('star' if kind == 'star' else 'body'), 'JPL star/body classification')
        if kind != 'star':
            require(body['parent'] == parent and body['body_kind'] == kind, 'JPL parent/classification')
        require(body['system'] == 'system:solar', 'JPL system identity')
        require(body['mass_kg'] == e['mass_kg'] and body['radius_meters'] == e['mean_radius_meters'], 'JPL physical publication mismatch')
        if e['mass_kg'] is None:
            physical_missing.append(name + ':mass')
        source_models[str(code)] = [line for line in raw.splitlines() if line.startswith(('Target body name:', 'Center body name:'))]
        block = raw.split('$$SOE\n')[1].split('$$EOE')[0]
        lines = [line.strip() for line in block.splitlines() if line.strip()]
        require(len(lines) == 9 and len(e['samples']) == 3, 'JPL selected epoch coverage')
        for index, sample in enumerate(e['samples']):
            date_row = lines[index * 3]
            jd, calendar = date_row.split(' = A.D. ')
            calendar = datetime.strptime(calendar.strip().removesuffix(' TDB'), '%Y-%b-%d %H:%M:%S.%f')
            expected_jd = 2451545. + (calendar - datetime(2000, 1, 1, 12)).total_seconds() / 86400.
            number_equal(float(jd), expected_jd, 'JPL calendar/JD consistency')
            require(sample['julian_day_tdb'] == float(jd) and sample['frame'] == 'IcrfSolarSystemBarycenterGeometric', 'JPL frame/time label')
            epoch_day = (calendar.date() - datetime(1970, 1, 1).date()).days
            epoch_nanos = ((calendar.hour * 3600 + calendar.minute * 60 + calendar.second) * 1_000_000_000 + calendar.microsecond * 1000)
            require(sample['epoch'] == dict(day=epoch_day, nanos=epoch_nanos, scale='tdb'), 'JPL split epoch')
            for line, labels, field in [(lines[index * 3 + 1], ('X', 'Y', 'Z'), 'position_meters'),
                                         (lines[index * 3 + 2], ('VX', 'VY', 'VZ'), 'velocity_mps')]:
                matches = re.findall(r'([A-Z]+)\s*=\s*([^\s]+)', line)
                require(tuple(k for k, _ in matches) == labels, 'JPL independent vector labels')
                for actual, (_, value) in zip(sample[field], matches):
                    expected = float(value) * 1000.
                    number_equal(actual, expected, 'JPL SI component mismatch')
                    error = abs(actual - expected)
                    if field == 'position_meters':
                        max_position_error = max(max_position_error, error)
                    else:
                        max_velocity_error = max(max_velocity_error, error)
            vectors += 1
    # Known real multi-planet and no-Gaia/multi-star hosts remain explicit sources.
    golden_names = ['Proxima Cen b', 'TRAPPIST-1 b', 'TRAPPIST-1 h', 'Kepler-16 b', 'PSR B1257+12 b']
    for name in golden_names:
        require(name in observed, 'missing selected golden ' + name)
    unique_blobs = {b['sha256']: b['bytes'] for b in [manifest['source_manifest'], cell['catalog'], *cell['sidecars'].values()]}
    distributions = {key: {name: dict(count=len(values), min=min(values) if values else None, median=statistics.median(values) if values else None, max=max(values) if values else None) for name, values in fields.items()} for key, fields in distributions.items()}
    return dict(status='passed', source_manifest_sha256=digest(source_bytes), pack_manifest_sha256=digest(manifest_bytes),
                source_bytes=source_sizes, ps=sidecars['planet_summary'],
                ps_reported_distributions=distributions, ps_missing_quantities=dict(missing), ps_limit_counts=dict(limits), ps_max_relative_conversion_error=dict(max_relative),
                multi_planet_hosts=sum(count > 1 for count in hosts.values()),
                multi_star_rows=sum(int(r['sy_snum'] or 1) > 1 for r in rows),
                goldens={name: dict(planet=observed[name]['planet'], host=observed[name]['host']) for name in golden_names},
                jpl=dict(targets=len(ephemerides), samples=vectors, max_component_error_meters=max_position_error,
                         max_component_error_mps=max_velocity_error, missing_mass=physical_missing, source_models=source_models,
                         coverage_tdb='2000-01-01 00:00:00 through 2000-01-02 00:00:00; exact 12h samples only'),
                storage=dict(records=len(records), partition_bytes=cell['catalog']['bytes'],
                             sidecar_bytes={k: b['bytes'] for k, b in cell['sidecars'].items()},
                             index_manifest_bytes=len(manifest_bytes), unique_blob_bytes=sum(unique_blobs.values())),
                limitations=['Private source-inventory cell; not a located full-galaxy spatial index.',
                             'No all-small-body Solar System claim; selected targets enumerated explicitly.',
                             'PS orbital elements may be missing; phase/frame/epoch and multi-star hierarchy not invented.',
                             'Exact Gaia source IDs only; independent Gaia release cross-match is #524.',
                             'Decoder arithmetic errors are not scientific model errors or interpolation validation.',
                             'Public/commercial release terms remain unreviewed #659; no native/hardware qualification.'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--snapshot', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = verify(args.snapshot, args.pack)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(status=report['status'], planets=report['ps']['planets'], targets=report['jpl']['targets'],
                          report=str(args.output))))


if __name__ == '__main__':
    main()
