"""Shared release actions must be tested and updated together."""
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parent.parent

class SharedPolicyTests(unittest.TestCase):
    def test_shared_action_group_preserves_security_updates(self):
        actions = (ROOT / '.github/dependabot.yml').read_text().split('package-ecosystem: github-actions', 1)[1]
        group = actions.split('      shared-policy:', 1)[1].split('      security:', 1)[0]
        self.assertIn('applies-to: version-updates', group)
        self.assertIn('patterns: ["legra-ai/github-actions*"]', group)
        self.assertNotIn('update-types:', group)
        self.assertIn('applies-to: security-updates', actions)
        self.assertIn('interval: daily', actions)
        self.assertIn('prefix: ci', actions)
        self.assertIn('include: scope', actions)

    def test_shared_actions_use_one_immutable_revision(self):
        revisions = set()
        for path in (ROOT / '.github/workflows').glob('*.yml'):
            revisions.update(re.findall(r'uses: legra-ai/github-actions[^@\s]*@([0-9a-f]{40})', path.read_text()))
        self.assertEqual(len(revisions), 1)

if __name__ == '__main__':
    unittest.main()
