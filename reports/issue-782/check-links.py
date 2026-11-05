"""Check changed guides' local paths/anchors and pinned historical paths offline."""
from pathlib import Path
import re, subprocess, urllib.parse
root = Path(__file__).resolve().parents[2]
files = subprocess.check_output(['git', 'diff', '--name-only', 'origin/main', '--', '*.md'], cwd=root, text=True).splitlines()
files += ['docs/current-client-status.md', 'reports/README.md', 'reports/issue-782/README.md']
errors, count = [], 0
for name in sorted(set(files)):
    path = root / name
    if not path.exists(): continue
    text = re.sub(r'```.*?```', '', path.read_text(), flags=re.S)
    for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', text):
        if target.startswith('https://github.com/mr-exception/salimon/blob/8f112bb'):
            revision, historical = target.split('/blob/', 1)[1].split('/', 1)
            if subprocess.run(['git', 'cat-file', '-e', revision + ':' + historical.split('#')[0]], cwd=root, stderr=subprocess.DEVNULL).returncode:
                errors.append(f'{name}: missing archive {target}')
            count += 1
            continue
        if re.match(r'[a-z]+:', target): continue
        local, _, fragment = urllib.parse.unquote(target).partition('#')
        resolved = path.parent / local if local else path
        count += 1
        if not resolved.exists(): errors.append(f'{name}: missing {target}'); continue
        if fragment and resolved.suffix == '.md':
            headings, seen = [], {}
            for heading in re.findall(r'^#{1,6}\s+(.+)', resolved.read_text(), re.M):
                slug = re.sub(r'[^\w\- ]', '', heading.lower().replace('`', '')).replace(' ', '-')
                n = seen.get(slug, 0); seen[slug] = n + 1
                headings.append(slug + (f'-{n}' if n else ''))
            if fragment not in headings: errors.append(f'{name}: missing anchor {target}')
print(f'{len(set(files))} guides; {count} links; {len(errors)} errors')
print('\n'.join(errors))
raise SystemExit(bool(errors))
