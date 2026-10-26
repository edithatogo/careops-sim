"""Source selection gate only. Does not qualify runtime, API release, or a pin."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

PIN = '44261eb5cf8275f132f63a3396903d116d701dbe'
OWNERS = '55b20e77eb9ca0df5a95b9e82c4fb815d3919a87'
TRACKS = {
 '03':'03-flow-des-trajectory-abm-behavior', '04':'04-analyst-kairo-ecs-arrow',
 '21':'21-verification-validation-uncertainty', '22':'22-experiment-runner-scenario-management',
 '25':'25-api-design-review-compatibility-governance', '32':'32-gpu-compute-acceleration'}
CONSUMED = ('kairo-ecs-types','kairo-ecs-core','kairo-ecs-state','kairo-ecs-rng',
 'kairo-ecs-des','kairo-ecs-abm','kairo-ecs-arrow','kairo-ecs-arrow-io',
 'kairo-ecs-calibration','kairo-ecs-cli')

def inspect(repo, candidate, inventory):
 def git(*args, expected=0):
  p = subprocess.run(['git','-C',str(repo),*args],capture_output=True)
  if p.returncode != expected:
   raise RuntimeError(p.stderr.decode())
  return p.stdout
 if not re.fullmatch('[0-9a-f]{40}',candidate):
  raise ValueError('Immutable full commit required')
 if inventory['revisions'] != {'pinned':PIN,'candidate':candidate}:
  raise ValueError('Inventory revision mismatch')
 git('cat-file','-e',candidate+'^{commit}')
 changed=git('diff','--name-only',PIN,candidate).decode().splitlines()
 consumed=[p for p in changed if any(p.startswith('crates/'+c+'/') for c in CONSUMED)]
 manifests=[p for p in changed if p.endswith('/Cargo.toml') or p in ('Cargo.toml','Cargo.lock','rust-toolchain.toml')]
 old_availability=git('show',PIN+':crates/kairo-ecs-gpu/src/compute.rs')
 new_availability=git('show',candidate+':crates/kairo-ecs-gpu/src/compute.rs')
 ancestor=subprocess.run(['git','-C',str(repo),'merge-base','--is-ancestor',PIN,candidate]).returncode
 if ancestor not in (0,1):
  raise RuntimeError('Ancestry check unavailable')
 owner_bindings=[]
 registry=git('show',OWNERS+':conductor/tracks.yaml').decode()
 for number,directory in TRACKS.items():
  block=re.search(r'(?ms)^  - id: '+number+r'\n.*?(?=^  - id:|\Z)',registry)
  if block is None:
   raise ValueError('Owner registry record missing: '+number)
  owner=re.search(r'^    owner: (.+)$',block[0],re.M)[1]
  for name in ('spec.md','plan.md'):
   path=f'conductor/tracks/{directory}/{name}'
   data=git('show',OWNERS+':'+path)
   owner_bindings.append({'track':number,'owner':owner,'revision':OWNERS,'path':path,
    'sha256':hashlib.sha256(data).hexdigest()})
 for path in ('conductor/contracts/versioning-compatibility.md','docs/design/protected-surface-inventory.json'):
  data=git('show',OWNERS+':'+path)
  owner_bindings.append({'track':'25','revision':OWNERS,'path':path,'sha256':hashlib.sha256(data).hexdigest()})
 s=inventory['summary']; reasons=[]
 if s['pinned_hash_mismatches']:reasons.append('baseline-source-hash-mismatch')
 if s['candidate_missing'] or s['candidate_changed']:reasons.append('reviewed-consumer-source-differs')
 if consumed:reasons.append('consumed-crate-source-differs')
 if manifests:reasons.append('dependency-feature-toolchain-review-required')
 if ancestor:reasons.append('divergent-history-needs-integration')
 if old_availability != new_availability:reasons.append('GPU-availability-contract-review-required')
 if any(not p.startswith('crates/kairo-ecs-gpu/') for p in changed):reasons.append('outside-bounded-optional-GPU-delta')
 return {'schema_version':1,'pin':PIN,'candidate':candidate,'pin_is_ancestor':ancestor==0,
  'scope':'source candidate for R2.2/R2.3/R3 qualification only; no pin advancement',
  'verdict':'hold_for_evidence' if reasons else 'candidate_for_qualification',
  'reasons':reasons,'changed_paths':changed,'consumed_crate_changes':consumed,
  'manifest_lock_toolchain_changes':manifests,'GPU_availability_source_unchanged':old_availability==new_availability,
  'reviewed_source_summary':s,'owner_bindings':owner_bindings,
  'pending_gates':['independent owner review of selection','R2.2 consumer API and independent format qualification',
   'R2.3 required native tests and any consumed device/host qualification',
   'R2.4 phase review','R3 baseline comparison, rollback rehearsal and pin decision',
   'Track25 optional GPU public additions review before release']}

def main():
 p=argparse.ArgumentParser(description=__doc__)
 p.add_argument('--repo',type=Path,required=True);p.add_argument('--candidate',required=True)
 p.add_argument('--inventory',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
 a=p.parse_args();r=inspect(a.repo,a.candidate,json.loads(a.inventory.read_text()))
 a.output.write_text(json.dumps(r,indent=2)+'\n')
 print(json.dumps({'verdict':r['verdict'],'reasons':r['reasons'],'changed_paths':r['changed_paths']},indent=2))
 return 2 if r['reasons'] else 0
if __name__=='__main__':raise SystemExit(main())
