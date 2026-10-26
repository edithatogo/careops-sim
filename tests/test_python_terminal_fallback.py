"""Run actual always-finalizers when the Python bootstrap never succeeded."""
import json
from pathlib import Path
import re
import subprocess
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]


class TerminalFallbackTests(unittest.TestCase):
    def test_missing_or_nonexecutable_runtime_retains_every_terminal_receipt(self):
        cases = [
            ('cargo-lock-refresh.yml', 'Record actual terminal outcomes without promotion', '.artifacts/ci-suite/qualification.json'),
            ('schema-lock-refresh.yml', 'Record actual terminal outcomes without promotion', '.artifacts/ci-suite/qualification.json'),
            ('schema-development-canary.yml', 'Record terminal discovery evidence', '.artifacts/ci-preview/discovery.json'),
            ('schema-development-canary.yml', 'Record honest terminal qualification', '.artifacts/schema-development/qualification.json'),
            ('rust-development-canary.yml', 'Record discovery terminal outcome', '.artifacts/rust-canary/discovery/discovery.json'),
            ('rust-development-canary.yml', 'Retain terminal outcome without candidate promotion', '.artifacts/rust-canary/qualification/qualification.json'),
        ]
        for workflow, name, output in cases:
            text = (ROOT / '.github/workflows' / workflow).read_text()
            step = text.split('      - name: ' + name + '\n', 1)[1].split('\n      - ', 1)[0]
            script = textwrap.dedent(step.split('        run: |\n', 1)[1])
            stages = re.findall(r'^          ([A-Z_]+_OUTCOME):', step, re.M)
            for runtime in [None, '/nonexistent/verified-python']:
                with self.subTest(workflow=workflow, step=name, runtime=runtime), tempfile.TemporaryDirectory() as temporary:
                    env = {'GITHUB_WORKSPACE': temporary, **dict.fromkeys(stages, 'skipped')}
                    env[stages[0]] = 'failure'
                    if runtime is not None:
                        env['CAREOPS_PYTHON'] = runtime
                    result = subprocess.run(['/bin/bash', '-euc', script], cwd=temporary, env=env, capture_output=True, text=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    receipt = json.loads((Path(temporary) / output).read_text())
                    self.assertEqual(receipt['state'], 'setup-blocked')
                    self.assertFalse(receipt['runtimeTestsExecuted'])
                    self.assertFalse(receipt['promoted'])
                    self.assertFalse(receipt['nativeWrites'])
                    self.assertEqual(receipt['stages'], {stage: env[stage] for stage in stages})


if __name__ == '__main__':
    unittest.main()
