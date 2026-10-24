from pathlib import Path
import subprocess
import tempfile
import unittest

from architecture_review import review


class ArchitectureReviewTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git('init')
        self.git('config', 'user.email', 'fixture@example.invalid')
        self.git('config', 'user.name', 'Fixture')
        for name in ('renderer', 'runtime', 'world'):
            path = self.root / 'client' / name
            (path / 'src').mkdir(parents=True)
            (path / 'Cargo.toml').write_text(f'[package]\nname = "salimon-{name}"\n')
            (path / 'src/lib.rs').write_text('pub struct Value;\n')
        self.git('add', '.')
        self.git('commit', '-m', 'Fixture (#1)')
        self.base = self.git('rev-parse', 'HEAD')
        self.policy = {'allowed_crate_edges': {'salimon_runtime': ['salimon_world', 'salimon_renderer']},
                       'thresholds': {'lines': 1000, 'branch_tokens': 200}, 'exceptions': []}

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.root), *args], stderr=subprocess.DEVNULL, text=True).strip()

    def write(self, owner, text):
        (self.root / f'client/{owner}/src/lib.rs').write_text(text)

    def run_review(self, **kwargs):
        return review(self.root, self.base, self.policy, **kwargs)

    def test_forbidden_direction_new_cycle_and_path_diagnostics(self):
        self.write('renderer', 'use salimon_runtime::Value;\npub struct Value;')
        self.write('runtime', 'use salimon_renderer::Value;\npub struct Value;')
        report = self.run_review()
        self.assertEqual(report['new_cycles'], [['salimon_renderer', 'salimon_runtime']])
        violations = [f for f in report['findings'] if f['rule'] == 'direction']
        self.assertEqual([f['path'] for f in violations], ['client/renderer/src/lib.rs'])
        self.assertTrue(report['warning_only'])

    def test_permitted_new_reference_still_requests_contract_review(self):
        self.write('runtime', 'use salimon_world::Value;\npub fn value() {}')
        findings = self.run_review()['findings']
        self.assertNotIn('direction', [f['rule'] for f in findings])
        self.assertIn('new-domain-reference', [f['rule'] for f in findings])
        self.assertIn('new-import', [f['rule'] for f in findings])

    def test_existing_cycle_does_not_become_new_and_comments_do_not_invent_edges(self):
        self.write('renderer', 'use salimon_runtime::Value; pub struct Value;')
        self.write('runtime', 'use salimon_renderer::Value; pub struct Value;')
        self.git('add', '.')
        self.git('commit', '-m', 'Legacy cycle')
        self.base = self.git('rev-parse', 'HEAD')
        self.write('world', '// use salimon_renderer::Value;\npub struct Value;')
        report = self.run_review()
        self.assertEqual(report['new_cycles'], [])
        self.assertFalse(any(f['path'].startswith('client/world') for f in report['findings']))

    def test_overlap_includes_drafts_skips_self_and_records_unavailable(self):
        self.write('world', 'pub struct Changed;')
        inventories = [{'number': n, 'head_sha': str(n) * 40, 'draft': True,
                        'paths': ['client/world/src/lib.rs']} for n in (2, 3)]
        report = self.run_review(active_prs=inventories, current_pr=2)
        self.assertEqual([p['number'] for p in report['overlaps']], [3])
        self.assertTrue(report['overlaps'][0]['draft'])
        self.assertEqual(self.run_review()['active_prs_status'], 'unavailable')
        self.assertEqual(self.run_review(active_prs=[])['active_prs_status'], 'available')

    def test_exact_documented_exceptions_and_broad_file_threshold(self):
        self.write('renderer', 'use salimon_world::Value;\n' + '\n' * 1000)
        first = self.run_review()
        direction = next(f for f in first['findings'] if f['rule'] == 'direction')
        self.policy['exceptions'] = [dict(direction, owner='renderer', reason='Reviewed fixture adapter; issue #1')]
        second = self.run_review()
        self.assertEqual(second['suppressed'], [direction])
        self.assertIn('broad-file', [f['rule'] for f in second['findings']])
        self.policy['exceptions'][0]['reason'] = ''
        self.assertEqual(self.run_review()['suppressed'], [])

    def test_unavailable_base_fails_instead_of_silently_passing(self):
        self.base = 'missing-ref'
        with self.assertRaises(subprocess.CalledProcessError):
            self.run_review()


if __name__ == '__main__':
    unittest.main()
