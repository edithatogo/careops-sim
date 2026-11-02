import copy
import json
import hashlib
from pathlib import Path
import unittest

from tools.composition_contract import validate_contract

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = ROOT / 'model-inputs/core/a7-evaluation.synthetic.json'


class CompositionEvaluationContractTests(unittest.TestCase):
    def fixture(self):
        return json.loads(FIXTURE.read_text())

    def reject(self, change):
        value = self.fixture()
        change(value)
        with self.assertRaises(ValueError):
            validate_contract(value)

    def test_two_consumer_join_binds_actual_source_and_accepted_receipt_bytes(self):
        join=json.loads((ROOT/'conductor/evidence/a7-two-consumer-20261103/join.json').read_text())
        self.assertEqual({c['domain'] for c in join['consumers']},{'ed','birthing'})
        self.assertEqual(join['scope'],'generic_synthetic_bounded_reuse_only')
        self.assertEqual(join['ed_source_sha256'],hashlib.sha256((ROOT/'crates/careops-ed/src/lib.rs').read_bytes()).hexdigest())
        for evidence in join['evidence']:
            self.assertEqual(hashlib.sha256((ROOT/evidence['path']).read_bytes()).hexdigest(),evidence['sha256'])
        self.assertEqual(join['hosted_qualification']['ed_mutants']['viable_survivors'],0)
        self.assertIn('shared ED/birthing event loop',join['nonclaims'])
        self.assertIn('clinical/site/operational acceptance',join['nonclaims'])

    def test_two_consumer_observation_accounting_retains_missing_delayed_and_excluded(self):
        result = validate_contract(self.fixture())
        self.assertEqual(result['observation_counts'], {'observed': 1, 'right_censored': 1, 'delayed':1, 'unknown':0, 'missing': 1, 'excluded': 1})
        self.assertEqual(result['eligible_observations'], 4)
        self.assertEqual(result['follow_up'], {'ED-observed': 6, 'birth-delayed': None, 'ED-window-end': 8, 'birth-missing': None})
        self.assertEqual(result['verified_event_free_follow_up']['birth-delayed'],6)
        self.assertEqual(result['available_features_at_audit_cutoff'], ['arrival-known'])
        self.assertEqual((result['latent_requests'], result['latent_unmet']), (2, 1))
        self.assertFalse(result['identifiable_micro_parameters'])

    def test_zero_observed_duration_is_distinct_from_missing(self):
        value = self.fixture()
        value['observation']['records'][0].update(event_tick=0, available_at=0)
        result = validate_contract(value)
        self.assertEqual(result['follow_up']['ED-observed'], 0)
        self.assertIsNone(result['follow_up']['birth-missing'])

    def test_event_at_window_end_is_not_counted_as_observed_even_when_known(self):
        value = self.fixture()
        value['observation']['as_of'] = 20
        value['observation']['records'][1]['endpoint_status']='observed'
        value['observation']['records'][2].update(last_observed=20,event_free_until=20,event_free_available_at=20)
        result = validate_contract(value)
        self.assertEqual(result['observation_counts']['right_censored'], 1)
        self.assertEqual(result['follow_up']['ED-window-end'], 18)

    def test_used_late_or_latent_features_reject(self):
        for feature in ['late-code', 'future-endpoint']:
            with self.subTest(feature=feature):
                self.reject(lambda v: v['observation']['used_features'].append({'feature_id':feature,'member_id':'ed-train','prediction_cutoff':1}))

    def test_feature_available_before_audit_but_after_prediction_is_not_usable(self):
        def leakage(v):
            v['observation']['features'].append({'id':'late-for-prediction','origin':'derived','available_at':9})
            v['observation']['used_features'].append({'feature_id':'late-for-prediction','member_id':'ed-train','prediction_cutoff':1})
        self.reject(leakage)
        self.reject(lambda v:v['observation']['used_features'][0].update(prediction_cutoff=10))

    def test_delayed_or_unknown_endpoint_never_infers_censoring_from_last_contact(self):
        self.reject(lambda v:v['observation']['records'][1].update(endpoint_status='right_censored',censor_reason='invalid-inference',event_free_until=10,event_free_available_at=10))
        self.reject(lambda v:v['observation']['records'][1].update(endpoint_status='right_censored',censor_reason='cannot-hide-late-event'))
        self.reject(lambda v:v['observation']['records'][1].update(endpoint_status='right_censored',censor_reason='unsupported',event_free_until=None,event_free_available_at=None))
        value=self.fixture()
        value['observation']['records'][0].update(endpoint_status='unknown',event_tick=None,available_at=None)
        result=validate_contract(value)
        self.assertEqual(result['observation_counts']['unknown'],1)
        self.assertIsNone(result['follow_up']['ED-observed'])

    def test_future_last_known_observation_and_reversed_clocks_reject(self):
        self.reject(lambda v: v['observation']['records'][0].update(last_observed=11))
        self.reject(lambda v: v['observation']['records'][0].update(available_at=5))
        self.reject(lambda v: v['observation']['records'][1].update(event_tick=0))

    def test_known_family_or_longitudinal_person_cannot_leak_between_stages(self):
        def family(v):
            v['validation']['members'][2].update(split='selection', at=12)
        self.reject(family)
        def person(v):
            v['validation']['members'].append({'id':'readmission','person_key':'person-ed-1','family_key':None,'cluster':'person-ed-1','split':'locked_test','at':22})
        self.reject(person)
        self.reject(lambda v: v['validation']['members'][2].update(cluster='invented-independent-cluster'))

    def test_unknown_cluster_evidence_and_retuning_or_bad_time_split_reject(self):
        self.reject(lambda v: v['validation'].update(cluster_status='unknown'))
        self.reject(lambda v: v['validation'].update(retuned_after_test=True))
        self.reject(lambda v: v['validation'].update(test_used_for_selection=True))
        self.reject(lambda v: v['validation']['split_windows'].update(selection=[9,20]))

    def test_total_only_observation_does_not_select_one_micro_decomposition(self):
        self.reject(lambda v: v['validation']['identifiability'].update(selected_decomposition={'walk':2,'work':3}))
        self.reject(lambda v: v['validation']['identifiability'].update(micro_hazards_identified=True))
        self.reject(lambda v: v['validation']['identifiability']['candidate_decompositions'][0].update(walk=2))

    def test_all_six_ledgers_remain_separate_with_no_cost_funding_netting(self):
        result = validate_contract(self.fixture())
        self.assertEqual(result['ledger_totals_separate'], {'activity':2,'resource':120,'cost':240,'funding':1000,'outcome':1})
        self.assertEqual(result['equity_records'], [{'id':'equity-access','numerator':1,'denominator':2}])
        self.assertNotIn('net', result)
        self.assertEqual(result['fiscal_activation'], 'proposal_only')
        self.assertEqual(result['fiscal_effective_period'], '2027-01')

    def test_unit_currency_price_year_and_period_mismatches_reject(self):
        for field, bad in [('unit','staff_minute'),('window',[0,19])]:
            with self.subTest(field=field):
                self.reject(lambda v: v['accounting']['entries'][2].update({field:bad}))
        self.reject(lambda v: v['accounting']['entries'][3].update(currency='USD'))
        self.reject(lambda v: v['accounting']['entries'][4].update(price_year=2027))

    def test_duplicate_accounting_component_and_payment_as_economic_cost_reject(self):
        def duplicate(v):
            row=copy.deepcopy(v['accounting']['entries'][3]);row['id']='second-charge';v['accounting']['entries'].append(row)
        self.reject(duplicate)
        self.reject(lambda v: v['accounting']['entries'][3].update(basis='payment'))
        self.reject(lambda v: v['accounting']['entries'][-1].update(denominator=0))

    def test_policy_lag_and_offline_exchange_boundaries_reject_early_feedback(self):
        for change in [{'lag_months':0},{'effective_period':'2026-11'},{'evidence_window':[0,19]},
                       {'capacity_unit':'staff_hour'},{'monthly_extrapolation':True},
                       {'combines_cost_and_funding':True},{'activation':'active'}]:
            with self.subTest(change=change):
                self.reject(lambda v: v['fiscal_exchange'].update(change))

    def test_version_provenance_and_base_contract_drift_are_not_silent(self):
        self.reject(lambda v: v.update(schema_version=2))
        self.reject(lambda v: v['provenance'].update(empirical_data_used=True))
        self.reject(lambda v: v['extends'][0].update(sha256='0'*64))
        self.reject(lambda v: v['observation'].update(cohort_rule=' '))
        self.reject(lambda v: v['fiscal_exchange'].update(rule_version=' '))


if __name__ == '__main__':
    unittest.main()
