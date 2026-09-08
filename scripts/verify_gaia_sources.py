"""Independently verify frozen TAP bytes, transcription, golden IDs and pack scope."""
import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
from acquire_gaia_sources import COLUMNS, GOLDENS, decode


def verify(root, ps_path):
    manifest = json.loads((root / 'manifest.json').read_text())
    source = manifest['sources'][0]
    if source['id'] != 'gaia:dr3' or 'DR3' not in source['release']:
        raise ValueError('wrong Gaia release')
    for artifact in source['artifacts']:
        name = Path(artifact['path'])
        if name.is_absolute() or len(name.parts) != 1:
            raise ValueError('artifact path')
        data = (root / name).read_bytes()
        if len(data) != artifact['bytes'] or hashlib.sha256(data).hexdigest() != artifact['sha256']:
            raise ValueError('source bytes/hash mismatch')
    results = {}
    for name in ('sample', 'golden'):
        fields, raw_rows = decode((root / f'{name}.xml').read_bytes())
        rows = list(csv.DictReader(io.StringIO((root / f'{name}.csv').read_text())))
        if [f['name'] for f in fields] != COLUMNS or rows != raw_rows:
            raise ValueError('CSV changed original TABLEDATA strings')
        if len({r['source_id'] for r in rows}) != len(rows):
            raise ValueError('duplicate source identity')
        results[name] = rows
    golden = {r['source_id']: r for r in results['golden']}
    if set(golden) != set(GOLDENS):
        raise ValueError('missing golden source')
    ranges = {'4472832130942575872': (546, 548), '5853498713190525696': (767, 769)}
    ps = list(csv.DictReader(ps_path.open()))
    matches = []
    for key, row in golden.items():
        lo, hi = ranges[key]
        if not lo < float(row['parallax']) < hi or float(row['ref_epoch']) != 2016.0:
            raise ValueError('golden astrometry/epoch differs from expected physical source')
        if not float(row['parallax_error']) > 0:
            raise ValueError('golden uncertainty missing')
        hosts = sorted({r['hostname'] for r in ps if r['gaia_dr3_id'] == 'Gaia DR3 ' + key})
        expected = "Barnard's star" if 'Barnard' in GOLDENS[key] else 'Proxima Cen'
        if hosts != [expected]:
            raise ValueError('PS exact DR3 host association ambiguous/missing')
        matches.append(dict(gaia_dr3_id=key, ps_hosts=hosts, parallax_mas=row['parallax'], parallax_error_mas=row['parallax_error']))
    selection = json.loads((root / 'selection.json').read_text())
    sample_count = selection['acquisitions'][0]['rows']
    if len(results['sample']) != sample_count or sorted(int(r['random_index']) for r in results['sample']) != list(range(sample_count)):
        raise ValueError('random-index sample incomplete')
    return dict(status='passed', input_rows=sample_count, exact_golden_cross_matches=matches,
                source_manifest_sha256=hashlib.sha256((root/'manifest.json').read_bytes()).hexdigest(),
                ps_sha256=hashlib.sha256(ps_path.read_bytes()).hexdigest(), full_release_complete=False)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, required=True)
    parser.add_argument('--ps', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(verify(args.sources, args.ps), indent=2))


if __name__ == '__main__':
    main()
