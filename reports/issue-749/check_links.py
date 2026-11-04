"""Check local Markdown links added by this PR, without auditing legacy debt."""
import pathlib
import re
import subprocess
import sys
import urllib.parse

root = pathlib.Path(__file__).resolve().parents[2]
base = sys.argv[1]
diff = subprocess.check_output(
    ['git', 'diff', '--no-ext-diff', '--unified=0', base, '--', '*.md'],
    cwd=root, text=True,
)
source = None
count = 0
for line in diff.splitlines():
    if line.startswith('+++ b/'):
        source = root / line[6:]
    elif line.startswith('+') and not line.startswith('+++'):
        for target in re.findall(r'!?\[[^\]]*\]\(([^\s)]+)\)', line[1:]):
            parsed = urllib.parse.urlsplit(target)
            if parsed.scheme or parsed.netloc:
                continue
            assert source is not None
            path = (source.parent / urllib.parse.unquote(parsed.path)).resolve() if parsed.path else source
            assert path.is_relative_to(root) and path.exists(), (source, target)
            if parsed.fragment:
                headings = re.findall(r'^#{1,6}\s+(.+?)\s*#*$', path.read_text(), re.M)
                anchors = {re.sub(r'[^\w\- ]', '', h.lower()).replace(' ', '-') for h in headings}
                assert urllib.parse.unquote(parsed.fragment) in anchors, (source, target)
            count += 1
assert count > 0, 'No added links checked; stage newly added files first.'
print(f'PASS: {count} added local Markdown links/anchors resolve (base {base}).')
