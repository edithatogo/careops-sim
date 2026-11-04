"""MVP native CLI scenarios: typed files, provenance, capacity and repeatability."""
from pathlib import Path
import tempfile
import unittest

from native_cli_support import (NativeCliCase, NOMINAL, CONSTRAINED, MALFORMED,
                                invoke, control, sha, native_tick, stable_rows)


class E23MvpScenarioTests(NativeCliCase):
    """Execution and verification of the headless MVP delivery recipe."""

    def test_nominal_mvp_scenario_execution(self):
        with tempfile.TemporaryDirectory(prefix='careops-e23-') as directory:
            output = Path(directory) / 'run'
            manifest, tables = self.run_ok(output)
            invocation = control(output / 'invocation.json')
            self.assertEqual(invocation['raw_input_sha256'], sha(NOMINAL))
            self.assertEqual(invocation['parsed_input_sha256'], manifest['parsed_input_sha256'])
            self.assertEqual(manifest['schema'], 'careops.ed.native-output.v1')
            self.assertEqual(manifest['scenario_id'], 'nominal-ed-mvp')
            self.assertEqual(manifest['base_seed'], 42)
            self.assertEqual(manifest['time_unit'], 'tick')
            self.assertEqual(manifest['horizon_ticks'], 20)
            replication, = tables['replications'].to_pylist()
            for name, expected in [('arrivals', 5), ('started', 5),
                                   ('completed', 5), ('unfinished', 0)]:
                self.assertEqual(replication[name], expected)
            self.assertEqual(tables['patients'].num_rows, 5)
            self.assertEqual(replication['completed'] + replication['unfinished'],
                             replication['arrivals'])

    def test_constrained_mvp_scenario_and_capacity_comparison(self):
        _, nominal = self.run_scenario(NOMINAL)
        _, constrained = self.run_scenario(CONSTRAINED)
        self.assertEqual(nominal['replications']['arrivals'].to_pylist(),
                         constrained['replications']['arrivals'].to_pylist())
        nom_waits = [native_tick(p, 'wait_ticks') for p in nominal['patients'].to_pylist()
                     if native_tick(p, 'wait_ticks') is not None]
        con_waits = [native_tick(p, 'wait_ticks') for p in constrained['patients'].to_pylist()
                     if native_tick(p, 'wait_ticks') is not None]
        nom_mean_wait = sum(nom_waits) / len(nom_waits)
        con_mean_wait = sum(con_waits) / len(con_waits)
        self.assertAlmostEqual(nom_mean_wait, 0.4)
        self.assertAlmostEqual(con_mean_wait, 4.2)
        self.assertGreater(con_mean_wait, nom_mean_wait, "Constrained capacity must increase wait times")

    def test_malformed_input_rejection(self):
        with tempfile.TemporaryDirectory(prefix='careops-e23-invalid-') as directory:
            output = Path(directory) / 'run'
            proc = invoke('run', MALFORMED, '--output-dir', output)
            self.assertNotEqual(proc.returncode, 0, "Malformed scenario must exit with failure code")
            self.assertIn('schema_version must be 1', proc.stderr)
            self.assertFalse((output / 'manifest.json').exists())

    def test_fixed_seed_repeatability(self):
        manifest1, tables1 = self.run_scenario(NOMINAL)
        manifest2, tables2 = self.run_scenario(NOMINAL)
        self.assertEqual(manifest1, manifest2)
        self.assertEqual(stable_rows(tables1), stable_rows(tables2),
                         "Fixed seed must produce identical typed outputs")


if __name__ == '__main__':
    unittest.main()
