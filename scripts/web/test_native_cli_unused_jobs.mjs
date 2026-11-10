import {pathToFileURL} from 'node:url';
// Real WASM/controller calls with a rejecting observer, not mocked engine replies.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
const repo=path.resolve(process.argv[3]??'.');
const {instantiateEngine}=await import(pathToFileURL(path.join(repo,'web/lab/wasm-engine.js')));
const {nativeEngineInputRequest}=await import(pathToFileURL(path.join(repo,'web/lab/native-adjacency-files.js')));
const {executeNativeRunJob}=await import(pathToFileURL(path.join(repo,'web/lab/native-run-job.js')));
const {executeNativeStatesJob}=await import(pathToFileURL(path.join(repo,'web/lab/native-states-job.js')));
const {createStateWorkflowBinding,stateWorkflowSettingsPatch}=await import(pathToFileURL(path.join(repo,'web/lab/native-state-workflow.js')));
const {createNativeStateSourceJob}=await import(pathToFileURL(path.join(repo,'web/lab/native-state-sources.js')));
const {preparedStateRunZip}=await import(pathToFileURL(path.join(repo,'web/lab/prepared-state-job.js')));
const {createProject,parseProject}=await import(pathToFileURL(path.join(repo,'web/lab/project.js')));
const {executeNativeStateJob}=await import(pathToFileURL(path.join(repo,'web/lab/native-state-job.js')));
const root=path.resolve(process.argv[2]),out=path.join(root,'wasm-cli-execution');await fs.mkdir(out,{recursive:true});
const status=JSON.parse(await fs.readFile(path.join(root,'status.json'),'utf8')),fixture=JSON.parse(await fs.readFile(path.join(root,'fixture.json'),'utf8'));
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
const bytes=await fs.readFile(path.join(root,'owned/engine.wasm'));assert.equal(sha(bytes),status.artifacts['engine.wasm'].sha256);
const engine=await instantiateEngine(bytes),sources={geometry:null};
for(const [key,suffix]of [['adjacency','.adj.bin'],['geoids','_geoids.json'],['centroids','_centroids.json']]){
 const file=path.join(fixture.input,fixture.stem+suffix);sources[key]={name:path.basename(file),base64:(await fs.readFile(file)).toString('base64')};
}
const settings={state:'RI',year:'2020',unit_kind:'tract'},preparation=nativeEngineInputRequest(sources,settings);
const prepared=engine.execute(preparation);assert.equal(prepared.execution_performed,false);
await fs.writeFile(path.join(out,'fixture-preparation.json'),JSON.stringify({purpose:'Explicit fixture setup before intent tests, not a Run under print-only',request:preparation,reply:prepared}));
const graphInputs={graph:{name:'RI-2020-graph.json',text:prepared.graph_json},auxiliary:null,settings:{label:'rhode_island_congressional_2020',chamber:'congressional',created_at:'2026-11-01T04:03:23Z',inspect_only:true}};

const declaration={status:'declared-before-execution',scope:'Same retained four-unit source and exact native CLI argument vectors; Run/States selected graph, State selected original adjacency workflow. Print-only is separately named browser adaptation. No complete/native filesystem/C-profile claim.',native_status_sha256:sha(await fs.readFile(path.join(root,'status.json'))),wasm_sha256:sha(bytes),cases:status.cases.map(c=>({case:c.case,family:c.family,arguments:c.arguments,expected_native_assignments:c.assignments,print_only:c.changed_field==='print_only'}))};
await fs.writeFile(path.join(out,'declaration.json'),JSON.stringify(declaration,null,2));
const rows=[],effectiveBaselines=new Map();
for(const nativeCase of status.cases){
 const family=nativeCase.family,command={arguments_json:JSON.stringify(nativeCase.arguments.slice(1))},calls=[];
 const inspection=engine.execute({operation:'inspect-native-'+family+'-command',arguments:JSON.parse(command.arguments_json)});
 await fs.writeFile(path.join(out,nativeCase.case+'-inspection.json'),JSON.stringify(inspection,null,2));assert.equal(inspection.status,'parsed');assert.deepEqual(inspection.arguments,nativeCase.arguments.slice(1));
 const effective=family==='state'?inspection.effective_algorithm:inspection.effective_configuration;
 if(nativeCase.changed_field===null)effectiveBaselines.set(family,{effective,fields:inspection.fields});
 else{const baseline=effectiveBaselines.get(family);assert.deepEqual(effective,baseline.effective);const changed=inspection.fields.find(f=>f.id===nativeCase.changed_field),before=baseline.fields.find(f=>f.id===nativeCase.changed_field);assert.equal(changed.source,'command-line');assert.notDeepEqual(changed.values,before.values);}

 const facade={artifactIdentity:engine.artifactIdentity,execute(request,options){calls.push(structuredClone(request));return engine.execute(request,options);}};
 let result=null,error=null,job=null,binding=null,projectFiles=null;
 const caseDir=path.join(out,nativeCase.case);await fs.mkdir(caseDir,{recursive:true});
 try{
  if(family==='state'){
   const review=engine.execute({operation:'inspect-native-state-command',arguments:JSON.parse(command.arguments_json)});
   binding=await createStateWorkflowBinding(command,review,{sourcePreparation:preparation});
   const files={nativeAdjacencyInputs:sources,nativeAdjacencySettings:settings,nativeStateSourceSettings:{enabled:true},stateCommandInputs:command,stateGraphInputs:{...graphInputs,graph:null}};
   const selected={...files,...stateWorkflowSettingsPatch(binding,files)};projectFiles=selected;job=await createNativeStateSourceJob(selected,'native',{selectedScope:true});job.scope.districts=2;
   const value=await executeNativeStateJob(facade,job);result=value.result??value;
  }else result=await (family==='run'?executeNativeRunJob:executeNativeStatesJob)(facade,{command,graph_inputs:graphInputs});
 }catch(failure){error=failure.message;}
 await fs.writeFile(path.join(caseDir,'raw.json'),JSON.stringify({original_command:command,job,binding,calls,result,error},null,2));
 if(nativeCase.changed_field==='print_only'){
  if(family==='state')assert.equal(error,'Remaining State flags need another workflow before source preparation or acquisition: --print-only');
  else{assert.equal(error,null);assert.equal(result.execution_performed,false);assert.equal(result[family==='run'?'native_run_print_only':'native_states_print_only'],true);}
  assert(!calls.some(c=>c.operation==='run-prepared-engine'&&c.input.inspect_only===false));
  assert(!calls.some(c=>['prepare-native-engine-input','prepare-native-adjacency-state-run','prepare-census-engine-input','prepare-tract-engine-input'].includes(c.operation)));
  rows.push({case:nativeCase.case,status:'passed-browser-intent-adaptation',partition_calls:0});
 }else{
  assert.equal(error,null,nativeCase.case+': '+error);
  const expected=Object.fromEntries(fixture.ids.map((id,index)=>[id,nativeCase.assignments[String(index)]]));
  assert.deepEqual(result.prepared.result.assignments,expected,nativeCase.case);
  const recipe=result['native_'+family].recipe;
  assert.equal((binding?binding.source_command:recipe.command).arguments_json,command.arguments_json);
  const archive=await preparedStateRunZip(result);await fs.writeFile(path.join(caseDir,'result.zip'),archive);
  const operation=family==='state'?'run-native-selected-state-source':'run-native-'+family+'-graph';
  const project=createProject({name:nativeCase.case,files:projectFiles??{[family==='run'?'runCommandInputs':'statesCommandInputs']:command,stateGraphInputs:graphInputs},operation,constraints:[],result,lastOperation:operation});
  const encoded=JSON.stringify(project);await fs.writeFile(path.join(caseDir,'result.bisect'),encoded);
  const reopened=parseProject(encoded);assert.deepEqual(reopened.result,result);
  rows.push({case:nativeCase.case,status:'passed',native_assignments_one_based:nativeCase.assignments,wasm_assignments_geoid_one_based:result.prepared.result.assignments,accepted_seed:result.prepared.result.build_report.attempts.at(-1).seed,partition_calls:calls.filter(c=>c.operation==='run-prepared-engine'&&!c.input.inspect_only).length,zip_sha256:sha(archive),project_sha256:sha(Buffer.from(encoded))});
 }
 await fs.writeFile(path.join(out,'progress.json'),JSON.stringify({status:'running',cases:rows},null,2));
 console.log(JSON.stringify(rows.at(-1)));
}
assert.equal(rows.filter(r=>r.status==='passed').length,22);assert.equal(rows.filter(r=>r.status==='passed-browser-intent-adaptation').length,3);
await fs.writeFile(path.join(out,'evidence.json'),JSON.stringify({status:'passed',scope:declaration.scope,actual_cli_cases:25,wasm_execution_matches:22,print_only_adaptations:3,wasm:engine.artifactIdentity,cases:rows},null,2));
