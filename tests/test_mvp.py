import copy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import mvp

class CoverageTests(unittest.TestCase):
    def setUp(self):
        self.catalog=mvp.tasks.derive(ROOT)
        self.recipes=json.loads((ROOT/mvp.RECIPES).read_text())
    def errors(self):return mvp.validate(ROOT,self.catalog,self.recipes)
    def test_complete_mvp_coverage(self):
        self.assertEqual(self.errors(),[])
        self.assertEqual(len(self.recipes['tasks']),81)
    def test_missing_parent_rejected(self):
        self.recipes['tasks'].pop();self.assertTrue(self.errors())
    def test_changed_objective_rejected(self):
        self.recipes['tasks'][1]['task_objective_sha256']='0'*64
        self.assertTrue(any('objective drift' in e for e in self.errors()))
    def test_skipped_join_rejected(self):
        self.recipes['tasks'][1]['leaves'][1]['dependencies']=[]
        self.assertTrue(any('join mismatch' in e for e in self.errors()))
    def test_worker_self_acceptance_rejected(self):
        self.recipes['tasks'][1]['leaves'][0]['approval']='worker'
        self.assertTrue(any('Self-acceptance' in e for e in self.errors()))
    def test_overlarge_recipe_rejected(self):
        self.recipes['tasks'][1]['leaves'][0]['max_context_bytes']=999999
        self.assertTrue(any('budget' in e for e in self.errors()))

class BindingTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.catalog=mvp.tasks.derive(ROOT)
        # These tests exercise packet preparation for D0.2 leaves. Keep the
        # fixture task open even after the real plan closes D0.2.
        next(t for t in self.catalog['tasks'] if t['id']=='D0.2')['accepted']=False
        self.recipes=json.loads((ROOT/mvp.RECIPES).read_text())
        for name in [str(mvp.RECIPES),'conductor/execution/worker-prompt.md','conductor/execution/mvp/worker-loop.md']:
            p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes((ROOT/name).read_bytes())
        (self.root/'input.txt').write_text('frozen interface\nknown output\n')
        self.leaf=next(l for t in self.recipes['tasks'] for l in t['leaves'] if l['id']=='D0.2.inventory')
        self.binding=dict(reviewer='coordinator',reservation_id='reservation-1',instance_id='all',instance_scope='Extract manifests only',
            instance_set=['all'],reviewed_recipe_sha256=mvp.digest(json.dumps(self.leaf,sort_keys=True).encode()),
            interface_contract='fixed typed JSON',context_slices=[dict(path='input.txt',start=1,end=2)],input_paths=['input.txt'],
            write_paths=['out.json','.artifacts/mvp/D0.2.inventory.all/result.json'],
            verification=[dict(argv=['python3','check.py'],cwd='.',expected_exit=0,oracle='exact fixture content')],prerequisite_receipts={})
    def run_prepare(self,leaf='D0.2.inventory'):
        with patch.object(mvp.subprocess,'check_output',side_effect=['','a'*40]):
            return mvp.prepare(self.root,self.catalog,self.recipes,leaf,self.binding)
    def test_prepares_exact_context_and_source_hashes(self):
        packet,bundle=self.run_prepare()
        self.assertIn('known output',bundle)
        self.assertEqual(packet['source_hashes']['input.txt'],mvp.digest((self.root/'input.txt').read_bytes()))
        self.assertEqual(packet['base_commit'],'a'*40)
    def test_stale_review_rejected(self):
        self.binding['reviewed_recipe_sha256']='0'*64
        with self.assertRaisesRegex(ValueError,'review drift'):self.run_prepare()
    def test_context_budget_rejected(self):
        (self.root/'input.txt').write_text('x'*25000+'\n')
        self.binding['context_slices'][0]['end']=1
        with self.assertRaisesRegex(ValueError,'budget'):self.run_prepare()
    def test_context_traversal_rejected(self):
        self.binding['context_slices'][0]['path']='../outside'
        with self.assertRaisesRegex(ValueError,'Unsafe context'):self.run_prepare()
    def test_unaccepted_parent_rejected(self):
        next(t for t in self.catalog['tasks'] if t['id']=='D0.1')['accepted']=False
        with self.assertRaisesRegex(ValueError,'Parent prerequisites'):self.run_prepare()
    def test_unaccepted_internal_leaf_rejected(self):
        leaf=next(l for t in self.recipes['tasks'] for l in t['leaves'] if l['id']=='D0.2.capabilities')
        self.binding['reviewed_recipe_sha256']=mvp.digest(json.dumps(leaf,sort_keys=True).encode())
        with self.assertRaisesRegex(ValueError,'Unaccepted leaf'):self.run_prepare('D0.2.capabilities')
    def test_cross_repository_source_drift_rejected(self):
        packet,bundle=self.run_prepare();ctx=self.root/packet['context_paths'][0];ctx.parent.mkdir(parents=True);ctx.write_text(bundle)
        packet['input_hashes'][packet['context_paths'][0]]=mvp.digest(ctx.read_bytes())
        (self.root/'input.txt').write_text('changed')
        errors=mvp.tasks.packet_errors(self.root,packet,check_git=False)
        self.assertTrue(any('Cross-repository source hash drift' in e for e in errors))

if __name__=='__main__':unittest.main()

class ExplicitLeafDependencyTests(unittest.TestCase):
    def test_actual_c2_join_preserves_transit_and_owned_resume(self):
        catalog = mvp.tasks.derive(ROOT)
        recipes = json.loads((ROOT/mvp.RECIPES).read_text())
        self.assertEqual(mvp.validate(ROOT, catalog, recipes), [])
        leaves = {l['id']: l for t in recipes['tasks'] for l in t['leaves']}
        self.assertEqual(leaves['C2.3.routing']['dependencies'], ['C2.0.red-tests'])
        self.assertEqual(leaves['C2.2.seeds']['dependencies'], ['C2.2.policy', 'C2.3.dispatch'])
        self.assertEqual(leaves['C2.1.paired']['dependencies'], ['C2.1.mode', 'C2.2.seeds', 'C2.3.dispatch'])
        leaves['C2.2.seeds']['dependencies'] = ['C2.2.policy']
        self.assertTrue(any('mismatch' in e for e in mvp.validate(ROOT, catalog, recipes)))

    def test_unknown_override_and_leaf_cycle_fail(self):
        catalog = mvp.tasks.derive(ROOT)
        recipes = json.loads((ROOT/mvp.RECIPES).read_text())
        task = next(t for t in catalog['tasks'] if t['id'] == 'C2.2')
        task['leaf_dependency_overrides']['C2.2.missing'] = ['C2.0.red-tests']
        self.assertIn('Unknown leaf dependency override', mvp.validate(ROOT, catalog, recipes))
        leaves = {l['id']: l for t in recipes['tasks'] for l in t['leaves']}
        leaves['C2.3.routing']['dependencies'] = ['C2.2.seeds']
        self.assertTrue(any('cycle' in e for e in mvp.validate(ROOT, catalog, recipes)))
