#!/usr/bin/env python3
"""Small read-only project context/check harness. No agents, installs or writes."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
REQUIRED = ('spec.md','plan.md','index.md','metadata.json','agent-contract.md',
            'risk-register.md','test-matrix.md','handoff.md')

def git(root, *args):
    run = subprocess.run(['git','-C',str(root),*args],capture_output=True,text=True,check=False)
    if run.returncode:
        raise ValueError(run.stderr.strip() or 'git command failed')
    return run.stdout.strip()

def validate(root, check_git=True):
    errors = []
    try:
        state = json.loads((root/'conductor/current-state.json').read_text())
        if not state.get('next_action') or not state.get('active_track'):
            errors.append('Current state needs an active_track and next_action')
        for path in state.get('context_paths',[]):
            if not (root/path).is_file(): errors.append(f'Missing context: {path}')
        if check_git:
            for path,expected in state['submodule_pins'].items():
                actual=git(root/path,'rev-parse','HEAD')
                if actual != expected: errors.append(f'Submodule drift: {path}: {actual} != {expected}')
    except (OSError,ValueError,KeyError) as error:
        errors.append(f'Current state invalid: {error}');state={}
    tracks={}
    for folder in sorted((root/'conductor/tracks').glob('*')):
        if not folder.is_dir():continue
        for name in REQUIRED:
            if not (folder/name).is_file():errors.append(f'Missing {folder.name}/{name}')
        try:
            metadata=json.loads((folder/'metadata.json').read_text())
            if metadata['track_id']!=folder.name:errors.append(f'ID mismatch: {folder.name}')
            if metadata['status'] not in ('proposed','in_progress','complete','blocked'):
                errors.append(f'Invalid status: {folder.name}')
            if metadata['status']=='complete':
                evidence=metadata.get('completion_evidence',[])
                if not evidence or any(not (root/p).is_file() for p in evidence):
                    errors.append(f'Completion evidence missing: {folder.name}')
            phases=set(re.findall(r'^## ([A-Z]\d+) —', (folder/'plan.md').read_text(),re.M))
            tracks[folder.name]=(metadata,phases)
        except (OSError,ValueError,KeyError) as error:
            errors.append(f'Invalid track {folder.name}: {error}')
    if state.get('active_track') not in tracks:
        errors.append('Active track is not registered')
    elif state.get('active_phase') not in tracks[state['active_track']][1]:
        errors.append('Active phase is not registered')
    edges={f'{name}:{phase}':set() for name,(_,phases) in tracks.items() for phase in phases}
    for name,(metadata,phases) in tracks.items():
        declared=metadata.get('phase_dependencies')
        if declared is None:
            ordered=sorted(phases,key=lambda x:int(x[1:]))
            declared={phase:ordered[i-1:i] for i,phase in enumerate(ordered)}
        if set(declared)!=phases:
            errors.append(f'Phase dependency coverage mismatch: {name}')
        for phase,prerequisites in declared.items():
            for prerequisite in prerequisites:
                origin=f'{name}:{phase}';target=f'{name}:{prerequisite}'
                if origin not in edges or target not in edges:
                    errors.append(f'Unknown dependency: {origin} -> {target}')
                else:edges[origin].add(target)
        for dependency in metadata.get('milestone_dependencies',[]):
            origin=f"{name}:{dependency['milestone']}";target=dependency['requires']
            if origin not in edges or target not in edges:errors.append(f'Unknown dependency: {origin} -> {target}')
            else:edges[origin].add(target)
    active=set();seen=set()
    def visit(node):
        if node in active:errors.append(f'Milestone cycle at {node}');return
        if node in seen:return
        active.add(node)
        for child in edges[node]:visit(child)
        active.remove(node);seen.add(node)
    for node in edges:visit(node)
    files=list((root/'conductor').rglob('*.md'))+[root/'AGENTS.md']
    links=0
    for path in files:
        if not path.exists():errors.append(f'Missing {path.name}');continue
        text=re.sub(r'```.*?```','',path.read_text(),flags=re.S)
        for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',text):
            if target.startswith(('https://','http://','#','mailto:')):continue
            links+=1
            if not (path.parent/target.split('#')[0]).exists():errors.append(f'Broken link in {path}: {target}')
    return {'tracks':len(tracks),'milestones':len(edges),'local_links':links,'errors':errors}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command',choices=('resume','check'))
    args=parser.parse_args()
    result=validate(ROOT)
    if args.command=='resume':
        state=json.loads((ROOT/'conductor/current-state.json').read_text())
        print(json.dumps({k:state[k] for k in ('active_track','active_phase','next_action','context_paths')},indent=2))
        print('Git changes:',git(ROOT,'status','--short') or 'clean')
    print(json.dumps(result,indent=2))
    return int(bool(result['errors']))

if __name__=='__main__':
    raise SystemExit(main())
