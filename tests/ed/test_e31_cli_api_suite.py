"""E31 fresh-user native CLI contract: real tables, finite controls and errors."""
import json
from pathlib import Path
import tempfile
import unittest
from native_cli_support import (NativeCliCase, NOMINAL, CONSTRAINED, MALFORMED,
                                invoke, control, sha, read_dataset, metric_rows, stable_rows)


class E31CliApiSuiteTests(NativeCliCase):
    def test_fresh_user_help_walkthrough(self):
        proc = invoke('--help')
        self.assertEqual(proc.returncode, 0, proc.stderr)
        for text in ['validate INPUT.json', 'run INPUT.json --output-dir FRESH',
                     'compare LEFT_DATASET RIGHT_DATASET', 'export DATASET', 'Parquet/Zstd']:
            self.assertIn(text, proc.stdout)

    def test_missing_or_unknown_command_returns_usage_error(self):
        for args in [(), ('unknown_action',)]:
            with self.subTest(args=args):
                self.assertEqual(invoke(*args).returncode, 2)

    def test_validate_nominal_scenario(self):
        proc = invoke('validate', NOMINAL)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        report = json.loads(proc.stdout)
        self.assertEqual(report['scenario_id'], 'nominal-ed-mvp')
        self.assertEqual(report['patient_count'], 5)
        self.assertEqual(report['schema_version'], 1)

    def test_validate_missing_and_malformed_scenarios(self):
        self.assertEqual(invoke('validate', NOMINAL.parent / 'absent.json').returncode, 3)
        self.assertEqual(invoke('validate', MALFORMED).returncode, 4)

    def test_run_nominal_stdout(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'run'
            manifest, tables = self.run_ok(root)
            self.assertEqual(manifest['scenario_id'], 'nominal-ed-mvp')
            self.assertEqual(manifest['base_seed'], 42)
            self.assertEqual(tables['patients'].num_rows, 5)
            rep = tables['replications'].to_pylist()[0]
            self.assertEqual((rep['completed'], rep['unfinished']), (5, 0))
            self.assertEqual(control(root / 'invocation.json')['raw_input_sha256'], sha(NOMINAL))
            self.assertLess((root / 'manifest.json').stat().st_size, 128 * 1024)

    def test_run_overrides_seed_and_horizon(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'run'
            manifest, tables = self.run_ok(root, NOMINAL, 'run', '--seed', 9999, '--horizon', 7)
            self.assertEqual((manifest['base_seed'], manifest['horizon_ticks']), (9999, 7))
            rep = tables['replications'].to_pylist()[0]
            self.assertEqual((rep['seed'], rep['horizon_ticks']), (9999, 7))
            self.assertLess(rep['completed'], 5)
            self.assertGreater(rep['unfinished'], 0)
            patients = tables['patients'].to_pylist()
            self.assertTrue(any(row['completion_tick_lo'] is None for row in patients))

    def test_run_output_overwrite_protection(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'run'
            self.run_ok(root)
            original = sha(root / 'manifest.json')
            self.assertEqual(invoke('run', NOMINAL, '--output-dir', root).returncode, 6)
            self.assertEqual(invoke('run', NOMINAL, '--output-dir', root, '--overwrite').returncode, 2)
            self.assertEqual(sha(root / 'manifest.json'), original)

    def test_compare_runs_lifecycle_and_diagnostics(self):
        with tempfile.TemporaryDirectory() as directory:
            roots = [Path(directory) / name for name in ['nominal', 'constrained']]
            _, left = self.run_ok(roots[0])
            _, right = self.run_ok(roots[1], CONSTRAINED)
            proc = invoke('compare', *(root / 'data' for root in roots))
            self.assertEqual(proc.returncode, 0, proc.stderr)
            report = json.loads(proc.stdout)
            self.assertEqual(report['uncertainty_method'], 'not_estimated')
            self.assertEqual(report['uncertainty_reason'], 'paired_difference_method_unqualified')
            waits = {m['name']: m for m in report['metrics']}['mean_wait_ticks']
            self.assertEqual(waits['unit'], 'tick')
            self.assertAlmostEqual(waits['left'], 0.4)
            self.assertAlmostEqual(waits['right'], 4.2)
            self.assertAlmostEqual(waits['delta'], 3.8)
            for tables in [left, right]:
                self.assertEqual(metric_rows(tables)['mean_wait_ticks']['denominator_name'], 'included_replications')

    def test_compare_incompatible_scenario_rejected(self):
        import csv
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            changed = json.loads(NOMINAL.read_text()); changed['time_unit'] = 'second'
            scenario = directory / 'seconds.json'; scenario.write_text(json.dumps(changed))
            # The ED model has no seconds conversion; reject that input first.
            invalid = invoke('run', scenario, '--output-dir', directory / 'invalid')
            self.assertEqual(invalid.returncode, 4)
            self.assertFalse((directory / 'invalid').exists())
            self.run_ok(directory / 'left')
            right = directory / 'right'
            exported = invoke('export', directory / 'left/data', '--output-dir', right, '--format', 'csv')
            self.assertEqual(exported.returncode, 0, exported.stderr)
            # Construct an explicitly incompatible transport fixture, not a
            # simulated conversion. CSV carries typed metadata in its manifest.
            manifest = control(right / 'manifest.json'); manifest['time_unit'] = 'second'
            for table in manifest['tables']:
                for part in table['parts']:
                    path = right / part['file']
                    with path.open(newline='') as stream:
                        reader = csv.DictReader(stream); names = reader.fieldnames; rows = list(reader)
                    for row in rows:
                        if 'time_unit' in row: row['time_unit'] = 'second'
                        if row.get('unit') == 'tick': row['unit'] = 'second'
                    with path.open('w', newline='') as stream:
                        writer = csv.DictWriter(stream, fieldnames=names, lineterminator='\n')
                        writer.writeheader(); writer.writerows(rows)
                    part['bytes'] = path.stat().st_size; part['sha256'] = sha(path)
            (right / 'manifest.json').write_text(json.dumps(manifest))
            read_dataset(right)
            # Conversion validates the independently rehashed native fixture
            # before comparison, distinguishing incompatibility from corruption.
            checked = invoke('export', right, '--output-dir', directory / 'checked', '--format', 'ipc')
            self.assertEqual(checked.returncode, 0, checked.stderr)
            proc = invoke('compare', directory / 'left/data', directory / 'checked')
            self.assertEqual(proc.returncode, 5)
            self.assertIn('compatible units', proc.stderr)

    def test_export_formats(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            _, source = self.run_ok(directory / 'run')
            for format in ['parquet', 'ipc', 'csv']:
                destination = directory / format
                proc = invoke('export', directory / 'run/data', '--output-dir', destination, '--format', format)
                self.assertEqual(proc.returncode, 0, proc.stderr)
                manifest, actual = read_dataset(destination)
                self.assertEqual(manifest['format'], format)
                self.assertEqual(stable_rows(source), stable_rows(actual))
                receipt = json.loads(proc.stdout)
                self.assertEqual(receipt['dataset_manifest_sha256'], sha(destination / 'manifest.json'))
                self.assertEqual(invoke('export', directory / 'run/data', '--output-dir', destination).returncode, 6)


if __name__ == '__main__':
    unittest.main()
