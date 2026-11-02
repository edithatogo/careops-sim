"""Validate versioned synthetic composition/evaluation artifacts.

This is a contract/reference-oracle tool, not a simulation, fitter or fiscal
optimizer. Existing Rust execution and calibration owners remain unchanged.
"""
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
BASE_CONTRACTS = {
    'conductor/design/calibration/c0.2-contract.md',
    'conductor/design/calibration/calibration-v1.schema.json',
    'model-inputs/ed/calibration/p31-censoring-dependence-fixtures.json',
    'model-inputs/ed/calibration/p31-identifiability-fixtures.json',
    'model-inputs/ed/calibration/p32-holdout-validation.synthetic.json',
}


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def unique(rows, key='id'):
    require(len({row[key] for row in rows}) == len(rows), 'duplicate record identity')


def validate_contract(value, root=ROOT):
    schema = json.loads((root / 'conductor/design/composition-evaluation-v1.schema.json').read_text())
    Draft202012Validator.check_schema(schema)
    require(not list(Draft202012Validator(schema).iter_errors(value)), 'composition schema violation')
    refs = value['extends']
    unique(refs, 'path')
    require({ref['path'] for ref in refs} == BASE_CONTRACTS, 'unbound base contracts')
    for ref in refs:
        path = root / ref['path']
        require(not path.is_symlink() and path.is_file(), 'unsupported base contract')
        require(hashlib.sha256(path.read_bytes()).hexdigest() == ref['sha256'], 'base contract hash drift')

    observation = value['observation']
    start, end = observation['window']
    cutoff = observation['as_of']
    require(start < end and start <= cutoff <= end, 'invalid observation window/cutoff')
    require(observation['origin'].startswith('synthetic:'), 'missing synthetic time origin')
    unique(observation['records'])
    require({r['domain'] for r in observation['records']} == {'ed', 'birthing'}, 'both consumers required')
    counts = Counter({k:0 for k in ['observed','right_censored','delayed','unknown','missing','excluded']})
    follow_up = {}
    verified_event_free = {}
    for record in observation['records']:
        risk = record['risk_start']
        event, available, last = (record[k] for k in ['event_tick', 'available_at', 'last_observed'])
        status=record['endpoint_status']
        free,free_available=record['event_free_until'],record['event_free_available_at']
        require(start <= risk < end and risk <= cutoff, 'risk start is outside historical cohort')
        require(event is None or event >= risk, 'event precedes risk start')
        require(available is None or (event is not None and available >= event), 'invalid endpoint knowledge time')
        require(last is None or risk <= last <= cutoff, 'future or reversed last-known observation')
        require((free is None)==(free_available is None), 'incomplete event-free evidence')
        if free is not None:
            require(risk<=free<=cutoff and free<=free_available<=cutoff, 'future/reversed event-free evidence')
            require(last is not None and free<=last, 'event-free evidence exceeds known follow-up')
            require(event is None or free<=event, 'event-free claim extends past an actual endpoint')
        if status=='excluded':
            require(record['excluded_reason'] is not None, 'exclusion has no reason')
            counts['excluded'] += 1
            continue
        require(record['excluded_reason'] is None,'exclusion reason/status mismatch')
        known_event=event is not None and event<end and available is not None and available<=cutoff
        late_event=event is not None and event<end and event<=cutoff and not known_event
        if status=='observed':
            require(known_event,'observed endpoint is unavailable or outside window')
            require(last is None or last >= event, 'observed event exceeds follow-up')
            follow_up[record['id']] = event - risk
        elif status=='right_censored':
            require(not known_event and not late_event and free is not None and record['censor_reason'] is not None, 'censoring lacks explicit event-free evidence/reason or hides delayed event')
            follow_up[record['id']]=min(free,end)-risk
        else:
            require(not known_event,'known endpoint contradicts unresolved status')
            if status=='delayed':
                require(late_event,'invalid delayed endpoint classification')
            if status=='unknown':
                require(event is None and available is None,'unknown endpoint contradicts retained event evidence')
            if status=='missing':
                require(event is None and available is None,'missing endpoint contradicts retained event evidence')
            require(record['censor_reason'] is None,'unresolved endpoint cannot silently claim censoring')
            follow_up[record['id']] = None
        if status!='right_censored':
            require(record['censor_reason'] is None,'non-censored record carries censor reason')
        counts[status]+=1
        verified_event_free[record['id']]=None if free is None else free-risk
    require(sum(counts.values()) == len(observation['records']), 'observation accounting mismatch')
    features = observation['features']
    unique(features)
    visible = {f['id'] for f in features if f['origin'] != 'latent' and f['available_at'] is not None and f['available_at'] <= cutoff}
    unique(observation['latent_requests'])
    unmet = sum(not r['served'] for r in observation['latent_requests'])

    validation = value['validation']
    splits = validation['split_windows']
    previous = None
    for name in ['train', 'selection', 'locked_test']:
        a, b = splits[name]
        require(a < b and (previous is None or a >= previous), 'overlapping/reversed held-out windows')
        previous = b
    unique(validation['members'])
    members={m['id']:m for m in validation['members']}
    feature_map={f['id']:f for f in features}
    for use in observation['used_features']:
        require(use['member_id'] in members and use['feature_id'] in feature_map,'unbound prediction feature use')
        prediction=members[use['member_id']]['at']
        require(use['prediction_cutoff']==prediction and prediction<=cutoff,'prediction cutoff does not bind actual member')
        feature=feature_map[use['feature_id']]
        require(feature['origin']!='latent' and feature['available_at'] is not None and feature['available_at']<=prediction,'future/latent feature at prediction time')
    cluster_folds = defaultdict(set)
    person_folds = defaultdict(set)
    for member in validation['members']:
        a, b = splits[member['split']]
        require(a <= member['at'] < b, 'member outside declared split window')
        require(member['cluster'] == (member['family_key'] or member['person_key']), 'split cluster does not bind known family/person')
        cluster_folds[member['cluster']].add(member['split'])
        person_folds[member['person_key']].add(member['split'])
    require(all(len(folds) == 1 for folds in cluster_folds.values()), 'person/family leakage across splits')
    require(all(len(folds) == 1 for folds in person_folds.values()), 'longitudinal person leakage across splits')
    require({r['split'] for r in validation['members']} == set(splits), 'missing held-out stage')
    ident = validation['identifiability']
    points = ident['candidate_decompositions']
    require(len({(p['walk'], p['work']) for p in points}) == len(points), 'duplicate parameter decomposition')
    require(all(p['walk'] + p['work'] == ident['observed_total'] for p in points), 'inconsistent total-only candidate')
    if len(points) > 1:
        require(ident['selected_decomposition'] is None, 'non-identifiable micro decomposition was selected')
    else:
        require(ident['selected_decomposition'] in [None, points[0]], 'selected parameter outside declared grid')

    accounting = value['accounting']
    require(accounting['window'] == observation['window'] and accounting['time_unit'] == observation['unit'], 'evaluation/accounting period or unit mismatch')
    entries = accounting['entries']
    unique(entries)
    seen = set()
    totals = Counter()
    equity = []
    expected_basis = {'activity': 'clinical_activity', 'resource': 'resource_use', 'cost': 'resource_use', 'funding': 'payment', 'outcome': 'clinical_outcome', 'equity': 'access'}
    require({e['ledger'] for e in entries} == set(expected_basis), 'missing separate ledger')
    for entry in entries:
        ledger = entry['ledger']
        require(entry['window'] == accounting['window'], 'ledger period mismatch')
        require(entry['unit'] == accounting['ledger_units'][ledger], 'ledger unit mismatch')
        require(entry['basis'] == expected_basis[ledger], 'payment/cost or ledger basis conflation')
        key = (ledger, entry['source_event'], entry['component'], tuple(entry['window']))
        require(key not in seen, 'duplicate accounting of a source component')
        seen.add(key)
        if ledger in ['cost', 'funding']:
            require(entry['currency'] == accounting['currency'] and entry['price_year'] == accounting['price_year'], 'currency/price-year mismatch without conversion')
            require(entry['unit'] == accounting['currency'] + '_minor', 'money scale/unit mismatch')
        else:
            require(entry['currency'] is None and entry['price_year'] is None, 'nonmonetary ledger carries monetary valuation')
        if ledger == 'equity':
            require(entry['denominator'] is not None and 0 < entry['denominator'] and entry['quantity'] <= entry['denominator'], 'invalid access numerator/denominator')
            equity.append({'id': entry['id'], 'numerator': entry['quantity'], 'denominator': entry['denominator']})
        else:
            require(entry['denominator'] is None, 'non-equity ledger has an undeclared ratio')
            totals[ledger] += entry['quantity']

    exchange = value['fiscal_exchange']
    require(exchange['evidence_window'] == accounting['window'] and exchange['evidence_time_unit'] == accounting['time_unit'], 'offline exchange observation period/unit mismatch')
    require(exchange['currency'] == accounting['currency'] and exchange['price_year'] == accounting['price_year'], 'offline exchange valuation mismatch')
    require(exchange['capacity_unit'] == accounting['ledger_units']['resource'], 'offline capacity unit mismatch')
    def period(label):
        year, month = map(int, label.split('-'))
        require(1 <= year <= 9999 and 1 <= month <= 12, 'invalid fiscal calendar period')
        return year * 12 + month - 1
    require(period(exchange['effective_period']) == period(exchange['decision_period']) + exchange['lag_months'], 'policy lag/effective-period mismatch')
    return {'observation_counts': dict(counts), 'eligible_observations': len(observation['records'])-counts['excluded'],
            'follow_up': follow_up, 'verified_event_free_follow_up':verified_event_free,'available_features_at_audit_cutoff': sorted(visible),
            'latent_requests': len(observation['latent_requests']), 'latent_unmet': unmet,
            'identifiable_micro_parameters': False, 'ledger_totals_separate': dict(totals),
            'equity_records': equity, 'fiscal_activation': exchange['activation'], 'fiscal_effective_period': exchange['effective_period']}


def reject_nonfinite(_):
    raise ValueError('nonfinite JSON constant')


def main():
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('artifact', type=Path)
    args = parser.parse_args()
    require(args.artifact.stat().st_size <= 262144, 'contract artifact byte bound exceeded')
    value = json.loads(args.artifact.read_text(), parse_constant=reject_nonfinite)
    print(json.dumps(validate_contract(value), sort_keys=True))


if __name__ == '__main__':
    main()
