import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('context',Path(__file__).parents[1]/'tools/context.py')
context=importlib.util.module_from_spec(spec);spec.loader.exec_module(context)

class ContextTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.track=self.root/'conductor/tracks/example'
        self.track.mkdir(parents=True)
        for name in context.REQUIRED:(self.track/name).write_text('# Document\n')
        self.metadata={'track_id':'example','status':'proposed'}
        self.save_meta()
        (self.track/'plan.md').write_text('## A0 — Start\n## A1 — Finish\n')
        (self.root/'AGENTS.md').write_text('[Spec](conductor/tracks/example/spec.md)\n')
        self.state={'active_track':'example','active_phase':'A0','next_action':'Review contract','context_paths':['conductor/tracks/example/spec.md'],'submodule_pins':{'libs/kairos':'abc'}}
        self.save_state()
    def save_meta(self):(self.track/'metadata.json').write_text(json.dumps(self.metadata))
    def save_state(self):(self.root/'conductor/current-state.json').write_text(json.dumps(self.state))
    def errors(self):return context.validate(self.root,check_git=False)['errors']
    def test_valid_project(self):self.assertEqual(self.errors(),[])
    def test_missing_context_is_detected(self):
        self.state['context_paths']=['missing.md'];self.save_state()
        self.assertTrue(any('Missing context' in e for e in self.errors()))
    def test_broken_link_is_detected(self):
        (self.track/'spec.md').write_text('[Missing](absent.md)')
        self.assertTrue(any('Broken link' in e for e in self.errors()))
    def test_unknown_milestone_is_detected(self):
        self.metadata['milestone_dependencies']=[{'milestone':'A1','requires':'example:A9'}];self.save_meta()
        self.assertTrue(any('Unknown dependency' in e for e in self.errors()))
    def test_cycle_is_detected(self):
        self.metadata['milestone_dependencies']=[{'milestone':'A0','requires':'example:A1'}];self.save_meta()
        self.assertTrue(any('cycle' in e for e in self.errors()))
    def test_false_completion_is_detected(self):
        self.metadata['status']='complete';self.save_meta()
        self.assertTrue(any('Completion evidence' in e for e in self.errors()))
    def test_submodule_drift_is_detected(self):
        with patch.object(context,'git',return_value='def'):
            self.assertTrue(any('Submodule drift' in e for e in context.validate(self.root)['errors']))
    def test_explicit_independent_phases_are_preserved(self):
        self.metadata['phase_dependencies']={'A0':[],'A1':[]}
        self.metadata['milestone_dependencies']=[{'milestone':'A0','requires':'example:A1'}]
        self.save_meta();self.assertEqual(self.errors(),[])
    def test_missing_phase_dependency_coverage_is_detected(self):
        self.metadata['phase_dependencies']={'A0':[]};self.save_meta()
        self.assertTrue(any('coverage mismatch' in e for e in self.errors()))
    def test_unknown_active_phase_is_detected(self):
        self.state['active_phase']='A9';self.save_state()
        self.assertTrue(any('Active phase' in e for e in self.errors()))
    def test_missing_resume_action_is_detected(self):
        self.state['next_action']='';self.save_state()
        self.assertTrue(any('next_action' in e for e in self.errors()))

if __name__=='__main__':unittest.main()
