import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';

const argv=process.argv.slice(2),arg=(name,required=true)=>{const i=argv.indexOf(name);assert(i>=0||!required,'Supply '+name);return i<0?null:path.resolve(argv[i+1]);};
const sourceRoot=arg('--source-root',false)??fileURLToPath(new URL('../../',import.meta.url));
const wasm=arg('--wasm'),nativeCli=arg('--native-cli'),parent=arg('--output'),sideRoot=arg('--c-sidecar',false);
await fs.mkdir(parent,{recursive:true});const root=await fs.mkdtemp(path.join(parent,sideRoot?'c-':'core-'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),read=async p=>JSON.parse(await fs.readFile(p,'utf8'));
const corpusBytes=await fs.readFile(new URL('./fixtures/fresh-native-backend-profile.json',import.meta.url));
assert.equal(sha(corpusBytes),'96cafcd30f7004914347ddb70d2d1b023e6a7d2e66d85ff11dfc5360bf4c371a');
const corpus=JSON.parse(corpusBytes);assert.equal(corpus.schema_version,'bisect-fresh-native-backend-corpus-v1');assert.equal(corpus.rows.length,48);
const {instantiateEngine}=await import(pathToFileURL(path.join(sourceRoot,'web/lab/wasm-engine.js')));
const {loadMetisSidecar}=await import(pathToFileURL(path.join(sourceRoot,'web/lab/metis-c-sidecar.js')));
const bytes=await fs.readFile(wasm),bare=await instantiateEngine(bytes);let loaded=bare;
if(sideRoot){const manifest=await read(path.join(sideRoot,'metis-sidecar.json'));const metisSidecar=await loadMetisSidecar({manifest,javascript:await fs.readFile(path.join(sideRoot,manifest.javascript.file)),wasm:await fs.readFile(path.join(sideRoot,manifest.wasm.file))});loaded=await instantiateEngine(bytes,{metisSidecar});}
const evidence={status:'running',wasm_sha256:sha(bytes),native_cli_sha256:sha(await fs.readFile(nativeCli)),native_cases:[],wasm_cases:[],scope:'48 original synthetic AL16 cases with actual current portable CLI and retained core/C native references. Two selected native profiles per image. Pure methods use a bare engine even on C packages. Whole plans, canonical assignments, raw-source/profile/export bindings and explicit required-C availability; natural failures are outcome comparisons only. No all-state capacity, C SDK rebuild, error-order or full parity claim.'};
const save=()=>fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2));await save();
const invoke=(engine,request)=>{try{return{outcome:'success',result:engine.execute(request)};}catch(error){return{outcome:'refusal',message:error.message};}};
try{
 for(const row of corpus.rows){
  // The full builders provide a portable native CLI, regardless of WASM image.
  const dir=path.join(root,'native',row.id);await fs.mkdir(dir,{recursive:true});const input=row.request.input;
  const write=async(name,bytes)=>{assert.equal(path.basename(name),name);await fs.writeFile(path.join(dir,name),bytes);};
  for(const source of Object.values(input.source.sources))if(source)await write(source.name,Buffer.from(source.base64,'base64'));
  const partisan=JSON.parse(input.auxiliary_json).native_auxiliary.partisan;await write(partisan.source_name,Buffer.from(partisan.bytes_base64,'base64'));
  const args=['state',...input.arguments,'--output-dir',path.join(dir,'output'),'--version','vBackendProfileRegression','--dpi','72'];
  await fs.writeFile(path.join(dir,'command.json'),JSON.stringify({binary:nativeCli,arguments:args}));
  const result=spawnSync(nativeCli,args,{cwd:dir,windowsHide:true,maxBuffer:64*1024*1024});await fs.writeFile(path.join(dir,'stdout'),result.stdout??'');await fs.writeFile(path.join(dir,'stderr'),result.stderr??'');
  assert.equal(result.status,row.native.core.exit,result.error?.message??result.stderr?.toString());
  if(result.status===0){
   const find=async directory=>{let files=[];for(const item of await fs.readdir(directory,{withFileTypes:true})){const p=path.join(directory,item.name);if(item.isDirectory())files.push(...await find(p));else files.push(p);}return files;};
   const files=await find(path.join(dir,'output')),plans=files.filter(p=>path.basename(p)==='plan.rplan'),manifests=files.filter(p=>path.basename(p)==='manifest.json');assert.equal(plans.length,1);assert.equal(manifests.length,1);
   assert.deepEqual((await read(plans[0])).plan,row.native.core.plan);assert.equal((await read(manifests[0])).native_replay_recipe.recursive_metis_backend,'metis-core');
  }
  evidence.native_cases.push({id:row.id,exit:result.status,exact_plan:result.status===0});await save();
 }
 for(const selected of ['metis-core','c-ffi'])for(const row of corpus.rows){
  const native=row.native[selected==='metis-core'?'core':'c-ffi'],configured=row.configured_backend??selected;
  const used=row.suite==='expanded'?row.fallback==='metis':!['moving-knife','spectral','single-district'].includes(row.method);
  const effective=!used?null:row.method==='prime-factor'?configured:selected;
  const unavailable=!sideRoot&&selected==='c-ffi'&&effective==='c-ffi';
  const engine=effective==='c-ffi'?loaded:bare,input={...row.request.input,native_recursive_backend:selected};
  const prepare=invoke(engine,{operation:'prepare-fresh-native-state-run',input}),run=invoke(engine,{operation:'run-fresh-native-state',input});
  const label=selected+'-'+row.id;await fs.writeFile(path.join(root,label+'.json'),JSON.stringify({input,prepare,run},null,2));
  if(unavailable){assert.equal(prepare.outcome,'refusal');assert.equal(run.outcome,'refusal');}
  else if(native.exit!==0){assert.equal(run.outcome,'refusal');}
  else{
   assert.equal(prepare.outcome,'success',prepare.message);assert.equal(run.outcome,'success',run.message);const r=run.result,b=r.native_backend;
   assert.equal(r.execution_performed,true);assert.equal(r.recorded_paths_opened,false);assert.equal(r.export_error,null);assert.deepEqual(r.export.document.plan,native.plan);
   assert.deepEqual(r.result.assignments,Object.fromEntries(native.plan.units.unit_ids.map((id,i)=>[id,native.plan.assignment[i]+1])));
   assert.equal(b.recursive_backend,selected);assert.equal(b.configured_backend,configured);assert.equal(b.configured_backend_source,row.configured_backend===null?'native-compile-profile-default':'command-line');assert.equal(b.metis_used,used);assert.equal(b.effective_backend,effective);assert.equal(b.configuration_role,!used?'unused':row.method==='prime-factor'?'configured-prime-factor':'native-compile-profile');assert.equal(b.runtime_profile,effective==='c-ffi'?'msvcrt-v1':null);
   assert.deepEqual(b,prepare.result.native_backend);assert.deepEqual(b,r.prepared.native_backend);assert.deepEqual(b,r.export.fresh_native_state_binding.native_backend);assert.deepEqual(r.export.fresh_native_state_binding.arguments,input.arguments);assert.equal(r.input_sha256,prepare.result.input_sha256);
   const resolved=JSON.parse(r.prepared.native_profile_algorithm_json);assert.equal(({RedistMetis:'metis-core',CFfi:'c-ffi',Gpmetis:'gpmetis'})[resolved.metis.engine],configured);
   for(const[k,source]of Object.entries(input.source.sources))if(source)assert.equal(r.selected_sources[k].sha256,sha(Buffer.from(source.base64,'base64')));
  }
  // Preparation of the no-fallback ILP failures must also remain C-independent.
  if(row.suite==='expanded'&&!used){assert.equal(prepare.outcome,'success',prepare.message);assert.equal(prepare.result.native_backend.metis_used,false);assert.equal(prepare.result.native_backend.effective_backend,null);assert.equal(prepare.result.native_backend.runtime_profile,null);}
  evidence.wasm_cases.push({id:row.id,selected_profile:selected,native_exit:native.exit,status:unavailable?'required-c-unavailable':native.exit!==0?'same-failure-outcome-only':'exact-complete-plan-and-binding',sidecar_attached:engine!==bare});await save();
 }
 for(const method of ['bisect','nway','ratio-optimal']){const row=corpus.rows.find(r=>r.id===method+'-metis-core');assert.notDeepEqual(row.native.core.plan.assignment,row.native['c-ffi'].plan.assignment);}
 assert.equal(evidence.native_cases.length,48);assert.equal(evidence.wasm_cases.length,96);evidence.status='passed-maintained-native-backend-profile-regression';await save();console.log(JSON.stringify({root,status:evidence.status,native_cases:48,wasm_cases:96}));
}catch(error){evidence.status='failed';evidence.error=error.stack;await save();throw error;}
