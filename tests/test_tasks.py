import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('tasks',Path(__file__).parents[1]/'tools/tasks.py')
tasks=importlib.util.module_from_spec(spec);spec.loader.exec_module(tasks)

def task(name,deps=(),paths=('src',),accepted=False):
    return {'id':name,'dependencies':list(deps),'write_reservations':list(paths),'accepted':accepted}

class SchedulingTests(unittest.TestCase):
    def catalog(self):
        return {'tasks':[task('D0.1',accepted=True), task('Q0.1',['D0.1'],['queue']),
                         task('C0.1',['D0.1'],['arrow']),task('E0.1',['Q0.1','C0.1'],['ed'])]}
    def test_serial_parallel_have_same_tasks_and_barriers(self):
        catalog=self.catalog()
        serial=tasks.schedule(catalog,1);parallel=tasks.schedule(catalog,4)
        self.assertEqual({x for w in serial for x in w},{x for w in parallel for x in w})
        self.assertEqual(parallel,[['Q0.1','C0.1'],['E0.1']])
    def test_conflicting_ancestors_are_serialized(self):
        catalog={'tasks':[task('Q0.1',paths=['src']),task('C0.1',paths=['src/arrow'])]}
        self.assertEqual(len(tasks.select(catalog,set(),4)),1)
    def test_similar_prefixes_are_not_conflicts(self):
        self.assertFalse(tasks.overlap('src/des','src/descriptive'))
    def test_active_reservations_block_selection(self):
        self.assertEqual(tasks.select(self.catalog(),{'D0.1'},4,['queue','arrow']),[])
    def test_active_reservation_descendant_blocks_overlapping_task_only(self):
        catalog={'tasks':[task('Q1.1',paths=['resources/des']),
                          task('C1.1',paths=['reports'])]}
        selected=tasks.select(catalog,set(),4,['resources/des/worker'])
        self.assertEqual([item['id'] for item in selected],['C1.1'])
    def test_active_reservation_ancestor_blocks_overlapping_task_only(self):
        catalog={'tasks':[task('Q1.2',paths=['resources/des/worker']),
                          task('C1.2',paths=['reports'])]}
        selected=tasks.select(catalog,set(),4,['resources/des'])
        self.assertEqual([item['id'] for item in selected],['C1.2'])
    def test_unaccepted_prerequisites_block(self):
        self.assertEqual(tasks.select(self.catalog(),set(),4)[0]['id'],'D0.1')
    def test_cycle_rejected(self):
        with self.assertRaisesRegex(ValueError,'cycle'):
            tasks.validate({'tasks':[task('Q0.1',['C0.1']),task('C0.1',['Q0.1'])]})
    def test_unknown_dependency_rejected(self):
        with self.assertRaisesRegex(ValueError,'Unknown'):
            tasks.validate({'tasks':[task('Q0.1',['Q9.9'])]})
    def test_false_acceptance_rejected(self):
        with self.assertRaisesRegex(ValueError,'unaccepted prerequisite'):
            tasks.validate({'tasks':[task('Q0.1'),task('Q0.2',['Q0.1'],accepted=True)]})
    def test_parameter_research_parallel_join_and_delivery_barriers(self):
        catalog=tasks.derive(tasks.ROOT)
        by_id={t['id']:t for t in catalog['tasks']}
        self.assertIn('P0.4',by_id['P1.1']['dependencies'])
        self.assertIn('P0.4',by_id['P2.1']['dependencies'])
        self.assertFalse(any(tasks.overlap(a,b) for a in by_id['P1.1']['write_reservations'] for b in by_id['P2.1']['write_reservations']))
        self.assertTrue({'P1.4','P2.4'} <= set(by_id['P3.1']['dependencies']))
        self.assertIn('P4.4',by_id['E1.1']['dependencies'])
        self.assertIn('P5.4',by_id['C6.1']['dependencies'])

    def test_queue_implementation_waits_for_readiness_and_ci_milestones(self):
        catalog=tasks.derive(tasks.ROOT)
        by_id={t['id']:t for t in catalog['tasks']}
        # Q0 design is accepted. Q1 code work still requires D2;
        # a Q-local DAG alone must not make implementation appear dispatchable.
        self.assertIn('D1.6',by_id['Q0.1']['dependencies'])
        self.assertEqual(
            by_id['Q0.2']['write_reservations'],
            ['conductor/design/queue','libs/kairos/conductor/design/queue'])
        self.assertTrue({'D2.5','Q0.4'} <= set(by_id['Q1.1']['dependencies']))
        self.assertTrue({'conductor/module-readiness.md','conductor/evidence'} <= set(by_id['D0.2']['write_reservations']))
        self.assertTrue({'conductor/decisions','conductor/current-state.json',
                         'conductor/tracks/development_readiness_20260925/plan.md'}
                        <= set(by_id['D0.3']['write_reservations']))
        accepted={t['id'] for t in catalog['tasks'] if t['accepted']}
        candidates={t['id'] for t in tasks.select(catalog,accepted,20)}
        self.assertNotIn('P1.4',candidates)  # Accepted P1.4 is no longer a preparation candidate.
        self.assertNotIn('P2.1',candidates)  # P2.1 leaves accepted.
        self.assertNotIn('P2.2',candidates)  # P2.2 was accepted.
        self.assertNotIn('P2.3',candidates)  # P2.3 was accepted.
        self.assertNotIn('P2.4',candidates)  # P2.4 was accepted.
        self.assertNotIn('P3.1',candidates)  # P3.1 is accepted at fixture-only scope.
        self.assertNotIn('P3.2',candidates)  # P3.2 is accepted at bounded method/gap-inventory scope.
        self.assertNotIn('P3.3',candidates)  # P3.3 evidence contracts are accepted; P3.4 phase review remains.
        if by_id['P3.4']['accepted']:
            self.assertNotIn('P3.4',candidates)
            self.assertIn('E0.4',by_id['P4.1']['dependencies'])
            def first_ready_p4(accepted_ids):
                for milestone in ('P4.1','P4.2','P4.3','P4.4'):
                    task=by_id[milestone]
                    if milestone not in accepted_ids and set(task['dependencies']) <= accepted_ids:
                        return milestone
                return None
            if by_id['E0.4']['accepted']:
                next_p4=first_ready_p4(accepted)
                if next_p4 is None:
                    self.assertTrue(all(by_id[m]['accepted'] for m in ('P4.1','P4.2','P4.3','P4.4')))
                else:
                    self.assertIn(next_p4,candidates)
                before_e0_closeout=accepted-{'E0.4'}
                before_e0_candidates={t['id'] for t in tasks.select(catalog,before_e0_closeout,20)}
                self.assertNotIn('P4.1',before_e0_candidates)
            else:
                self.assertNotIn('P4.1',candidates)  # P4 profiles wait for the E0 runtime/schema closeout.
                after_e0_closeout=accepted|{'E0.2','E0.3','E0.4'}
                after_e0_candidates={t['id'] for t in tasks.select(catalog,after_e0_closeout,20)}
                next_p4=first_ready_p4(after_e0_closeout)
                if next_p4 is None:
                    self.assertTrue(all(by_id[m]['accepted'] for m in ('P4.1','P4.2','P4.3','P4.4')))
                else:
                    self.assertIn(next_p4,after_e0_candidates)
        else:
            self.assertIn('P3.4',candidates)
        q1_dependencies=set(by_id['Q1.1']['dependencies'])
        self.assertIn('D2.5',q1_dependencies)
        ready_for_q1=(accepted|q1_dependencies)-{'Q1.1'}
        for prerequisite in q1_dependencies:
            with self.subTest(missing=prerequisite):
                blocked=ready_for_q1-{prerequisite}
                self.assertNotIn('Q1.1',[t['id'] for t in tasks.select(catalog,blocked,20)])
        self.assertIn('Q1.1',[t['id'] for t in tasks.select(catalog,ready_for_q1,20)])
        invalid=copy.deepcopy(catalog)
        by_id_invalid={t['id']:t for t in invalid['tasks']}
        by_id_invalid['Q0.1']['accepted']=True
        by_id_invalid['D1.6']['accepted']=False
        with self.assertRaisesRegex(ValueError,'unaccepted prerequisite'):
            tasks.validate(invalid)
    def test_development_readiness_tasks_can_write_owned_evidence_not_kairos_source(self):
        by_id={t['id']:t for t in tasks.derive(tasks.ROOT)['tasks']}
        for identifier in ('D1.1','D1.2','D1.3','D1.4','D1.5'):
            self.assertIn('conductor/evidence',by_id[identifier]['write_reservations'])
            self.assertNotIn('libs/kairos',by_id[identifier]['write_reservations'])
    def test_mvp_and_v1_exclude_optional_feature_dependencies(self):
        by_id={t['id']:t for t in tasks.derive(tasks.ROOT)['tasks']}
        def ancestors(identifier):
            found=set();todo=[identifier]
            while todo:
                current=todo.pop()
                if current not in found:
                    found.add(current);todo.extend(by_id[current]['dependencies'])
            return found
        mvp=ancestors('E2.4');v1=ancestors('E4.4')
        self.assertTrue({'P4.4','Q4.5','C1.4','C2.4','D2.5'} <= mvp)
        self.assertFalse(any(x.startswith(('C3.','C4.','C5.','C6.','D4.','E3.','E4.')) for x in mvp))
        self.assertTrue({'E2.4','E3.4','Q5.4','C6.5','P5.4','D3.5','D4.4'} <= v1)
        self.assertFalse(any(x.startswith(('E5.','E6.','E7.','E8.','D5.')) for x in v1))
    def test_post_v1_work_waits_for_native_release_gate(self):
        by_id={t['id']:t for t in tasks.derive(tasks.ROOT)['tasks']}
        for identifier in ('E5.1','E6.1','D5.1'):
            self.assertIn('E4.4',by_id[identifier]['dependencies'])
    def test_nonpositive_worker_count_rejected(self):
        with self.assertRaises(ValueError):tasks.select(self.catalog(),set(),0)
    def test_real_catalog_modes_cover_all_tasks_without_conflict(self):
        catalog=tasks.derive(tasks.ROOT)
        serial=tasks.schedule(catalog,1);parallel=tasks.schedule(catalog,4)
        self.assertEqual({i for w in serial for i in w},{i for w in parallel for i in w})
        by_id={t['id']:t for t in catalog['tasks']}
        accepted={t['id'] for t in catalog['tasks'] if t['accepted']}
        for wave in parallel:
            for i,identifier in enumerate(wave):
                self.assertTrue(set(by_id[identifier]['dependencies'])<=accepted)
                for other in wave[i+1:]:
                    self.assertFalse(any(tasks.overlap(a,b) for a in by_id[identifier]['write_reservations'] for b in by_id[other]['write_reservations']))
            accepted.update(wave)
        self.assertEqual(len(accepted),len(catalog['tasks']))

class PacketTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);(self.root/'input.md').write_text('fixed contract')
        self.packet={'packet_id':'Q1.1.unit','task_id':'Q1.1','target_repo':'.','base_commit':'a'*40,
          'objective':'One task','context_paths':['input.md'],
          'input_hashes':{'input.md':hashlib.sha256(b'fixed contract').hexdigest()},
          'write_paths':['out.json'],'protected_paths':['input.md'],'leaf_dependencies':[],
          'interface_contract':'fixed','steps':['implement'],
          'verification':[{'argv':['check'],'cwd':'.','expected_exit':0,'oracle':'expected outcome'}],
          'acceptance':['one result'],'result_path':'out.json','stop_conditions':['drift'],'status':'prepared'}
    def errors(self):return tasks.packet_errors(self.root,self.packet,check_git=False)
    def test_bound_packet_valid(self):self.assertEqual(self.errors(),[])
    def test_template_rejected(self):
        self.packet['base_commit']='REQUIRED'
        self.assertTrue(self.errors())
    def test_input_drift_rejected(self):
        (self.root/'input.md').write_text('changed')
        self.assertTrue(any('hash drift' in e for e in self.errors()))
    def test_protected_write_rejected(self):
        self.packet['write_paths'].append('input.md')
        self.assertTrue(any('Protected write' in e for e in self.errors()))
    def test_traversal_rejected(self):
        self.packet['write_paths'].append('../escape')
        self.assertTrue(any('Unsafe' in e for e in self.errors()))
    def test_missing_oracle_rejected(self):
        self.packet['verification'][0]['oracle']=''
        self.assertTrue(any('oracle' in e for e in self.errors()))
    def test_failing_final_acceptance_rejected(self):
        self.packet['verification'][0]['expected_exit']=1
        self.assertTrue(any('Final verification' in e for e in self.errors()))
    def test_oversize_packet_rejected(self):
        self.packet['write_paths']=[f'file{i}' for i in range(6)]
        self.assertTrue(any('Oversized' in e for e in self.errors()))
    def test_stale_base_rejected(self):
        self.assertTrue(any('Base commit drift' in e for e in tasks.packet_errors(self.root,self.packet)))

if __name__=='__main__':unittest.main()

class ExplicitDependencyTests(unittest.TestCase):
    def test_c2_independent_foundations_and_full_join(self):
        by = {t['id']: t for t in tasks.derive(tasks.ROOT)['tasks']}
        self.assertEqual(by['C2.2']['dependencies'], ['C2.0'])
        self.assertEqual(by['C2.3']['dependencies'], ['C2.0'])
        self.assertEqual(by['C2.1']['dependencies'], ['C2.2', 'C2.3'])
        self.assertEqual(by['C2.4']['dependencies'], ['C2.1'])
        self.assertEqual(by['C2.0']['dependencies'], ['C0.4', 'P3.4'])

    def test_override_errors_rejected_during_derivation(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            for source in (tasks.ROOT/'conductor/tracks').iterdir():
                if not source.is_dir(): continue
                target = root/'conductor/tracks'/source.name
                target.mkdir(parents=True)
                for name in ('metadata.json', 'plan.md'):
                    (target/name).write_bytes((source/name).read_bytes())
            path = root/'conductor/tracks/empirical_calibration_20260925/metadata.json'
            baseline = json.loads(path.read_text())
            for override in ({'C9.9':['C2.0']}, {'C2.3':['C9.9']}, {'C2.3':['C2.1']}, {'C2.3':'C2.0'}):
                meta = copy.deepcopy(baseline)
                meta['task_dependency_overrides'].update(override)
                path.write_text(json.dumps(meta))
                with self.assertRaises(ValueError): tasks.derive(root)

    def test_malformed_override_containers_fail_deliberately(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            target = root/'conductor/tracks/example'
            target.mkdir(parents=True)
            for field in ('task_dependency_overrides', 'leaf_dependency_overrides'):
                for malformed in (None, [], 'C2.0', 7):
                    (target/'metadata.json').write_text(json.dumps({field: malformed}))
                    with self.assertRaisesRegex(ValueError, 'Invalid '+field):
                        tasks.derive(root)
