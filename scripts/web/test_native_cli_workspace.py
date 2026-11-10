"""Actual CLI workspace branch witnesses; all writes/reset targets owned and snapshotted."""
from pathlib import Path
import datetime,json,hashlib,shutil,subprocess,os
import argparse
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo-root',type=Path,default=Path.cwd())
parser.add_argument('--native-executable',type=Path)
parser.add_argument('--wasm',type=Path)
parser.add_argument('--output-root',type=Path)
options=parser.parse_args();repo=options.repo_root.resolve();scripts=Path(__file__).resolve().parent
native=(options.native_executable or repo/('target/release/bisect.exe' if os.name=='nt' else 'target/release/bisect')).resolve()
wasm=(options.wasm or repo/'target/wasm32-unknown-unknown/release/bisect_wasm.wasm').resolve()
assert native.is_file() and wasm.is_file(),'Build or explicitly select the native CLI and WASM before this gate.'
out=(options.output_root or repo/'target/native-cli-workspace'/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')).resolve()
out.mkdir(parents=True,exist_ok=False);owned=out/'owned';owned.mkdir();exe=owned/('bisect.exe' if os.name=='nt' else 'bisect');shutil.copy2(native,exe);shutil.copyfile(wasm,owned/'engine.wasm')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();load=lambda p:json.loads(p.read_text(encoding='utf-8'))
fixture_run=subprocess.run(['node',str(scripts/'native_cli_workspace_fixture.mjs'),str(out),str(repo)],capture_output=True)
(out/'fixture-generation.stdout.log').write_bytes(fixture_run.stdout);(out/'fixture-generation.stderr.log').write_bytes(fixture_run.stderr);assert fixture_run.returncode==0,'Fixture generation failed; retained logs identify the failure.'
inputs=out/'inputs'
# Retain synthetic source bytes for each declared vintage, never imply real vintage data.
for year in ['2010','2000']:
 for p in (inputs/'V3/data/2020/adjacency').iterdir():
  dest=inputs/'V3/data'/year/'adjacency'/p.name.replace('2020',year);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest)
specs=[]
def add(name,family,setup,extra,expected,execution,year='2020',states='RI'):specs.append(dict(name=name,family=family,setup=setup,extra=extra,expected_exit=expected,expected_execution=execution,year=year,states=states))
add('run-fresh','run','fresh',[],0,True)
for name,extra,executing in [('complete',[],False),('complete-reset',['--reset'],False),('complete-reprocess',['--reprocess'],True),('complete-reset-reprocess',['--reset','--reprocess'],True)]:add('run-'+name,'run','complete',extra,0,executing)
for name,extra,executing in [('complete',[],False),('complete-reprocess',['--reprocess'],True)]:add('states-'+name,'states','complete',extra,0,executing)
add('state-label-fresh','state','fresh',[],0,True)
for name,extra,expected,execution in [('collision',[],1,False),('collision-reset',['--reset'],1,False),('force',['--force'],0,True),('force-reset',['--force','--reset'],0,True),('incomplete-force-reset',['--force','--reset'],1,False)]:add('state-'+name,'state','incomplete' if name.startswith('incomplete') else 'label-complete',extra,expected,execution)
add('run-incomplete-reset-reprocess','run','incomplete',['--reset','--reprocess'],1,False)
add('states-incomplete-reprocess','states','incomplete',['--reprocess'],1,False)
add('run-historical-marker','run','historical',[],0,False)
add('run-all-vintages','run','fresh',[],0,True,year='all')
add('run-unmatched-filter','run','fresh',[],0,False,states='ZZ')
sourcefiles=['crates/bisect-cli/src/main.rs','crates/bisect-cli/src/runner/mod.rs','crates/bisect-cli/src/runner/support.rs','crates/bisect-core/src/state_catalog.rs','crates/bisect-report/src/run_target.rs']
(out/'declaration.json').write_text(json.dumps({'status':'declared-before-execution','scope':'18 actual current native CLI orchestration cases on an owned four-unit synthetic RI fixture, including synthetic 2000/2010/2020 copies. No real-vintage capacity, browser/selected-folder host or C-profile claim. Native reset acts only inside checked case outputs.','native_cli_sha256':sha(exe),'wasm_sha256':sha(owned/'engine.wasm'),'test_sources':{p.name:sha(p) for p in [Path(__file__),scripts/'native_cli_workspace_fixture.mjs',scripts/'test_native_cli_workspace_decisions.mjs']},'fixture_sha256':sha(out/'fixture.json'),'source_pins':{f:sha(repo/f) for f in sourcefiles},'cases':specs},indent=2))
# Generate every seed baseline from this exact executable and newly generated fixture.
base={'cases':[]}
for family in ['run','state','states']:
 baseline=out/('seed-'+family);baseline.mkdir();output=baseline/'outputs/vSeed'
 if family=='state':args=['state','--state','RI','--year','2020','--districts','2','--metis-engine','metis-core','--seed','18446744073709551615','--adjacency',str(inputs/'V3/data/2020/adjacency/ri_adjacency_2020.adj.bin'),'--manifest','--version','vSeed','--output-dir',str(output)]
 else:args=[family,'--year','2020','--states','RI','--workers','1','--version','vSeed','--output-dir',str(output)]
 result=subprocess.run([str(exe),*args],cwd=baseline,env={**os.environ,'BISECT_MANIFEST':str(inputs/'manifest.json'),'RAYON_NUM_THREADS':'1'},capture_output=True,timeout=60)
 (baseline/'arguments.json').write_text(json.dumps(args,indent=2),encoding='utf-8');(baseline/'stdout.log').write_bytes(result.stdout);(baseline/'stderr.log').write_bytes(result.stderr)
 assert result.returncode==0,(family,'seed baseline failed')
 base['cases'].append({'case':family+'-baseline','output_root':str(output),'exit_code':result.returncode,'arguments':args})
(out/'seed-baselines.json').write_text(json.dumps(base,indent=2),encoding='utf-8')
rows=[]
def snapshot(root,dest):
 dest.mkdir();files={}
 for p in root.rglob('*'):
  if p.is_file():
   rel=p.relative_to(root);q=dest/rel;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);files[rel.as_posix()]={'sha256':sha(p),'bytes':p.stat().st_size,'mtime_ns':str(p.stat().st_mtime_ns)}
 return files
for spec in specs:
 case=out/spec['name'];case.mkdir();output=case/'outputs/vBranches';output.mkdir(parents=True);assert output.resolve().is_relative_to(case.resolve()) and case.resolve().is_relative_to(out.resolve())
 root=output/'2020'/('plans/Workspace_RI' if spec['family']=='state' else 'states/rhode_island');data=root/'data'
 if spec['setup'] in ['complete','label-complete','incomplete']:
  family='state' if spec['family']=='state' else spec['family'];priorcase=next(c for c in base['cases'] if c['case']==family+'-baseline');old=Path(priorcase['output_root'])/'2020/states/rhode_island';shutil.copytree(old,root)
  if spec['family']=='state':
   m=load(root/'manifest.json');m['label']='Workspace_RI';(root/'manifest.json').write_text(json.dumps(m))
  if spec['setup']=='incomplete':(root/'manifest.tmp').write_bytes(b'original incomplete witness; preserve')
 if spec['setup']=='historical':
  p=output/'2020/states/ri/data/final_assignments.pkl';p.parent.mkdir(parents=True);p.write_bytes(b'presence-only historical marker; never deserialize')
 data.mkdir(parents=True,exist_ok=True);(data/'sentinel.txt').write_bytes(b'data sentinel');(root/'sibling.txt').write_bytes(b'sibling sentinel')
 before=snapshot(output,case/'before')
 if spec['family']=='state':args=['state','--state','RI','--year','2020','--districts','2','--metis-engine','metis-core','--seed','18446744073709551615','--adjacency',str(inputs/'V3/data/2020/adjacency/ri_adjacency_2020.adj.bin'),'--label','Workspace_RI','--manifest','--version','vBranches','--output-dir',str(output)]
 else:args=[spec['family'],'--year',spec['year'],'--states',spec['states'],'--workers','1','--version','vBranches','--output-dir',str(output)]
 args+=spec['extra'];(case/'arguments.json').write_text(json.dumps(args,indent=2));result=subprocess.run([str(exe),*args],cwd=case,env={**os.environ,'BISECT_MANIFEST':str(inputs/'manifest.json'),'RAYON_NUM_THREADS':'1'},capture_output=True,timeout=60);(case/'stdout.log').write_bytes(result.stdout);(case/'stderr.log').write_bytes(result.stderr);after=snapshot(output,case/'after');executed=b'loading adjacency' in result.stdout+result.stderr
 row={**spec,'arguments':args,'exit_code':result.returncode,'execution_observed':executed,'before':before,'after':after,'stdout_sha256':sha(case/'stdout.log'),'stderr_sha256':sha(case/'stderr.log')};rows.append(row);(out/'status.json').write_text(json.dumps({'status':'running','cases':rows},indent=2));print(json.dumps({k:row[k] for k in ['name','exit_code','execution_observed']}),flush=True)
 assert result.returncode==spec['expected_exit'] and executed==spec['expected_execution'],spec['name']
 if not executed:assert before==after,(spec['name'],'unexpected mutation')
 assert (root/'sibling.txt').read_bytes()==b'sibling sentinel'
 reset_applied=executed and '--reset' in args;assert (data/'sentinel.txt').exists()!=reset_applied
 if spec['expected_exit']==1:assert b'Incomplete plan' in result.stderr or b'already exists' in result.stderr
 if executed:
  generated=list(output.rglob('final_assignments.json'));assert len(generated)==(3 if spec['year']=='all' else 1),(spec['name'],'missing accepted assignment outputs')
  audits=[]
  for artifact in generated:
   assignments=load(artifact);assert set(assignments)=={'0','1','2','3'} and all(type(v) is int for v in assignments.values()) and sorted(assignments.values())==[1,1,2,2],spec['name']
   for label in [1,2]:
    nodes=sorted(int(k) for k,v in assignments.items() if v==label);assert nodes[1]-nodes[0]==1,(spec['name'],'disconnected synthetic chain district')
   rel=artifact.relative_to(output).as_posix();old=row['before'].get(rel);assert old is None or str(artifact.stat().st_mtime_ns)!=old['mtime_ns'],(spec['name'],'scheduled assignment output was not freshly written')
   if spec['family']=='state':
    plan=load(artifact.parent.parent/'plan.rplan');context=load(artifact.parent.parent/'context.rctx');assert plan['plan']['units']['unit_ids']==context['units']['unit_ids']==load(out/'fixture.json')['ids'];assert context['populations']==[100]*4;assert [v+1 for v in plan['plan']['assignment']]==[assignments[str(i)] for i in range(4)];assert plan['plan']['k']==2
   audits.append({'path':rel,'sha256':sha(artifact),'population_per_district':200,'connected_chain_districts':True,'fresh_write_observed':True})
  row['accepted_output_audits']=audits

 if spec['name']=='run-all-vintages':
  logs=result.stderr.decode();assert [logs.index('year='+y) for y in ['2020','2010','2000']]==sorted(logs.index('year='+y) for y in ['2020','2010','2000']);assert len(list(output.rglob('final_assignments.json')))==3
assert len(rows)==18 and sha(exe)==sha(native)
(out/'status.json').write_text(json.dumps({'status':'passed','cases':rows,'actual_cli_invocations':len(rows)+len(base['cases'])},indent=2));print(json.dumps({'status':'passed','directory':str(out),'cases':len(rows)}))

# Retain raw WASM decisions and a terminal status; failure never becomes a passing gate.
decision=subprocess.run(['node',str(scripts/'test_native_cli_workspace_decisions.mjs'),str(out),str(repo)],capture_output=True)
(out/'decision.stdout.log').write_bytes(decision.stdout);(out/'decision.stderr.log').write_bytes(decision.stderr)
if decision.returncode:
 (out/'gate-status.json').write_text(json.dumps({'status':'failed','stage':'wasm-decisions','exit_code':decision.returncode}),encoding='utf-8')
 raise SystemExit(decision.returncode)
assert sha(owned/'engine.wasm')==sha(wasm) and sha(exe)==sha(native),'Input binary changed during gate.'
(out/'gate-status.json').write_text(json.dumps({'status':'passed','actual_cli_invocations':21,'branch_cases':18,'native_cli_sha256':sha(exe),'wasm_sha256':sha(owned/'engine.wasm'),'native_status_sha256':sha(out/'status.json'),'wasm_decision_evidence_sha256':sha(out/'wasm-decision-comparison/evidence.json')},indent=2),encoding='utf-8')
print('PASS actual CLI/WASM workspace regression: 3 fresh baselines, 18 native branch cases, matching WASM decisions; retained raw evidence at '+str(out))
