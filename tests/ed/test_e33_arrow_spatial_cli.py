"""E33 real Arrow/Parquet, qualified metric semantics, no invented spatial capture."""
from pathlib import Path
import tempfile
import unittest
from native_cli_support import (NativeCliCase, NOMINAL, invoke, read_dataset,
                                metric_rows, stable_rows)


class E33ArrowSpatialCliTests(NativeCliCase):
    def test_cli_export_arrow_and_parquet_schemas(self):
        import pyarrow as pa
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            _, source = self.run_ok(directory / 'run')
            for format in ['ipc', 'parquet']:
                proc = invoke('export', directory / 'run/data', '--output-dir', directory / format, '--format', format)
                self.assertEqual(proc.returncode, 0, proc.stderr)
                manifest, tables = read_dataset(directory / format)
                self.assertEqual(stable_rows(source), stable_rows(tables))
                patients = tables['patients']
                for column in ['arrival_tick_hi', 'arrival_tick_lo', 'wait_ticks_hi', 'wait_ticks_lo', 'elapsed_ticks_hi', 'elapsed_ticks_lo']:
                    self.assertEqual(patients.schema.field(column).type, pa.uint64())
                declared = {c['name']: c for c in manifest['tables'][0]['columns']}
                self.assertTrue(declared['wait_ticks_lo']['nullable'])
                self.assertFalse(declared['arrival_tick_lo']['nullable'])
                self.assertEqual(manifest['tables'][0]['schema'], 'careops.ed.patients.v2')

    def test_cli_batch_arrow_bundle_export(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'batch'
            manifest, tables = self.run_ok(root, NOMINAL, 'batch', '--replications', 4, '--workers', 2, '--seed', 12345)
            self.assertEqual(len(manifest['replication_seeds']), 4)
            self.assertEqual(len(set(manifest['replication_seeds'])), 4)
            self.assertEqual(tables['patients'].num_rows, 20)
            metrics = metric_rows(tables)
            wait = metrics['mean_wait_ticks']
            self.assertEqual(wait['denominator_name'], 'included_replications')
            self.assertEqual(wait['uncertainty_replications'], 4)
            self.assertEqual(wait['ddof'], 1)
            self.assertAlmostEqual(wait['value'], 0.4)
            for field in ['variance', 'standard_error']:
                self.assertEqual(wait[field], 0.0)
            self.assertAlmostEqual(wait['ci_95_lower'], 0.4)
            self.assertAlmostEqual(wait['ci_95_upper'], 0.4)
            self.assertEqual(metrics['completion_rate']['denominator_name'], 'total_arrivals')
            self.assertEqual(metrics['throughput_per_horizon_tick']['denominator_name'], 'per_replication_horizon_ticks')
            self.assertEqual(metrics['completion_rate']['status'], 'ratio_method_unqualified')
            self.assertIsNone(metrics['completion_rate']['ci_95_lower'])
            self.assertIn('censored_presentation_rate', metrics)
            # Required publication contains only genuinely captured/domain tables.
            self.assertEqual(set(tables), {'patients', 'replications', 'metrics'})
            self.assertFalse((root / 'spatial_summary.csv').exists())
            self.assertNotIn('spatial', {t['name'] for t in manifest['tables']})

    def test_cli_batch_spatial_csv_export(self):
        # The old header-only manufactured export is retired. The native CLI
        # rejects that option before IO, rather than claiming observed movement.
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / 'unused'
            proc = invoke('batch', NOMINAL, '--output-dir', root, '--export-spatial', Path(directory) / 'spatial.csv')
            self.assertEqual(proc.returncode, 2)
            self.assertFalse(root.exists())
            self.assertFalse((Path(directory) / 'spatial.csv').exists())

    def test_cli_batch_deterministic_bundles_across_workers(self):
        with tempfile.TemporaryDirectory() as directory:
            results = [self.run_ok(Path(directory) / str(workers), NOMINAL, 'batch',
                                   '--replications', 4, '--workers', workers, '--seed', 9999)
                       for workers in [1, 2, 4]]
            for manifest, tables in results[1:]:
                self.assertEqual(manifest['replication_seeds'], results[0][0]['replication_seeds'])
                self.assertEqual(stable_rows(tables), stable_rows(results[0][1]))

    def test_cli_error_codes_for_missing_arguments(self):
        for option in ['--format', '--output-dir', '--replications', '--workers', '--checkpoint', '--resume']:
            with self.subTest(option=option):
                self.assertEqual(invoke('batch', NOMINAL, option).returncode, 2)
        for retired in ['--export-arrow', '--export-spatial', '--spatial-rev']:
            with self.subTest(retired=retired):
                self.assertEqual(invoke('batch', NOMINAL, retired).returncode, 2)


if __name__ == '__main__':
    unittest.main()
