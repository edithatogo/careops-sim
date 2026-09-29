import subprocess
import sys
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TOOL = ROOT / "tools" / "validate_ed_usage_matrix.py"
REGISTRY = ROOT / "model-inputs/ed/schema/parameter-ids.json"
MATRIX = ROOT / "model-inputs/ed/schema/parameter-usage-matrix-proposal.json"


class EDUsageMatrixTests(unittest.TestCase):
    def test_usage_matrix_covers_all_registered_ids_and_marks_proposals(self):
        result = subprocess.run(
            [sys.executable, str(TOOL), "--registry", str(REGISTRY), "--matrix", str(MATRIX)],
            cwd=ROOT, text=True, capture_output=True, check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("101 registered IDs", result.stdout)


if __name__ == "__main__":
    unittest.main()
