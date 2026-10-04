import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {verifyAssignments} from '../../web/lab/assignment-verification.js';
import {Worker as NodeWorker} from 'node:worker_threads';

const root=process.argv[2]||'dist/wasm-vra-ui',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const states=['RI','IA','NC'].filter(code=>manifest.graphs[`${code}:2020`]);assert.ok(states.length);
const fetcher=async ref=>{const data=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm'))),demographics={},graphs={};let executions=0;
for(const code of states){
 const graph=graphs[code]=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8'));
 demographics[code]={schema_version:'bisect-demographic-fractions-v1',state:code,year:'2020',basis:'total-population',source_label:'SYNTHETIC regression fractions — not Census demographics',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,i<graph.geoids.length/2?0.9:0.1]))};
 await fs.writeFile(`target/vra-ui-${code}-synthetic.json`,JSON.stringify(demographics[code]));
}
class EngineWorker{
 postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:message.id,result:engine.execute(message.request)}});}}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}
 terminate(){this.terminated=true;}
}
const workerUrl=new URL('../../web/lab/assignment-verification-worker.js',import.meta.url).href;
function verifier(){const native=new NodeWorker(`const {parentPort}=require('node:worker_threads');globalThis.self={postMessage:data=>parentPort.postMessage(data)};import(${JSON.stringify(workerUrl)}).then(()=>parentPort.on('message',data=>self.onmessage({data})));`,{eval:true});const adapter={postMessage:data=>native.postMessage(data),terminate:()=>native.terminate()};native.on('message',data=>adapter.onmessage({data}));native.on('error',()=>adapter.onerror());return adapter;}
const base={name:'VRA demographic project regression',mode:'national',states,year:'2020',chamber:'congressional',districts:null,structure:'standard-bisect',weights:'geographic',search:'vra-recom',seed:42,seeds:8,steps:20,percentile:0.5,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,timeout_seconds:30,vra_threshold:0.5,demographics};
const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker()),restorer=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute the engine.');});let exact=0,tampered=0,recomCases=0,sectionCases=0,variantIndex=0;
const variants=[{steps:0},{steps:20},...['single','multi'].flatMap(search=>[0,0.4,1].map(w_vra=>({structure:'ratio-optimal-vra',search,w_vra})))];
for(const variant of variants){
 const section=variant.structure==='ratio-optimal-vra',config={...base,...variant};if(section)delete config.vra_threshold;const caseId=variantIndex++;validateLabConfig(config);const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
 const project=await createLabProject(lab,run.id);assert.equal(project.experiment.status,'completed',JSON.stringify(project.experiment.logs));parseLabProject(JSON.stringify(project));
 const before=executions,restored=await restorer.restoreProject(project);assert.equal(executions,before);assert.deepEqual(restored.config,config);
 for(const state of project.experiment.states){
  const code=state.code,request={graph:graphs[code],options:state.metrics.requested_options,demographics:config.demographics[code]},file=`target/vra-ui-native-${caseId}-${code}.json`;await fs.writeFile(file,JSON.stringify(request));
  const native=spawnSync('target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);const result=JSON.parse(native.stdout);
  assert.deepEqual(result.assignments,project.assignments[code]);for(const [key,value]of Object.entries(result.metrics))assert.deepEqual(state.metrics[key==='districts'?'district_count':key],value);
  await verifyAssignments(graphs[code],state,project.assignments[code],config,{workerFactory:verifier});
  const map=await restorer.api(`/api/runs/${restored.id}/${code}/map`);assert.equal(map.features.length,graphs[code].geoids.length);assert.ok(map.features.every(f=>f.properties.district===project.assignments[code][f.properties.geoid]));exact++;if(section)sectionCases++;else recomCases++;
 }
 const reopened=await createLabProject(restorer,restored.id);assert.deepEqual(reopened.experiment.config.demographics,demographics);
 const primary=states[0],jobs=restorer.jobs.size,outputs=restorer.outputs.size;
 for(const mutate of [
  p=>p.experiment.config.demographics[primary].minority_fractions[graphs[primary].geoids[0]]=0.8,
  p=>p.experiment.config.demographics[primary].year='2010',
  p=>delete p.experiment.config.demographics[primary].minority_fractions[graphs[primary].geoids[0]],
  p=>p.experiment.states[0].metrics.structure_evidence.demographics_sha256='0'.repeat(64),
  ...(section?[p=>p.experiment.states[0].metrics.structure_evidence.alignment++,p=>p.experiment.states[0].metrics.structure_evidence.selection_score++]:[p=>p.experiment.states[0].metrics.structure_evidence.run.protected_districts=[0],p=>p.experiment.states[0].metrics.structure_evidence.run.accepted_moves++]),
 ]){const bad=structuredClone(project);mutate(bad);await assert.rejects(restorer.restoreProject(bad),/VRA|demographic|Demographic/);assert.equal(restorer.jobs.size,jobs);assert.equal(restorer.outputs.size,outputs);tampered++;}
 if(section)await fs.writeFile('target/vra-section-ui-project-fixture.bisect',JSON.stringify(project));else if(config.steps===20)await fs.writeFile('target/vra-ui-project-fixture.bisect',JSON.stringify(project));
}
for(const change of [{vra_threshold:1.1},{balance_tolerance:5},{demographics:{}},{structure:'nway'}])assert.throws(()=>validateLabConfig({...base,...change}));
await fs.writeFile('target/vra-ui-project-verification.json',JSON.stringify({real_graphs_synthetic_demographics:true,states,exact_native_wasm_cases:exact,recom_cases:recomCases,vra_section_cases:sectionCases,atomic_tamper_rejections:tampered,real_verifier_workers:true,engine_execution_on_open:false},null,2));
console.log(`VRA projects (ReCom ${recomCases}, VRASection ${sectionCases}): ${exact} real-graph native/WASM and Save/Open/map cases with explicitly synthetic fractions; ${tampered} atomic tamper rejections, real verification Workers, complete demographic persistence, no engine execution on Open.`);
