import json
import tempfile
import unittest
from pathlib import Path

from check_docs import anchors, check


class DocumentationChecks(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.config = {'include': ['**/*.md'], 'required_paths': ['owner/architecture.md']}
        self.write('owner/architecture.md', '# Contract\n\n## Repeated\n## Repeated\n')

    def write(self, path, text):
        p = self.root / path
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    def errors(self, changed=None):
        return check(self.root, self.config, changed)[0]

    def test_valid_links_reference_definition_unicode_duplicate_anchors_and_fences(self):
        self.write('README.md', '[valid](owner/architecture.md#repeated-1)\n[reference][r]\n[r]: owner/architecture.md#contract\n```md\n[ignored](absent.md)\n```\n')
        self.assertEqual(self.errors(), [])
        self.assertIn('café-42', anchors('# Café 42'))

    def test_broken_path_anchor_and_escape_report_lines(self):
        self.write('README.md', '[bad](missing.md)\n[anchor](owner/architecture.md#absent)\n[out](../secret)\n')
        errors = self.errors()
        self.assertEqual(len(errors), 3)
        self.assertTrue(any('README.md:2: missing Markdown anchor' in e for e in errors))

    def test_exact_baseline_does_not_hide_new_failure(self):
        self.write('README.md', '[bad](missing.md)\n')
        self.config['baseline'] = list(check(self.root, self.config)[1])
        self.assertEqual(self.errors(), [])
        self.write('README.md', '[bad](missing.md)\n[other](other.md)\n')
        self.assertEqual(len(self.errors()), 1)

    def test_stale_rule_is_scoped_and_historical_exclusion_is_explicit(self):
        self.config['stale_rules'] = [{'paths': ['owner/*.md'], 'pattern': 'bad contract', 'reason': 'superseded'}]
        self.write('owner/architecture.md', '# Contract\nbad contract\n')
        self.write('history/old.md', 'bad contract\n[bad](missing.md)')
        self.config['exclude'] = ['history/*']
        self.assertEqual(len(self.errors()), 1)

    def test_required_guides_json_example_and_changed_owner_gate(self):
        self.write('owner/architecture.md', '# Contract')
        self.write('client/domain/Cargo.toml', '[package]')
        self.write('client/domain/README.ai.md', '# AI')
        self.write('client/domain/architecture.md', '# Architecture')
        self.write('client/domain/invariants.md', '# Invariants')
        changed = ['client/domain/src/lib.rs']
        self.assertEqual(len(self.errors(changed)), 1)
        self.assertEqual(self.errors(changed + ['client/domain/architecture.md']), [])
        self.config['no_impact'] = [{'paths': changed, 'reason': 'Regression-only; public behavior unchanged'}]
        self.assertEqual(self.errors(changed), [])
        self.config['json_examples'] = [{'path': 'example.json', 'fields': {'schema_version': 1}}]
        self.write('example.json', json.dumps({'schema_version': 2}))
        self.assertEqual(len(self.errors(changed)), 1)


if __name__ == '__main__':
    unittest.main()
