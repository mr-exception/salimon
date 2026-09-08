#!/usr/bin/env python3
"""Warning-only architecture review; lexical candidates require human assessment."""
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

from architecture_audit import audit, code_only, cycles, git

ROOT = Path(__file__).resolve().parents[1]


def base_text(root, base, path):
    result = subprocess.run(['git', '-C', str(root), 'show', f'{base}:{path}'],
                            capture_output=True, text=True, check=False)
    if result.returncode:
        # A newly added path is expected; a missing/unavailable base is not.
        git(root, 'rev-parse', '--verify', f'{base}^{{commit}}')
        return ''
    return result.stdout


def references(source):
    return set(re.findall(r'\b(salimon_\w+)\s*::', code_only(source)))


def review(root, base, policy, active_prs=None, current_pr=None):
    report = audit(root, limit=30)
    changed = set(git(root, 'diff', '--name-only', '--diff-filter=ACMR', base).splitlines())
    before, after = {}, report['crate_graph']
    findings, suppressed = [], []
    known = set(after)
    for path in git(root, 'ls-tree', '-r', '--name-only', base, '--', 'client').splitlines():
        parts = Path(path).parts
        if len(parts) < 4 or parts[2] != 'src' or not path.endswith('.rs') or 'generated' in parts or parts[-1] == 'spatial_contracts.rs':
            continue
        manifest = base_text(root, base, f'client/{parts[1]}/Cargo.toml')
        match = re.search(r'^name\s*=\s*"([^"]+)"', manifest, re.M)
        if match:
            owner = match[1].replace('-', '_')
            before.setdefault(owner, set()).update(references(base_text(root, base, path)) & known - {owner})

    def add(rule, path, detail):
        item = {'rule': rule, 'path': path, 'detail': detail}
        exceptions = [e for e in policy['exceptions'] if e['rule'] == rule and e['path'] == path
                      and e['detail'] == detail and e.get('reason') and e.get('owner')]
        (suppressed if exceptions else findings).append(item)

    allowed = policy['allowed_crate_edges']
    for item in report['files']:
        path, owner = item['path'], item['owner']
        targets = set(item['crate_references']) & known - {owner}
        for target in sorted(targets - set(allowed.get(owner, []))):
            add('direction', path, f'{owner} -> {target}; consult the domain ownership map')
        if path not in changed:
            continue
        old = base_text(root, base, path)
        for target in sorted(targets - references(old)):
            add('new-domain-reference', path, f'new reference to {target}; review contract ownership')
        code = code_only((root / path).read_text())
        old_code = code_only(old)
        imports = lambda s: set(' '.join(m.split()) for m in re.findall(r'\buse\s+([^;]+);', s))
        for use in sorted(imports(code) - imports(old_code)):
            if re.match(r'(crate|super|salimon_\w+)::', use):
                add('new-import', path, use)
        thresholds = policy['thresholds']
        if item['lines'] >= thresholds['lines'] or item['branch_tokens'] >= thresholds['branch_tokens']:
            add('broad-file', path, f"{item['lines']} lines, {item['branch_tokens']} branch tokens; size alone does not require extraction")
    old_cycles = cycles(before)
    for cycle in report['crate_cycles']:
        if cycle not in old_cycles:
            paths = sorted(item['path'] for item in report['files'] if item['owner'] in cycle and item['path'] in changed)
            for path in paths or ['client']:
                add('new-cycle', path, ' -> '.join(cycle))
    overlaps = []
    if active_prs is not None:
        for pr in active_prs:
            if pr['number'] == current_pr:
                continue
            shared = sorted(changed & set(pr['paths']))
            if shared:
                overlaps.append({'number': pr['number'], 'head_sha': pr['head_sha'],
                                 'draft': pr['draft'], 'paths': shared})
                for path in shared:
                    add('active-pr-overlap', path, f"PR #{pr['number']} ({'draft' if pr['draft'] else 'open'}) at {pr['head_sha']}; inspect actual hunks before stacking")
    return {'schema': 1, 'base': git(root, 'rev-parse', base), 'head': git(root, 'rev-parse', 'HEAD'),
            'changed_paths': sorted(changed), 'findings': findings, 'suppressed': suppressed,
            'new_cycles': [c for c in report['crate_cycles'] if c not in old_cycles],
            'active_prs_status': 'available' if active_prs is not None else 'unavailable',
            'overlaps': overlaps, 'warning_only': True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', required=True)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--policy', type=Path, default=ROOT / 'scripts/architecture-policy.json')
    parser.add_argument('--active-prs', type=Path)
    parser.add_argument('--current-pr', type=int)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    try:
        report = review(args.root, args.base, json.loads(args.policy.read_text()),
                        json.loads(args.active_prs.read_text()) if args.active_prs else None, args.current_pr)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2) + '\n')
        # Plain text avoids treating repository-derived strings as workflow commands.
        for finding in report['findings']:
            print(f"Review [{finding['rule']}] {finding['path']}: {finding['detail']}")
        print(f"Warning-only: {len(report['findings'])} candidates; active PR data {report['active_prs_status']}")
    except (OSError, ValueError, KeyError, TypeError, subprocess.CalledProcessError) as error:
        print(f'Architecture review unavailable: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
