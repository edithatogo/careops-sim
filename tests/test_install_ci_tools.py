import hashlib
import importlib.util
import io
from pathlib import Path
import unittest
from unittest.mock import Mock, patch
import urllib.error

spec = importlib.util.spec_from_file_location("installer", Path(__file__).resolve().parents[1] / "tools/install_ci_tools.py")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

class InstallCiToolsTests(unittest.TestCase):
    def setUp(self):
        self.spec = {"url": "https://example.org/pinned.tgz", "sha256": hashlib.sha256(b"valid").hexdigest()}
    def test_transient_http_retries_then_checks_exact_bytes(self):
        opener = Mock(side_effect=[urllib.error.HTTPError(self.spec["url"], 503, "unavailable", {}, None), io.BytesIO(b"valid")])
        sleep = Mock()
        self.assertEqual(module.download(self.spec, "tool", opener, sleep), b"valid")
        self.assertEqual(opener.call_count, 2); sleep.assert_called_once_with(0.5)
    def test_nontransient_and_exhausted_transient_fail(self):
        for status, count in [(404, 1), (502, 5)]:
            opener = Mock(side_effect=urllib.error.HTTPError(self.spec["url"], status, "failed", {}, None)); sleep = Mock()
            with self.assertRaises(urllib.error.HTTPError): module.download(self.spec, "tool", opener, sleep)
            self.assertEqual(opener.call_count, count); self.assertEqual(sleep.call_count, count - 1)
    def test_checksum_and_size_mismatch_never_retry(self):
        for payload, limit in [(b"wrong", 80 * 1024 * 1024), (b"valid", 4)]:
            opener = Mock(return_value=io.BytesIO(payload)); sleep = Mock()
            with patch.object(module, "MAX_ARCHIVE_BYTES", limit):
                with self.assertRaises(SystemExit): module.download(self.spec, "tool", opener, sleep)
            self.assertEqual(opener.call_count, 1); sleep.assert_not_called()
    def test_timeout_retries_are_bounded(self):
        opener = Mock(side_effect=TimeoutError()); sleep = Mock()
        with self.assertRaises(TimeoutError): module.download(self.spec, "tool", opener, sleep)
        self.assertEqual(opener.call_count, 5); self.assertEqual([c.args[0] for c in sleep.call_args_list], [0.5, 1, 2, 4])

if __name__ == "__main__": unittest.main()
