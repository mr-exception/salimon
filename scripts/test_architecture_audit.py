import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import architecture_audit as audit


class ArchitectureAuditTests(unittest.TestCase):
    def test_literals_and_nested_comments_do_not_invent_branches_or_imports(self):
        source = 'if x { "match salimon_fake::x"; r##"while"##; /* for /* if */ loop */ } // match\n'
        code = audit.code_only(source)
        self.assertEqual(code.count('\n'), source.count('\n'))
        self.assertEqual(code.split(), ['if', 'x', '{', ';', ';', '}'])

    def test_cycles_include_components_and_self_edges(self):
        self.assertEqual(audit.cycles({'a': {'b'}, 'b': {'a'}, 'c': {'c'}, 'd': set()}),
                         [['a', 'b'], ['c']])

    def test_first_party_history_overlap_is_deterministic(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            def git(*args):
                subprocess.run(['git', '-C', tmp, *args], check=True, capture_output=True)
            git('init')
            git('config', 'user.email', 'fixture@example.invalid')
            git('config', 'user.name', 'Fixture')
            for name, source in [('a', 'use salimon_b::B; fn x() { if true {} }'),
                                 ('b', 'pub struct B;')]:
                owner = root / 'client' / name
                (owner / 'src').mkdir(parents=True)
                (owner / 'Cargo.toml').write_text(f'[package]\nname = "salimon-{name}"\n')
                (owner / 'src/lib.rs').write_text(source)
            (root / 'vendor').mkdir()
            (root / 'vendor/noise.rs').write_text('if while match')
            (root / 'client/a/src/spatial_contracts.rs').write_text('generated')
            git('add', '.')
            git('commit', '-m', 'Fixture (#1)')
            first = audit.audit(root)
            self.assertEqual(json.dumps(first), json.dumps(audit.audit(root)))
            self.assertEqual(len(first['files']), 2)
            self.assertEqual(first['crate_graph']['salimon_a'], ['salimon_b'])
            self.assertEqual(first['cochange_pairs'], [
                {'paths': ['client/a/src/lib.rs', 'client/b/src/lib.rs'], 'count': 1}])


if __name__ == '__main__':
    unittest.main()
