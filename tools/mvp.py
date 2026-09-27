#!/usr/bin/env python3
"""Check MVP leaf coverage; prepare bounded Luna packets from coordinator bindings.
No model launch, lease acquisition, command execution or acceptance authority.
"""
import argparse
import hashlib
import json
import re
from pathlib import Path
import subprocess
import tasks

ROOT = Path(__file__).resolve().parents[1]
RECIPES = Path('conductor/execution/mvp/recipes.json')


def digest(data):
    return hashlib.sha256(data).hexdigest()


def closure(catalog, terminal='E2.4'):
    by = {t['id']: t for t in catalog['tasks']}
    found = set()
    todo = [terminal]
    while todo:
        current = todo.pop()
        if current not in found:
            found.add(current)
            todo.extend(by[current]['dependencies'])
    return found


def validate(root, catalog, recipes):
    errors = []
    by = {t['id']: t for t in catalog['tasks']}
    rows = recipes.get('tasks', [])
    ids = [t['task_id'] for t in rows]
    if len(ids) != len(set(ids)) or set(ids) != closure(catalog):
        errors.append('MVP parent coverage mismatch or duplicate')
    leaves = [leaf for t in rows for leaf in t['leaves']]
    leaf_ids = [leaf['id'] for leaf in leaves]
    if len(leaf_ids) != len(set(leaf_ids)):
        errors.append('Duplicate leaf ID')
    last = {t['task_id']: t['leaves'][-1]['id'] for t in rows if t['leaves']}
    for row in rows:
        parent = by.get(row['task_id'])
        if not parent:
            errors.append('Unknown parent'); continue
        if row['task_objective_sha256'] != digest(parent['objective'].encode()):
            errors.append(f"Parent objective drift: {parent['id']}")
        if row['parent_dependencies'] != parent['dependencies']:
            errors.append(f"Parent dependency drift: {parent['id']}")
        if row['context_sources'] != parent['context_paths'] or row['source'] != parent['source']:
            errors.append(f"Parent context/source drift: {parent['id']}")
        if not row['leaves']:
            errors.append('Parent lacks leaves')
        for i, leaf in enumerate(row['leaves']):
            expected = [row['leaves'][i-1]['id']] if i else [last.get(d) for d in parent['dependencies']]
            if leaf['dependencies'] != expected:
                errors.append(f"Leaf prerequisite/join mismatch: {leaf['id']}")
            if not leaf['id'].startswith(parent['id']+'.'):
                errors.append('Leaf outside parent')
            for field in ('output','oracle','fanout','approval'):
                if not leaf.get(field): errors.append(f'Missing {field}')
            if leaf.get('model') != 'gpt-6-luna': errors.append('Unexpected model')
            if leaf.get('target_repo') not in ('.','libs/kairos'): errors.append('Invalid repository')
            if leaf.get('role') not in ('worker','proposal_or_review'): errors.append('Invalid role')
            if leaf.get('approval') != 'coordinator_acceptance_required': errors.append('Self-acceptance forbidden')
            limits={'max_write_paths':5,'max_commands':3,'max_context_bytes':24000,'max_corrections':2,'timebox_minutes':90}
            for field, maximum in limits.items():
                if not 0 < leaf.get(field,0) <= maximum: errors.append(f'Invalid budget: {field}')
            for path in row['context_sources']:
                if not (root/path).is_file(): errors.append(f'Missing context source: {path}')
    return errors


def prepare(root, catalog, recipes, leaf_id, binding):
    """Bindings must be reviewed, exact and current; returns packet/context text."""
    row, leaf = next((row,leaf) for row in recipes['tasks'] for leaf in row['leaves'] if leaf['id']==leaf_id)
    by = {t['id']:t for t in catalog['tasks']}
    if by[row['task_id']]['accepted']: raise ValueError('Parent already accepted; do not re-execute')
    if any(not by[d]['accepted'] for d in row['parent_dependencies']): raise ValueError('Parent prerequisites not accepted')
    required=('reviewer','reservation_id','instance_id','instance_scope','instance_set','interface_contract',
              'context_slices','input_paths','write_paths','verification','prerequisite_receipts','reviewed_recipe_sha256')
    if any(k not in binding for k in required): raise ValueError('Incomplete coordinator binding')
    if binding['reviewed_recipe_sha256'] != digest(json.dumps(leaf,sort_keys=True).encode()): raise ValueError('Recipe review drift')
    for key in ('reviewer','reservation_id','instance_id','instance_scope','interface_contract'):
        if not isinstance(binding[key],str) or not binding[key].strip() or binding[key]=='REQUIRED': raise ValueError(f'Unresolved {key}')
    if binding['instance_id'] not in binding['instance_set'] or len(set(binding['instance_set']))!=len(binding['instance_set']): raise ValueError('Invalid instance join set')
    if not re.fullmatch(r'[A-Za-z0-9_.-]+',binding['instance_id']): raise ValueError('Unsafe instance ID')
    target = root/leaf['target_repo']
    status=subprocess.check_output(['git','-C',str(target),'status','--porcelain'],text=True)
    if status.strip(): raise ValueError('Target checkout must be clean before binding')
    base=subprocess.check_output(['git','-C',str(target),'rev-parse','HEAD'],text=True).strip()
    # Existing parent closeouts are authoritative. Internal leaf joins require
    # reviewed receipts; the coordinator separately verifies their behavioral truth.
    for predecessor in leaf['dependencies']:
        if by['.'.join(predecessor.split('.')[:2])]['accepted']: continue
        receipt=binding['prerequisite_receipts'].get(predecessor)
        if not receipt or receipt.get('status')!='accepted' or not receipt.get('reviewer'):
            raise ValueError(f'Unaccepted leaf join: {predecessor}')
        if not receipt.get('all_instances_accepted') or not receipt.get('artifacts'):
            raise ValueError('Incomplete predecessor instances/evidence')
        for path,sha in receipt['artifacts'].items():
            if not tasks.safe_relative(path) or not (root/path).is_file() or digest((root/path).read_bytes())!=sha:
                raise ValueError('Predecessor evidence drift')
    text=[f"# {leaf_id}\n\nModel: gpt-6-luna; output: {leaf['output']}\nOracle: {leaf['oracle']}\n",
          f"Instance: {binding['instance_id']} — {binding['instance_scope']}\nInterface: {binding['interface_contract']}\n"]
    hashes={}
    source_hashes={str(RECIPES):digest((root/RECIPES).read_bytes())}
    if not binding['context_slices']: raise ValueError('Context slices required')
    for item in binding['context_slices']:
        path=item['path']
        if not tasks.safe_relative(path): raise ValueError('Unsafe context path')
        repo=item.get('repo',leaf['target_repo'])
        if repo not in ('.','libs/kairos'): raise ValueError('Unsafe context repository')
        source=root/repo/path;lines=source.read_text().splitlines()
        start,end=item['start'],item['end']
        if not isinstance(start,int) or not isinstance(end,int) or not 1<=start<=end<=len(lines): raise ValueError('Invalid context slice')
        if repo==leaf['target_repo']: hashes[path]=digest(source.read_bytes())
        source_hashes[str(source.relative_to(root))]=digest(source.read_bytes())
        text.append(f'\n## {path}:{start}-{end}\n'+ '\n'.join(lines[start-1:end])+'\n')
    for path in binding['input_paths']:
        if not tasks.safe_relative(path): raise ValueError('Unsafe input path')
        hashes[path]=digest((target/path).read_bytes())
    # Instructions and recipe are part of the bounded context, not an invisible budget.
    for name in ('worker-prompt.md','mvp/worker-loop.md'):
        instruction=root/'conductor/execution'/name
        text.append(instruction.read_text())
        source_hashes[str(instruction.relative_to(root))]=digest(instruction.read_bytes())
    bundle='\n'.join(text)
    if len(bundle.encode())>leaf['max_context_bytes']: raise ValueError('Context budget exceeded; split the packet')
    writes=binding['write_paths'];checks=binding['verification']
    if len(writes)>leaf['max_write_paths'] or len(checks)>leaf['max_commands']: raise ValueError('Packet budget exceeded')
    if not checks or any(not c.get('oracle') for c in checks): raise ValueError('Exact commands/oracles required')
    packet_id=leaf_id+'.'+binding['instance_id']
    context_path=f'.artifacts/mvp/{packet_id}/context.md'
    result_path=f'.artifacts/mvp/{packet_id}/result.json'
    if result_path not in writes: raise ValueError(f'Include result output in allowed writes: {result_path}')
    packet=dict(schema_version=1,packet_id=packet_id,task_id=row['task_id'],target_repo=leaf['target_repo'],
        base_commit=base,model_role=leaf['role'],model_candidate='gpt-6-luna',status='prepared',
        objective=binding['instance_scope'],context_paths=[context_path],input_hashes=hashes,
        write_paths=writes,protected_paths=['.git','conductor/execution','conductor/current-state.json'],
        leaf_dependencies=leaf['dependencies'],interface_contract=binding['interface_contract'],
        steps=['Read bound context; recheck hashes and coordinator reservation.','Execute only the instance output; escalate any undefined interface.',
               'Run exact verification; stop after two failed corrections.','Return actual diff, commands and result for coordinator review.'],
        verification=checks,acceptance=[leaf['oracle'],'All instance/parent joins require independent coordinator acceptance'],
        result_path=result_path,stop_conditions=['Input/base drift','Scope or interface unresolved','Oracle unavailable','Two unsuccessful corrections'],
        coordinator_binding=binding,source_hashes=source_hashes,context_budget_bytes=leaf['max_context_bytes'])
    # Caller writes context only after all checks above. Standard packet checker
    # then verifies hashes, commands/cwds, paths and final-success expectation.
    return packet,bundle


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=['check','show','prepare'])
    parser.add_argument('leaf',nargs='?');parser.add_argument('--binding',type=Path)
    args=parser.parse_args()
    try:
        catalog=tasks.derive(ROOT);recipes=json.loads((ROOT/RECIPES).read_text())
        errors=validate(ROOT,catalog,recipes)
        if errors: raise ValueError('; '.join(errors))
        if args.command=='check':
            print(f"PASS: {len(recipes['tasks'])} MVP parents; {sum(len(t['leaves']) for t in recipes['tasks'])} bounded leaves; complete joins and context/objective hashes")
        elif args.command=='show':
            for row in recipes['tasks']:
                for leaf in row['leaves']:
                    if leaf['id']==args.leaf:
                        print(json.dumps(dict(parent=row['task_id'],source=row['source'],context_sources=row['context_sources'],leaf=leaf,
                            reviewed_recipe_sha256=digest(json.dumps(leaf,sort_keys=True).encode())),indent=2));return 0
            raise ValueError('Unknown leaf')
        else:
            if not args.leaf or not args.binding: raise ValueError('Leaf and --binding required')
            packet,bundle=prepare(ROOT,catalog,recipes,args.leaf,json.loads(args.binding.read_text()))
            target=ROOT/packet['target_repo'];ctx=target/packet['context_paths'][0]
            ctx.parent.mkdir(parents=True,exist_ok=True);ctx.write_text(bundle)
            packet['input_hashes'][packet['context_paths'][0]]=digest(ctx.read_bytes())
            errors=tasks.packet_errors(ROOT,packet)
            if errors: raise ValueError('; '.join(errors))
            out=ROOT/'.artifacts/packets'/f"{packet['packet_id']}.json";out.parent.mkdir(parents=True,exist_ok=True)
            out.write_text(json.dumps(packet,indent=2)+'\n')
            print(f'Prepared {out}; coordinator reservation/dispatch required. No worker launched.')
        return 0
    except (ValueError,KeyError,OSError,StopIteration,subprocess.CalledProcessError) as exc:
        print(f'FAIL: {exc}');return 1

if __name__=='__main__': raise SystemExit(main())
