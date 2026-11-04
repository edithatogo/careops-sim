"""Negative controls for native benchmark gates; no alternative runtime oracle."""
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import ed_native_benchmarks as native

class NativeBenchmarkTests(unittest.TestCase):
    def test_controls_reject_duplicates_nonfinite_symlinks_and_oversize(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'control.json'
            for value in ['{"a":1,"a":2}','{"a":NaN}']:
                path.write_text(value)
                with self.assertRaises(native.BenchmarkError): native.load(path)
            path.write_text('{"a":1}')
            self.assertEqual(native.load(path),{'a':1})
            with self.assertRaises(native.BenchmarkError):native.load(path,2)
            link=Path(directory)/'linked.json';link.symlink_to(path)
            with self.assertRaises(native.BenchmarkError):native.load(link)

    def test_integer_and_limb_validation_never_coerces(self):
        for value in [True,False,0.0,-1,None,'1']:
            with self.subTest(value=value),self.assertRaises(native.BenchmarkError):native._int(value)
        for row in [{'t_hi':None,'t_lo':1},{'t_hi':-1,'t_lo':0},{'t_hi':0,'t_lo':2**64},{'t_hi':0.0,'t_lo':0}]:
            with self.assertRaises(native.BenchmarkError):native._tick(row,'t')
        self.assertEqual(native._tick({'t_hi':1,'t_lo':0},'t'),2**64)
        self.assertIsNone(native._tick({'t_hi':None,'t_lo':None},'t'))

    def test_wall_rss_and_stdout_gates_are_enforced_before_file_decoder(self):
        template=json.loads(Path('crates/careops-ed/examples/one_patient.json').read_text())
        base={'timed_out':False,'exit_code':0,'stdout_overflow':False,'elapsed_seconds':0.01,'peak_rss_bytes':1,'stdout':b'{}\n','stderr':b''}
        for change, message in [({'elapsed_seconds':native.MAX_WALL_SECONDS+0.001},'wall-time'),({'peak_rss_bytes':native.MAX_RSS_BYTES+1},'peak-RSS'),({'stdout':b''},'stdout'),({'stdout':b'junk'},'stdout')]:
            with tempfile.TemporaryDirectory() as directory:
                result=dict(base,**change)
                with mock.patch.object(native,'run_bounded',return_value=result),mock.patch.object(native,'bounded_bytes',return_value=b'{}'),mock.patch.object(native,'native_summary') as decoder:
                    with self.assertRaisesRegex(native.BenchmarkError,message):native._one_run(Path('/test/native'),Path(directory),100,template)
                    decoder.assert_not_called()
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(native,'run_bounded',return_value=dict(base)),mock.patch.object(native,'bounded_bytes',return_value=b'{}'),mock.patch.object(native,'native_summary',return_value={'verified':True}) as decoder:
                result=native._one_run(Path('/test/native'),Path(directory),100,template)
                self.assertEqual(result['repeat_summary_sha256'],native.digest(native.frozen.canonical({'verified':True})))
                decoder.assert_called_once()

    def test_closed_column_contract_keeps_provenance_unsigned(self):
        for table in ['patients','replications']:
            columns={c['name']:c for c in native.closed_columns(table)}
            for name in ['replication_index','seed']:
                self.assertEqual(columns[name],{'name':name,'physical_type':'UInt64','nullable':False})
        self.assertEqual(len(native.closed_columns('metrics')),24)
        with self.assertRaises(native.BenchmarkError): native.closed_columns('unknown')
