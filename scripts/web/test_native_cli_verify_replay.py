"""Real CLI State -> Verify integration; owned synthetic sources, never a mock child."""
from pathlib import Path
import argparse,datetime,hashlib,json,os,shutil,subprocess,tempfile

REPO=Path(__file__).resolve().parents[2]
sha=lambda b:hashlib.sha256(b).hexdigest()
def tree(root):
 return {p.relative_to(root).as_posix():None if p.is_dir()else sha(p.read_bytes()) for p in root.rglob('*') if p.is_file()or p.is_dir()}
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--native-executable',required=True);args=parser.parse_args();source=Path(args.native_executable).resolve();assert source.is_file()
 stage=REPO/'target/native-verify-replay';stage.mkdir(parents=True,exist_ok=True);root=Path(tempfile.mkdtemp(prefix='run-',dir=stage));native=root/('owned-bisect.exe' if os.name=='nt'else'owned-bisect');shutil.copyfile(source,native);native.chmod(0o755)
 status={'status':'running','native_source':str(source),'native_sha256':sha(native.read_bytes()),'root':str(root),'test_source_sha256':sha(Path(__file__).read_bytes()),'fixture_sources':{name:sha((REPO/name).read_bytes())for name in ['scripts/web/fixtures/native-workspace-case.mjs','scripts/web/fixtures/cffi-engine-case.mjs','data/manifest.json']},'checks':[],'scope':'Actual native CLI replay output-root regression on a four-unit synthetic RI one-district control. No browser/full backend/method/source/host/capacity/plan acceptance.'}
 def save(): (root/'evidence.json').write_text(json.dumps(status,indent=2)+'\n',encoding='utf-8')
 save()
 try:
  fixture=root/'fixture';fixture.mkdir();module=REPO/'scripts/web/fixtures/native-workspace-case.mjs'
  code="""const fs=await import('node:fs/promises'),path=await import('node:path'),{nativeWorkspaceCase}=await import(process.argv[2]);const root=path.resolve(process.argv[1]);for(const[name,bytes]of nativeWorkspaceCase().files){const file=path.resolve(root,'inputs',name);if(!file.startsWith(root+path.sep))throw Error('Escaped fixture');await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,bytes);}"""
  prep=subprocess.run(['node','--input-type=module','--eval',code,str(fixture),module.as_uri()],cwd=root,capture_output=True,timeout=60);(root/'fixture-stdout.txt').write_bytes(prep.stdout);(root/'fixture-stderr.txt').write_bytes(prep.stderr);assert prep.returncode==0,prep.stderr
  data=json.loads((REPO/'data/manifest.json').read_text(encoding='utf-8-sig'));data['states']={'RI':data['states']['RI']};data['local_outputs_dir']=str(fixture/'inputs');data['local_data_dir']=str(fixture/'data');(fixture/'data').mkdir();manifest=fixture/'data-manifest.json';manifest.write_text(json.dumps(data),encoding='utf-8');env={**os.environ,'BISECT_MANIFEST':str(manifest)}
  def run(label,argv,cwd):
   result=subprocess.run([str(native),*argv],cwd=cwd,env=env,capture_output=True,timeout=120);(root/(label+'-stdout.txt')).write_bytes(result.stdout);(root/(label+'-stderr.txt')).write_bytes(result.stderr);(root/(label+'-invocation.json')).write_text(json.dumps({'argv':[str(native),*argv],'cwd':str(cwd),'exit_code':result.returncode},indent=2)+'\n',encoding='utf-8');return result
  baseline=root/'baseline';baseline.mkdir();base_output=baseline/'outputs';state_args=['state','--state','RI','--year','2020','--chamber','congressional','--districts','1','--label','ReplayControl','--version','v1','--output-dir',str(base_output/'v1'),'--adjacency',str(fixture/'inputs/V3/data/2020/adjacency/ri_adjacency_2020.adj.bin'),'--metis-engine','metis-core','--search','single','--seed','42','--balance-tolerance','0.5','--force'];created=run('state-original',state_args,baseline);assert created.returncode==0,created.stderr
  relative=Path('v1/2020/plans/ReplayControl');original=base_output/relative;assert (original/'manifest.json').is_file();assert len(json.loads((original/'data/final_assignments.json').read_text()))==4
  source_before=tree(fixture);baseline_before=tree(base_output)
  for name in ['exact-replay','changed-reference','invalid-certificate']:
   case=root/(name+' folder \u03a9');case.mkdir();output=case/'outputs';shutil.copytree(base_output,output);plan=output/relative
   if name=='changed-reference':
    path=plan/'data/final_assignments.json';assignments=json.loads(path.read_text());assignments[next(iter(assignments))]=99;path.write_text(json.dumps(assignments),encoding='utf-8')
   if name=='invalid-certificate':
    path=plan/'audit-certificate.json';assert path.is_file();path.write_bytes(path.read_bytes()+b' ')
   before=tree(output);label='Replay_'+name;argv=['verify','--manifest',str(plan/'manifest.json'),'--output-base',str(output),'--verify-label',label,'--min-similarity','1','--skip-binary-check'];result=run(name,argv,case);after=tree(output);assert all(p in after and after[p]==h for p,h in before.items()),'Original input modified'
   expected=output/'verify/2020/plans'/label/'data/final_assignments.json';wrong=output/'2020/plans'/label
   if name=='exact-replay':
    assert result.returncode==0,result.stderr.decode(errors='replace');assert b'[PASS] Verification succeeded' in result.stderr;assert expected.is_file() and not wrong.exists();assert json.loads(expected.read_text())==json.loads((plan/'data/final_assignments.json').read_text())
   elif name=='changed-reference':
    assert result.returncode!=0 and b'Jaccard similarity' in result.stderr and expected.is_file();assert not wrong.exists()
   else:
    assert result.returncode!=0 and after==before and b'Running re-verification' not in result.stderr
   status['checks'].append({'id':name,'status':'passed','exit_code':result.returncode,'original_files_unchanged':True,'expected_versioned_replay_exists':expected.is_file(),'unexpected_unversioned_replay_exists':wrong.exists()});save()
  assert tree(fixture)==source_before and tree(base_output)==baseline_before;status['status']='passed';status['original_fixture_unchanged']=True;status['baseline_native_output_unchanged']=True;save();print('PASS actual CLI Verify replay, mismatch and precheck effects: '+str(root));print(json.dumps({'native_verify_evidence_root':str(root)}))
 except Exception as error:
  status['status']='failed';status['error']=str(error);save();print('Retained failed CLI Verify test: '+str(root));raise
if __name__=='__main__':main()
