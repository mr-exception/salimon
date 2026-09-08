#!/usr/bin/env python3
"""Reproducible first-party Rust metrics. Lexical proxies, not a Rust compiler."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
# Blank comments and literals before counting tokens. Nested block comments are
# handled explicitly; line positions are retained for path-specific diagnostics.
TOKEN = re.compile(r'//[^\n]*|/\*|r(\#*)"|"|\b(?:if|for|while|loop|match)\b|&&|\|\|')


def code_only(source):
    chars = list(source)
    pos = 0
    while match := TOKEN.search(source, pos):
        start, end = match.span()
        token = match[0]
        if token == '/*':
            depth = 1
            while depth and end < len(source):
                nested = re.search(r'/\*|\*/', source[end:])
                if not nested:
                    end = len(source)
                    break
                end += nested.end()
                depth += 1 if nested[0] == '/*' else -1
        elif token.startswith('r'):
            stop = source.find('"' + match[1], end)
            end = len(source) if stop < 0 else stop + 1 + len(match[1])
        elif token == '"':
            while end < len(source):
                if source[end] == '\\':
                    end += 2
                elif source[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
        elif not token.startswith('//'):
            pos = end
            continue
        chars[start:end] = ['\n' if c == '\n' else ' ' for c in source[start:end]]
        pos = end
    return ''.join(chars)


def cycles(graph):
    """Return strongly connected components, rather than every possible cycle."""
    index, stack, active, indices, low, found = 0, [], set(), {}, {}, []

    def visit(node):
        nonlocal index
        indices[node] = low[node] = index
        index += 1
        stack.append(node)
        active.add(node)
        for target in sorted(graph.get(node, [])):
            if target not in indices:
                visit(target)
                low[node] = min(low[node], low[target])
            elif target in active:
                low[node] = min(low[node], indices[target])
        if low[node] == indices[node]:
            component = []
            while True:
                member = stack.pop()
                active.remove(member)
                component.append(member)
                if member == node:
                    break
            if len(component) > 1 or node in graph.get(node, []):
                found.append(sorted(component))

    for node in sorted(graph):
        if node not in indices:
            visit(node)
    return sorted(found)


def git(root, *args):
    return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()


def audit(root, history_ref='HEAD', limit=30):
    files, graph, imports = [], defaultdict(set), []
    paths = sorted(p for p in (root / 'client').glob('*/src/**/*.rs')
                   if 'generated' not in p.parts and p.name != 'spatial_contracts.rs')
    crates = {p.parent.name: re.search(r'^name\s*=\s*"([^"]+)"', p.read_text(), re.M)[1]
              .replace('-', '_') for p in (root / 'client').glob('*/Cargo.toml')}
    for path in paths:
        rel = path.relative_to(root).as_posix()
        source = path.read_text()
        code = code_only(source)
        owner = crates[path.relative_to(root).parts[1]]
        graph[owner]  # Include isolated crates.
        targets = sorted(set(re.findall(r'\b(salimon_\w+)\s*::', code)))
        graph[owner].update(t for t in targets if t in crates.values() and t != owner)
        uses = sorted(set(re.findall(r'\buse\s+([^;]+);', code)))
        imports.extend({'path': rel, 'import': ' '.join(use.split())} for use in uses)
        files.append({'path': rel, 'owner': owner, 'lines': len(source.splitlines()),
                      'bytes': len(source.encode()),
                      'functions': len(re.findall(r'\bfn\s+\w+', code)),
                      'branch_tokens': len(re.findall(r'\b(?:if|for|while|loop|match)\b|&&|\|\|', code)),
                      'module_declarations': sorted(set(re.findall(r'\bmod\s+(\w+)', code))),
                      'crate_references': targets})
    # Squash and merge subjects with a trailing PR number. This is local history
    # evidence, not proof that every commit was merged through a GitHub PR.
    commits = git(root, 'log', '--first-parent', '--format=%H %s', history_ref).splitlines()
    prs, counts, pairs = [], Counter(), Counter()
    eligible = {f['path'] for f in files}
    for commit in commits:
        sha, subject = commit.split(' ', 1)
        match = re.search(r'\(#(\d+)\)$', subject)
        if not match:
            continue
        changed = sorted(set(git(root, 'diff-tree', '--root', '--no-commit-id',
                                 '--name-only', '-r', '-m', '--first-parent', sha).splitlines()) & eligible)
        prs.append({'pr': int(match[1]), 'commit': sha, 'paths': changed})
        counts.update(changed)
        pairs.update((a, b) for i, a in enumerate(changed) for b in changed[i + 1:])
        if len(prs) >= limit:
            break
    return {'schema': 1, 'history_ref': git(root, 'rev-parse', history_ref),
            'history_limit': limit, 'files': files, 'imports': imports,
            'crate_graph': {k: sorted(v) for k, v in sorted(graph.items())},
            'crate_cycles': cycles(graph), 'recent_pr_commits': prs,
            'cochange_counts': dict(sorted(counts.items(), key=lambda p: (-p[1], p[0]))),
            'cochange_pairs': [{'paths': [a, b], 'count': n}
                               for (a, b), n in sorted(pairs.items(), key=lambda p: (-p[1], p[0]))]}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--history-ref', default='HEAD')
    parser.add_argument('--history-limit', type=int, default=30)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--summary', action='store_true', help='print warning-only review candidates')
    args = parser.parse_args()
    if args.history_limit < 1:
        parser.error('--history-limit must be positive')
    try:
        report = audit(args.root.resolve(), args.history_ref, args.history_limit)
    except (OSError, subprocess.CalledProcessError, ValueError) as error:
        parser.exit(1, f'architecture audit failed: {error}\n')
    payload = json.dumps(report, indent=2) + '\n'
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(payload)
    else:
        print(payload, end='')
    if args.summary:
        for item in report['files']:
            count = report['cochange_counts'].get(item['path'], 0)
            if item['lines'] >= 1000 or item['branch_tokens'] >= 200 or count >= 10:
                print(f"Review {item['path']}: {item['lines']} lines, "
                      f"{item['branch_tokens']} branch tokens, {count} recent PR commits")
        for cycle in report['crate_cycles']:
            print('Review crate cycle: ' + ', '.join(cycle))


if __name__ == '__main__':
    main()
