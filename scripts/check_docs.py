#!/usr/bin/env python3
"""Offline Markdown/owned-guide checks; deliberately not a semantic verifier."""
import argparse
import fnmatch
import json
import re
import subprocess
import sys
import unicodedata
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
LINK = re.compile(r'!?\[[^\]\n]*\]\(<?([^\s)>]+)>?(?:\s+"[^"]*")?\)|^\s*\[[^\]]+\]:\s*<?([^\s>]+)>?', re.M)


def prose(text):
    """Blank code fences, preserving lines for diagnostics."""
    fence = None
    out = []
    for line in text.splitlines():
        match = re.match(r'^\s*(`{3,}|~{3,})', line)
        if match:
            token = match[1]
            if fence is None:
                fence = token
            elif token[0] == fence[0] and len(token) >= len(fence):
                fence = None
            out.append('')
        else:
            out.append('' if fence else line)
    return '\n'.join(out)


def anchors(text):
    counts, result = {}, set(re.findall(r'<a\s+(?:id|name)=[\"\']([^\"\']+)', text))
    for line in prose(text).splitlines():
        match = re.match(r'^ {0,3}#{1,6}\s+(.+?)(?:\s+#+)?$', line)
        if not match:
            continue
        title = re.sub(r'!?\[([^\]]+)\]\([^)]*\)', r'\1', match[1])
        title = re.sub(r'<[^>]+>', '', title).lower()
        slug = ''.join(c for c in title if c in '-_ ' or unicodedata.category(c)[0] in 'LN').replace(' ', '-')
        count = counts.get(slug, 0)
        counts[slug] = count + 1
        result.add(slug + (f'-{count}' if count else ''))
    return result


def check(root, config, changed=None):
    errors, findings = [], set()
    exclusions = config.get('exclude', [])
    files = sorted({p for pattern in config['include'] for p in root.glob(pattern)
                    if p.is_file() and not any(fnmatch.fnmatch(p.relative_to(root).as_posix(), x) for x in exclusions)})
    for path in files:
        rel = path.relative_to(root).as_posix()
        text = prose(path.read_text(encoding='utf-8'))
        for match in LINK.finditer(text):
            dest = unquote(match[1] or match[2])
            parsed = urlsplit(dest)
            if parsed.scheme or parsed.netloc:
                continue
            line = text.count('\n', 0, match.start()) + 1
            target = (root / parsed.path.lstrip('/') if parsed.path.startswith('/') else path.parent / parsed.path).resolve()
            if not target.is_relative_to(root.resolve()):
                findings.add((rel, line, dest, 'link escapes repository'))
            elif not target.exists():
                findings.add((rel, line, dest, 'missing local target'))
            elif parsed.fragment and target.suffix.lower() == '.md' and parsed.fragment not in anchors(target.read_text(encoding='utf-8')):
                findings.add((rel, line, dest, 'missing Markdown anchor'))
        for rule in config.get('stale_rules', []):
            if any(fnmatch.fnmatch(rel, pattern) for pattern in rule['paths']):
                for n, line in enumerate(text.splitlines(), 1):
                    if re.search(rule['pattern'], line, re.I):
                        findings.add((rel, n, rule['pattern'], rule['reason']))
    # Exact baseline entries preserve dated debt without blanket ignores. A new or
    # moved bad reference fails, including inside previously indebted documents.
    baseline = {tuple(item) for item in config.get('baseline', [])}
    for rel, line, dest, reason in sorted(findings - baseline):
        errors.append(f'{rel}:{line}: {reason}: {dest}')
    for item in config.get('required_paths', []):
        if not (root / item).exists():
            errors.append(f'{item}:1: required owned guide/example missing')
    for item in config.get('json_examples', []):
        try:
            value = json.loads((root / item['path']).read_text(encoding='utf-8'))
            for key, expected in item['fields'].items():
                if value.get(key) != expected:
                    errors.append(f"{item['path']}:1: expected {key}={expected!r}")
        except (OSError, ValueError) as error:
            errors.append(f"{item['path']}:1: invalid JSON example: {error}")
    if changed is not None:
        for manifest in sorted(root.glob('client/*/Cargo.toml')):
            owner = manifest.parent.relative_to(root).as_posix()
            touched = [p for p in changed if p.startswith(owner + '/') and (p.endswith('.rs') or p.endswith('Cargo.toml'))]
            if not touched:
                continue
            for name in ('README.ai.md', 'architecture.md', 'invariants.md'):
                if not (manifest.parent / name).exists():
                    errors.append(f'{owner}/{name}:1: changed crate requires owned guide')
            docs = [p for p in changed if p.startswith(owner + '/') and p.endswith('.md') and (root / p).exists()]
            exceptions = config.get('no_impact', [])
            allowed = any(e.get('reason', '').strip() and all(any(fnmatch.fnmatch(p, g) for g in e['paths']) for p in touched) for e in exceptions)
            if not docs and not allowed:
                errors.append(f'{owner}:1: source changed without owning guide update or scoped no_impact reason in scripts/docs-policy.json')
    return errors, findings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', help='Git revision to compare for same-PR documentation impact')
    parser.add_argument('--write-baseline', action='store_true', help='explicit debt review only; never used in CI')
    args = parser.parse_args()
    policy = ROOT / 'scripts/docs-policy.json'
    config = json.loads(policy.read_text())
    changed = None
    if args.base:
        changed = subprocess.check_output(['git', 'diff', '--name-only', '--diff-filter=ACMRT', f'{args.base}...HEAD'], cwd=ROOT, text=True).splitlines()
    errors, findings = check(ROOT, config, changed)
    if args.write_baseline:
        config['baseline'] = sorted(findings)
        policy.write_text(json.dumps(config, indent=2) + '\n')
        print(f'Recorded {len(findings)} reviewed historical findings; rerun validation.')
        return 0
    for error in errors:
        print(error, file=sys.stderr)
    print(f'Documentation: {len(errors)} failures; {len(findings & {tuple(x) for x in config.get("baseline", [])})} exact historical findings retained.')
    return bool(errors)


if __name__ == '__main__':
    sys.exit(main())
