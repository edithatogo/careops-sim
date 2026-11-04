"""E32 native batch, deterministic workers and actual capture-boundary recovery."""
import json
from pathlib import Path
import tempfile
import unittest
from native_cli_support import (NativeCliCase, NOMINAL, invoke, control, sha,
                                stable_rows, prepare_capture_only)


class E32BatchWorkersTests(NativeCliCase):
    def test_batch_help_text(self):
        proc = invoke('--help')
        self.assertEqual(proc.returncode, 0)
        for option in ['batch INPUT.json', '--replications', '--workers', '--checkpoint', '--resume']:
            self.assertIn(option, proc.stdout)

    def test_batch_deterministic_equivalence_across_workers(self):
        with tempfile.TemporaryDirectory() as directory:
            previous = None
            for workers in [1, 2, 4]:
                manifest, tables = self.run_ok(Path(directory) / str(workers), NOMINAL, 'batch',
                                               '--replications', 4, '--workers', workers, '--seed', 42)
                self.assertEqual(len(set(manifest['replication_seeds'])), 4)
                self.assertEqual(manifest['replication_seeds'][0], 42)
                reps = tables['replications'].to_pylist()
                self.assertEqual([row['replication_index'] for row in reps], [0, 1, 2, 3])
                self.assertEqual([row['seed'] for row in reps], manifest['replication_seeds'])
                self.assertTrue(all(row['completed'] == 5 and row['unfinished'] == 0 for row in reps))
                rows = stable_rows(tables)
                if previous is not None:
                    self.assertEqual(previous, rows)
                previous = rows

    def test_batch_checkpoint_and_output_dir(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'batch'
            self.run_ok(root, NOMINAL, 'batch', '--replications', 4, '--workers', 2, '--checkpoint', 2)
            checkpoint = control(root / 'capture/checkpoint.json')
            self.assertEqual(checkpoint['config']['replications'], 4)
            self.assertEqual(len(checkpoint['completed_replications']), 4)
            self.assertTrue((root / 'batch_run.json').is_file())
            self.assertFalse((root / 'batch_run.partial').exists())
            self.assertEqual(control(root / 'invocation.json')['checkpoint_interval'], 2)

    def test_batch_resumption_from_checkpoint(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory); root = directory / 'batch'
            _, original = self.run_ok(root, NOMINAL, 'batch', '--replications', 4, '--workers', 1, '--seed', 777)
            capture_hashes, invocation_hash = prepare_capture_only(root, directory / 'published-history')
            checkpoint = root / 'capture/checkpoint.json'
            actual_checkpoint = checkpoint.read_bytes()
            checkpoint.write_text(json.dumps({'schema_version': 1, 'status': 'in_progress', 'pending_indices': [2, 3]}))
            try:
                rejected = invoke('batch', NOMINAL, '--output-dir', root, '--resume', checkpoint)
                self.assertEqual(rejected.returncode, 4, rejected.stderr)
                self.assertFalse((root / 'manifest.json').exists())
                self.assertEqual(capture_hashes, {p.name: sha(p) for p in (root / 'capture').glob('rep-*-seq-*.part')})
            finally:
                checkpoint.write_bytes(actual_checkpoint)
            _, resumed = self.run_ok(root, NOMINAL, 'batch', '--resume', root / 'capture/checkpoint.json', '--workers', 2)
            self.assertEqual(stable_rows(original), stable_rows(resumed))
            self.assertEqual(capture_hashes, {p.name: sha(p) for p in (root / 'capture').glob('rep-*-seq-*.part')})
            self.assertEqual(invocation_hash, sha(root / 'invocation.json'))
            complete_hash = sha(root / 'manifest.json')
            self.assertEqual(invoke('batch', NOMINAL, '--output-dir', root, '--resume', root / 'capture/checkpoint.json').returncode, 6)
            self.assertEqual(complete_hash, sha(root / 'manifest.json'))
            fresh = directory / 'fresh'
            self.assertEqual(invoke('batch', NOMINAL, '--output-dir', fresh, '--resume', root / 'capture/checkpoint.json').returncode, 7)
            self.assertFalse(fresh.exists())

    def test_batch_missing_scenario_returns_usage_error(self):
        self.assertEqual(invoke('batch').returncode, 2)

    def test_batch_nonexistent_scenario_returns_file_not_found(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'unused'
            self.assertEqual(invoke('batch', Path(directory) / 'absent.json', '--output-dir', root).returncode, 3)
            self.assertFalse(root.exists())


if __name__ == '__main__':
    unittest.main()
