"""E43 native fresh-user walkthrough; engineering verification, not clinical validation."""
import json
from pathlib import Path
import tempfile
import unittest
from native_cli_support import (NativeCliCase, NOMINAL, CONSTRAINED, binary_path, invoke,
                                control, sha, read_dataset, metric_rows, stable_rows,
                                prepare_capture_only)


class E43DeliveryContractWalkthroughTests(NativeCliCase):
    def test_01_installation_and_help_interface(self):
        self.assertTrue(binary_path().is_file())
        for flag in ['--help', '-h']:
            proc = invoke(flag)
            self.assertEqual(proc.returncode, 0, proc.stderr)
            for text in ['validate INPUT.json', 'run INPUT.json', 'batch INPUT.json',
                         'compare LEFT_DATASET RIGHT_DATASET', 'export DATASET', '--output-dir',
                         'never overwritten', 'Parquet/Zstd', 'Software verification', 'clinical domain validation']:
                self.assertIn(text, proc.stdout)
        for args in [(), ('unrecognized',), ('run',), ('export',), ('compare',)]:
            self.assertEqual(invoke(*args).returncode, 2)

    def test_02_nominal_scenario_inspection_and_validation(self):
        scenario = control(NOMINAL)
        self.assertEqual(scenario['schema_version'], 1)
        self.assertEqual(scenario['scenario_id'], 'nominal-ed-mvp')
        self.assertEqual(scenario['capacity_location']['capacity']['physical_count'], 5)
        self.assertEqual(len(scenario['patients']), 5)
        proc = invoke('validate', NOMINAL)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        report = json.loads(proc.stdout)
        self.assertEqual((report['scenario_id'], report['schema_version'], report['patient_count']),
                         ('nominal-ed-mvp', 1, 5))

    def test_03_scenario_editing_surge_condition(self):
        scenario = control(NOMINAL)
        scenario['scenario_id'] = 'surge-ed-mvp'
        for i in range(1, 6):
            scenario['patients'].append(dict(patient_id=f'surge-patient-{i:03d}', arrival_tick=2*i,
                                            work_ticks=4, resource_id='res.acute', task_class_id='task.acute_treatment'))
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'surge.json'; source.write_text(json.dumps(scenario))
            proc = invoke('validate', source)
            self.assertEqual(proc.returncode, 0, proc.stderr)
            self.assertEqual(json.loads(proc.stdout)['patient_count'], 10)
            manifest, tables = self.run_ok(Path(directory) / 'surge', source)
            self.assertEqual(manifest['scenario_id'], 'surge-ed-mvp')
            self.assertEqual(tables['patients'].num_rows, 10)
            replication = tables['replications'].to_pylist()[0]
            self.assertEqual(replication['arrivals'], 10)
            self.assertEqual(replication['arrivals'], replication['completed'] + replication['unfinished'])

    def test_04_execution_atomic_writes_and_overwrite_protection(self):
        with tempfile.TemporaryDirectory() as directory:
            for name, scenario, wait in [('nominal', NOMINAL, 0.4), ('constrained', CONSTRAINED, 4.2)]:
                root = Path(directory) / name
                manifest, tables = self.run_ok(root, scenario)
                rep = tables['replications'].to_pylist()[0]
                self.assertEqual((rep['arrivals'], rep['completed']), (5, 5))
                self.assertAlmostEqual(rep['mean_wait_ticks'], wait)
                self.assertEqual(manifest['raw_input_sha256'], None)  # raw identity belongs to CLI invocation.
                self.assertEqual(control(root / 'invocation.json')['raw_input_sha256'], sha(scenario))
                self.assertEqual(control(root / 'invocation.json')['parsed_input_sha256'], manifest['parsed_input_sha256'])
                complete_hash = sha(root / 'manifest.json')
                self.assertFalse((root / 'batch_run.partial').exists())
                self.assertEqual(invoke('run', scenario, '--output-dir', root).returncode, 6)
                self.assertEqual(sha(root / 'manifest.json'), complete_hash)

    def test_05_comparative_analysis_signed_deltas_and_neutral_reporting(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            self.run_ok(directory / 'nominal')
            self.run_ok(directory / 'constrained', CONSTRAINED)
            reports = []
            for left, right in [('nominal', 'constrained'), ('constrained', 'nominal')]:
                proc = invoke('compare', directory / left / 'data', directory / right / 'data')
                self.assertEqual(proc.returncode, 0, proc.stderr)
                report = json.loads(proc.stdout); reports.append(report)
                metrics = {row['name']: row for row in report['metrics']}
                self.assertEqual(metrics['mean_wait_ticks']['unit'], 'tick')
                self.assertEqual(report['uncertainty_method'], 'not_estimated')
                self.assertEqual(report['uncertainty_reason'], 'paired_difference_method_unqualified')
                self.assertEqual(metrics['completion_rate']['delta'], 0)
                self.assertNotIn('clinical_validation', report)
            positive = {row['name']: row for row in reports[0]['metrics']}['mean_wait_ticks']
            negative = {row['name']: row for row in reports[1]['metrics']}['mean_wait_ticks']
            self.assertAlmostEqual(positive['delta'], 3.8)
            self.assertAlmostEqual(negative['delta'], -3.8)
            self.assertAlmostEqual(positive['left'], 0.4)
            self.assertAlmostEqual(positive['right'], 4.2)
            zero_input = control(NOMINAL)
            zero_input['patients'] = zero_input['patients'][:1]
            zero_source = directory / 'zero-wait.json'; zero_source.write_text(json.dumps(zero_input))
            self.run_ok(directory / 'zero', zero_source)
            proc = invoke('compare', directory / 'zero/data', directory / 'nominal/data')
            self.assertEqual(proc.returncode, 0, proc.stderr)
            zero_wait = {row['name']: row for row in json.loads(proc.stdout)['metrics']}['mean_wait_ticks']
            self.assertEqual(zero_wait['left'], 0.0)
            self.assertAlmostEqual(zero_wait['delta'], 0.4)
            self.assertNotIn('percent_change', zero_wait)  # Native v1 reports finite absolute differences only.

    def test_06_multiformat_export_csv_arrow_parquet(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            _, original = self.run_ok(directory / 'run')
            for format in ['csv', 'ipc', 'parquet']:
                destination = directory / format
                proc = invoke('export', directory / 'run/data', '--output-dir', destination, '--format', format)
                self.assertEqual(proc.returncode, 0, proc.stderr)
                manifest, tables = read_dataset(destination)
                self.assertEqual(manifest['format'], format)
                self.assertEqual(tables['patients'].num_rows, 5)
                self.assertEqual(stable_rows(original), stable_rows(tables))
                conversion = control(destination / 'conversion.json')
                self.assertTrue(conversion)
                self.assertEqual(json.loads(proc.stdout)['conversion_receipt_sha256'], sha(destination / 'conversion.json'))

    def test_07_intentional_error_troubleshooting(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory)
            for kind in ['zero_capacity', 'zero_horizon', 'unknown_zone', 'malformed']:
                scenario = control(NOMINAL)
                if kind == 'zero_capacity':
                    scenario['capacity_location']['capacity'].update(physical_count=0, open_count=0, staffed_count=0)
                elif kind == 'zero_horizon':
                    scenario['horizon_ticks'] = 0
                elif kind == 'unknown_zone':
                    scenario['capacity_location']['locations'][0]['zone_id'] = 'zone.unregistered'
                source = directory / (kind + '.json')
                source.write_text('{ broken json' if kind == 'malformed' else json.dumps(scenario))
                proc = invoke('validate', source)
                self.assertEqual(proc.returncode, 4, proc.stderr)
                self.assertTrue(proc.stderr.strip())
            self.assertEqual(invoke('validate', directory / 'absent.json').returncode, 3)
            for mode in ['metal', 'pdes', 'grpc', 'mpi', 'distributed']:
                root = directory / mode
                proc = invoke('run', directory / 'absent.json', '--output-dir', root, '--backend', mode)
                self.assertEqual(proc.returncode, 8, proc.stderr)
                self.assertFalse(root.exists())
            self.run_ok(directory / 'good')
            manifest = directory / 'good/data/manifest.json'
            original = manifest.read_bytes(); payload = json.loads(original); payload['status'] = 'incomplete'
            manifest.write_text(json.dumps(payload))
            try:
                proc = invoke('compare', manifest.parent, manifest.parent)
                self.assertEqual(proc.returncode, 4, proc.stderr)
            finally:
                manifest.write_bytes(original)

    def test_08_rollback_and_checkpoint_recovery_rehearsal(self):
        with tempfile.TemporaryDirectory() as directory:
            directory = Path(directory); root = directory / 'batch'
            _, original = self.run_ok(root, NOMINAL, 'batch', '--replications', 4, '--workers', 2, '--checkpoint', 2)
            hashes, invocation_hash = prepare_capture_only(root, directory / 'published-history')
            checkpoint = root / 'capture/checkpoint.json'
            checkpoint_bytes = checkpoint.read_bytes()
            checkpoint.write_text('{ broken checkpoint JSON')
            try:
                failed = invoke('batch', NOMINAL, '--output-dir', root, '--resume', checkpoint)
                self.assertEqual(failed.returncode, 4, failed.stderr)
                self.assertFalse((root / 'manifest.json').exists())
                self.assertEqual(hashes, {p.name: sha(p) for p in (root / 'capture').glob('rep-*-seq-*.part')})
            finally:
                checkpoint.write_bytes(checkpoint_bytes)
            _, recovered = self.run_ok(root, NOMINAL, 'batch', '--resume', checkpoint, '--workers', 4)
            self.assertEqual(stable_rows(original), stable_rows(recovered))
            self.assertEqual(sha(root / 'invocation.json'), invocation_hash)
            self.assertEqual(hashes, {p.name: sha(p) for p in (root / 'capture').glob('rep-*-seq-*.part')})
            # Rollback uses the preserved original input in a fresh destination.
            _, rollback = self.run_ok(directory / 'rollback', NOMINAL, 'batch', '--replications', 4, '--workers', 1)
            self.assertEqual(stable_rows(original), stable_rows(rollback))


if __name__ == '__main__':
    unittest.main()
