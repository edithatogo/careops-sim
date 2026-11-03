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

class ArchivedTrackContextTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        (self.root/'conductor/evidence').mkdir(parents=True)
        (self.root/'conductor/evidence/done.md').write_text('# Accepted evidence\n')
        self.archived=self.track('archive','foundation','R14','complete',True)
        self.active=self.track('tracks','next','R20','proposed',False,
            [{'milestone':'R20','requires':'foundation:R14'}])

    def track(self,area,name,phase,status,accepted,dependencies=None):
        path=self.root/'conductor'/area/name;path.mkdir(parents=True)
        for file in ('spec.md','index.md','agent-contract.md','risk-register.md','test-matrix.md','handoff.md'):
            (path/file).write_text('# Document\n')
        metadata={'track_id':name,'status':status,'phase_dependencies':{phase:[]},
                  'milestone_dependencies':dependencies or [],
                  'completion_evidence':['conductor/evidence/done.md'] if status=='complete' else []}
        (path/'metadata.json').write_text(json.dumps(metadata))
        mark='x' if accepted else ' '
        (path/'plan.md').write_text(f'## {phase} — Work\n- [{mark}] {phase}.1 Complete reviewed work\n')
        return path

    def errors(self):
        state={'active_track':'next','active_phase':'R20','next_action':'Prepare next reviewed packet',
               'context_paths':['conductor/tracks/next/spec.md'],'submodule_pins':{}}
        (self.root/'conductor/current-state.json').write_text(json.dumps(state))
        (self.root/'AGENTS.md').write_text('[Prior spec](conductor/archive/foundation/spec.md)\n')
        return context.validate(self.root,check_git=False)['errors']

    def test_complete_archive_preserves_milestone_dependency_and_current_active_api(self):
        self.assertEqual(self.errors(),[])

    def test_archived_pending_or_unknown_status_and_unaccepted_plan_fail(self):
        path=self.archived/'metadata.json';original=json.loads(path.read_text())
        for status in ('proposed','in_progress','blocked','archived'):
            with self.subTest(status=status):
                path.write_text(json.dumps(dict(original,status=status)))
                self.assertTrue(any('Archived track must be complete' in e for e in self.errors()))
        path.write_text(json.dumps(original))
        (self.archived/'plan.md').write_text('## R14 — Work\n- [ ] R14.1 Pending\n')
        self.assertTrue(any('Archived track has incomplete' in e for e in self.errors()))

    def test_checked_task_cannot_hide_pending_or_malformed_checkbox_sibling(self):
        for sibling in (
            '- [~] R14.2 In progress', '- [xx] R14.2 Malformed status',
            '- [-] R14.2 Unsupported status', '- [x] Missing numbered task ID',
            '+ [x] R14.2 Unsupported task marker',
        ):
            with self.subTest(sibling=sibling):
                (self.archived/'plan.md').write_text(
                    '## R14 — Work\n- [x] R14.1 Accepted\n'+sibling+'\n')
                self.assertTrue(any('Archived track has incomplete or malformed tasks' in error
                                    for error in self.errors()))

    def test_duplicate_archive_id_missing_metadata_and_broken_links_are_detected(self):
        duplicate=self.track('tracks','foundation','R14','complete',True)
        self.assertTrue(any('Duplicate track ID' in e for e in self.errors()))
        import shutil
        shutil.rmtree(duplicate)
        (self.archived/'spec.md').write_text('[Missing](absent.md)')
        self.assertTrue(any('Broken link' in e for e in self.errors()))
        (self.archived/'metadata.json').unlink()
        self.assertTrue(any('metadata.json' in e for e in self.errors()))

    def test_bare_dependency_followup_resolves_archived_spec(self):
        register={'schema_version':1,'receipts':[{'owner_track':'70-polars-native-data-processing',
            'direct_dependency_disposition':'existing','impact_review':{'status':'complete',
            'outcome':'no_action','implications':['Recorded acceptance'],'opportunities':[],
            'evidence':['conductor/evidence/done.md'],'follow_up':'foundation',
            'upstream_decision':'accepted','reviewer':'bounded-review',
            'consumer_context':'synthetic archive followup fixture','rationale':'Retained acceptance',
            'upstream_commit':'a'*40,'reviewed_at':'2026-11-04'}}]}
        (self.root/'conductor/dependency-handoffs.json').write_text(json.dumps(register))
        self.assertEqual(context.validate_dependency_handoffs(self.root),[])


if __name__=='__main__':unittest.main()
