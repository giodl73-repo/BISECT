"""Predeclared actual CLI counterfactuals for all 22 source-traced unused fields."""
import datetime,hashlib,json,os,pathlib,shutil,subprocess,sys
import argparse
import zipfile
from native_cli_state_observables import checked_observables
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo-root',type=pathlib.Path,default=pathlib.Path.cwd())
parser.add_argument('--native-executable',type=pathlib.Path)
parser.add_argument('--wasm',type=pathlib.Path)
parser.add_argument('--output-root',type=pathlib.Path)
args=parser.parse_args();ROOT=args.repo_root.resolve();SCRIPTS=pathlib.Path(__file__).resolve().parent
native=(args.native_executable or ROOT/('target/release/bisect.exe' if os.name=='nt' else 'target/release/bisect')).resolve()
wasm=(args.wasm or ROOT/'target/wasm32-unknown-unknown/release/bisect_wasm.wasm').resolve()
assert native.is_file() and wasm.is_file(),'Build or explicitly select native CLI and WASM.'
OUT=(args.output_root or ROOT/'target/native-cli-unused'/datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ')).resolve();OUT.mkdir(parents=True,exist_ok=False)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sys.path.insert(0,str(ROOT/'scripts/web'));from wasm_native_consumption import FAMILIES
variants={
 'run':{'dpi':['--dpi','72'],'election_year':['--election-year','2016'],'experiment_name':['--experiment-name','unused-lab'],
  'minimum_boundary_length':['--minimum-boundary-length','999'],'print_only':['--print-only'],
  'processing_mode':['--processing-mode','batch'],'run_type':['--run-type','test'],
  'skip_analysis':['--skip-analysis'],'skip_demographic':['--skip-demographic'],'skip_political':['--skip-political'],'stages':['--stages','nation']},
 'state':{'dpi':['--dpi','72'],'objtype':['--objtype','vol'],'print_only':['--print-only'],'target_mm_districts':['--target-mm-districts','1']},
 'states':{'dpi':['--dpi','72'],'election_year':['--election-year','2016'],'print_only':['--print-only'],
  'processing_mode':['--processing-mode','batch'],'run_analysis':['--run-analysis'],
  'skip_demographic':['--skip-demographic'],'skip_political':['--skip-political']}}
for family,fields in variants.items():assert set(fields)==set(FAMILIES['cli.'+family][3])
prepared=subprocess.run(['node',str(SCRIPTS/'native_cli_workspace_fixture.mjs'),str(OUT),str(ROOT)],cwd=ROOT,capture_output=True,check=True)
(OUT/'fixture-generation.log').write_bytes(prepared.stdout+prepared.stderr);fixture=json.loads((OUT/'fixture.json').read_text(encoding='utf-8'))
owned=OUT/'owned';owned.mkdir()
artifacts={}
for original,name in [(native,'native-bisect.exe' if os.name=='nt' else 'native-bisect'),(wasm,'engine.wasm')]:
    shutil.copyfile(original,owned/name);assert sha(original)==sha(owned/name)
    if os.name!='nt':shutil.copymode(original,owned/name)
    artifacts[name]={'original':str(original),'owned':str(owned/name),'sha256':sha(owned/name)}
declaration={'status':'declared-before-execution','fixture_sha256':sha(OUT/'fixture.json'),'variants':variants,
 'algorithm':'native default edge-weighted; no budget/tolerance changes','units':4,'population_per_unit':100,'districts':2,
 'state_seed':'18446744073709551615','invocation_counts':{'original_counterfactual_commands':25,'additional_read_only_manifest_checks':5,'total_native_cli_processes':30},'run_states_seed':'native default, no seed flag exists in those argument types',
 'controlled_difference':'Fresh output root and exact recorded working directory per invocation are verified. Only that cwd is projected away after native typed recipe and independent context hash checks; all other recipe, context, plan and effective options must match. Only one unused field differs in baseline args.',
 'scope':'One synthetic four-unit tract graph/current native image, explicit metis-core State/default Run/States. Actual CLI flag counterfactuals and WASM jobs; not real-data/all-method/C-partition/actual-browser/full acceptance.','unused_field_source_sha256':sha(ROOT/'scripts/web/wasm_native_consumption.py'),'test_sources':{p.name:sha(p) for p in [pathlib.Path(__file__),SCRIPTS/'test_native_cli_unused_jobs.mjs',SCRIPTS/'native_cli_workspace_fixture.mjs',SCRIPTS/'native_cli_state_observables.py']}}
(OUT/'declaration.json').write_text(json.dumps(declaration,indent=2)+'\n',encoding='utf-8')
records=[];status={'status':'running','declaration_sha256':sha(OUT/'declaration.json'),'artifacts':artifacts,'cases':records,'native_manifest_dry_run_checks':0}
def save():(OUT/'status.json').write_text(json.dumps(status,indent=2)+'\n',encoding='utf-8')
save()
for family,fields in variants.items():
    baseline=None
    for label,extra in [('baseline',[]),*fields.items()]:
        case=OUT/(family+'-'+label);case.mkdir();output=case/'outputs'/'vUnused'
        if family=='state':arguments=['state','--state','RI','--year','2020','--districts','2','--metis-engine','metis-core',
          '--seed','18446744073709551615','--adjacency',str(pathlib.Path(fixture['input'])/(fixture['stem']+'.adj.bin')),
          '--version','vUnused','--output-dir',str(output),'--manifest']
        else:arguments=[family,'--year','2020','--states','RI','--workers','1','--version','vUnused','--output-dir',str(output)]
        arguments+=extra
        (case/'arguments.json').write_text(json.dumps(arguments,indent=2)+'\n',encoding='utf-8')
        result=subprocess.run([str(owned/('native-bisect.exe' if os.name=='nt' else 'native-bisect')),*arguments],cwd=case,
          env={**os.environ,'BISECT_MANIFEST':fixture['manifest'],'RAYON_NUM_THREADS':'1'},capture_output=True,timeout=60)
        (case/'stdout.log').write_bytes(result.stdout);(case/'stderr.log').write_bytes(result.stderr)
        files={p.relative_to(output).as_posix():{'sha256':sha(p),'byte_length':p.stat().st_size} for p in output.rglob('*') if p.is_file()} if output.exists() else {}
        assignmentPaths=[name for name in files if name.endswith('/data/final_assignments.json')]
        values=json.loads((output/assignmentPaths[0]).read_text(encoding='utf-8')) if len(assignmentPaths)==1 else None
        row={'case':family+'-'+label,'family':family,'changed_field':None if label=='baseline' else label,'arguments':arguments,
          'output_root':str(output),'exit_code':result.returncode,'assignments':values,'files':files,
          'stdout_sha256':sha(case/'stdout.log'),'stderr_sha256':sha(case/'stderr.log')}
        records.append(row);save();print(json.dumps({'case':row['case'],'exit_code':result.returncode,'output_files':len(files)}),flush=True)
        assert result.returncode==0,(row['case'],result.stderr[-2500:])
        assert values and set(values)=={'0','1','2','3'} and sorted(set(values.values()))==[1,2]
        for label in [1,2]:
            nodes=sorted(int(k) for k,v in values.items() if v==label);assert len(nodes)==2 and nodes[1]-nodes[0]==1,(row['case'],'population/connectivity mismatch in four-unit chain')
        if family=='state':
            parent=(output/assignmentPaths[0]).parent.parent
            plan=json.loads((parent/'plan.rplan').read_text(encoding='utf-8'));context=json.loads((parent/'context.rctx').read_text(encoding='utf-8'));native_manifest=json.loads((parent/'manifest.json').read_text(encoding='utf-8'))
            assert plan['plan']['units']['unit_ids']==context['units']['unit_ids']==fixture['ids'] and context['populations']==[100]*4
            assert [a+1 for a in plan['plan']['assignment']]==[values[str(i)] for i in range(4)] and plan['plan']['k']==2
            verification=subprocess.run([str(owned/('native-bisect.exe' if os.name=='nt' else 'native-bisect')),'verify','--manifest',str(parent/'manifest.json'),'--dry-run'],cwd=case,capture_output=True,timeout=60)
            (case/'manifest-verify.stdout.log').write_bytes(verification.stdout);(case/'manifest-verify.stderr.log').write_bytes(verification.stderr)
            assert verification.returncode==0,(row['case'],'Native typed recipe/context binding verification failed',verification.stderr[-2500:])
            status['native_manifest_dry_run_checks']+=1
            row['manifest_dry_run']={'exit_code':0,'partition_execution':False,'stdout_sha256':sha(case/'manifest-verify.stdout.log'),'stderr_sha256':sha(case/'manifest-verify.stderr.log')}
            row['native_state_observables']=checked_observables(plan,context,native_manifest,case,declaration['state_seed'])
        if baseline is None:baseline=row
        else:
            assert row['assignments']==baseline['assignments'],row['case']
            if family=='state':assert row['native_state_observables']==baseline['native_state_observables'],(row['case'],'State plan/context/effective options changed')

for name,item in artifacts.items():assert sha(owned/name)==item['sha256']
status['status']='native-cli-passed';status['actual_cli_invocations']=len(records);status['unused_field_variants']=sum(len(v) for v in variants.values());save()
assert len(records)==25 and status['unused_field_variants']==22 and status['native_manifest_dry_run_checks']==5
print(json.dumps({'status':'native-cli-passed','actual_cli_invocations':25,'variants':22,'directory':str(OUT)}))

# Same-input Rust/WASM jobs exercise selected sources, exact original flags,
# explicit print-only adaptation, and source-preserving projects/exports.
comparison=subprocess.run(['node',str(SCRIPTS/'test_native_cli_unused_jobs.mjs'),str(OUT),str(ROOT)],cwd=ROOT,capture_output=True)
(OUT/'wasm-jobs.stdout.log').write_bytes(comparison.stdout);(OUT/'wasm-jobs.stderr.log').write_bytes(comparison.stderr)
if comparison.returncode:
 (OUT/'gate-status.json').write_text(json.dumps({'status':'failed','stage':'wasm-jobs','exit_code':comparison.returncode}),encoding='utf-8')
 raise SystemExit(comparison.returncode)
assert sha(native)==next(v['sha256'] for k,v in artifacts.items() if k!='engine.wasm') and sha(wasm)==artifacts['engine.wasm']['sha256']
evidence=OUT/'wasm-cli-execution/evidence.json';reply=json.loads(evidence.read_text(encoding='utf-8'));assert reply['status']=='passed' and reply['wasm_execution_matches']==22 and reply['print_only_adaptations']==3
zip_audits=[]
for case in status['cases']:
    d=OUT/'wasm-cli-execution'/case['case'];raw=json.loads((d/'raw.json').read_text(encoding='utf-8'))
    assert json.loads(raw['original_command']['arguments_json'])==case['arguments'][1:]
    if case['changed_field']=='print_only':continue
    result=raw['result'];project=json.loads((d/'result.bisect').read_text(encoding='utf-8'));assert project['result']==result
    expected={id:case['assignments'][str(i)] for i,id in enumerate(fixture['ids'])};assert result['prepared']['result']['assignments']==expected
    with zipfile.ZipFile(d/'result.zip') as archive:
        assert archive.testzip() is None
        assert json.loads(archive.read('native-state/data/final_assignments.json'))==case['assignments']
        plan=json.loads(archive.read('plan.rplan'));context=json.loads(archive.read('context.rctx'))
        assert plan['plan']['units']['unit_ids']==context['units']['unit_ids']==fixture['ids'];assert context['populations']==[100]*4
        assert [a+1 for a in plan['plan']['assignment']]==list(expected.values())
        assert archive.read('plan.rplan').decode('utf-8')==result['prepared']['export']['document_json']
        assert archive.read('context.rctx').decode('utf-8')==result['prepared']['export']['context_json']
        if case['family']=='state':
            assert json.loads(archive.read('state-flags/original-workflow-arguments.json'))==case['arguments'][1:]
            assert project['files']['nativeAdjacencyInputs']==raw['job']['native_state']['source_inputs']['preparation']['input']['sources']
            assert result['prepared']['result']['build_report']['attempts'][-1]['seed']=='18446744073709551615'
        else:
            prefix=case['family']+'-flags/'
            assert json.loads(archive.read(prefix+'arguments.json'))==case['arguments'][1:]
            assert project['files']['stateGraphInputs']==result['native_'+case['family']]['recipe']['graph_inputs']
    zip_audits.append({'case':case['case'],'zip_sha256':sha(d/'result.zip'),'project_sha256':sha(d/'result.bisect'),'exact_cli_assignments':True,'source_recipe_retained':True})
assert len(zip_audits)==22
(OUT/'export-readback.json').write_text(json.dumps({'status':'passed','scope':'Independent Python ZIP and saved-project readback; assignments/unit/population and source/argument bindings, not complete native/browser context equality.','cases':zip_audits},indent=2),encoding='utf-8')
(OUT/'gate-status.json').write_text(json.dumps({'status':'passed','native_cli_invocations':25,'native_manifest_dry_run_checks':5,'total_cli_process_invocations':30,'unused_field_pairs':22,'wasm_execution_matches':22,'browser_intent_adaptations':3,'native_status_sha256':sha(OUT/'status.json'),'wasm_evidence_sha256':sha(evidence),'export_readback_sha256':sha(OUT/'export-readback.json'),'artifacts':artifacts},indent=2),encoding='utf-8')
print('PASS actual native CLI unused fields: all 22 declared variants, 25 native runs, 22 matching WASM executions and three print-only intent adaptations; '+str(OUT))
