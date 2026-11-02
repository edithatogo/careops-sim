"""Actual independent-reader regressions; run after installing pinned PyArrow."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from read_batch import read_batch

BINARY = Path(sys.argv.pop(1)).resolve()
FIXTURE = Path(__file__).with_name('fixtures') / 'synthetic.json'


class IndependentReaderTests(unittest.TestCase):
    def test_float_spelling_and_zero_nonbreached_metric_provenance(self):
        for rate, target in [(1e-9, 0), (1e20, 30), (0.25, 3)]:
            with self.subTest(rate=rate, target=target), tempfile.TemporaryDirectory() as temp:
                root = Path(temp)
                root.chmod(0o700)
                output = root / 'output'
                output.mkdir(mode=0o700)
                value = json.loads(FIXTURE.read_text())
                value['replications'] = 1
                value['scenario']['progression_rate'] = rate
                value['scenario']['target_birth_seconds'] = target
                config = root / 'input.json'
                config.write_text(json.dumps(value))
                subprocess.run([str(BINARY), str(config), str(output), '1'], check=True, capture_output=True)
                report = read_batch(output, value)
                self.assertEqual(report['replications'][0]['policy_metrics_checked'], 9)
                batch = output / value['batch_id']
                receipt = json.loads((batch / 'batch_manifest.json').read_text())[0]
                manifest = batch / receipt['directory'] / 'birthing_manifest.json'
                manifest.unlink()
                with self.assertRaisesRegex(ValueError, 'unsupported or oversized declaration'):
                    read_batch(output, value)


if __name__ == '__main__':
    unittest.main()
