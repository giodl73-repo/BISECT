import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,createLabProject,parseLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
const root='dist/wasm-national', manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
manifest.wasm_sha256=createHash('sha256').update(bytes).digest('hex');
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':'')),run=promisify(execFile);
const sha=bytes=>createHash('sha256').update(bytes).digest('hex'),cases=[];
const options={structure:'simulated-annealing',weights:'unweighted',search:'single',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,sa_steps_per_tract:10,sa_t0_factor:0.01,sa_t_final:0.0001};
const n=12,line={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array(n).fill([-71,41])};
const inputs=[{name:'line-k2',graph:line,k:2},{name:'line-zero',graph:{...line,population:[0,100,100,100,100,100,100,100,100,100,100,0]},k:2},{name:'line-k3',graph:line,k:3},{name:'line-k6',graph:line,k:6},{name:'infeasible',graph:{...line,population:[100000,...line.population.slice(1)]},k:2}];
for(const code of ['RI','IA'])inputs.push({name:code,graph:JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8')),k:manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional});
let successes=0,failures=0;
for(const {name,graph,k}of inputs)for(const sa_steps_per_tract of [0,2,10])for(const weights of ['unweighted','geographic','county']){
 const request={graph,options:{...options,sa_steps_per_tract,districts:k,weights,alpha_county:weights==='county'?2:0}},file=path.resolve(`target/annealing-${name}-${sa_steps_per_tract}-${weights}.json`);
 await fs.writeFile(file,JSON.stringify(request));
 let actual,error;try{actual=engine.execute(request);}catch(e){error=e.message;}
 if(error){
  await assert.rejects(run(native,[file],{timeout:120000,maxBuffer:16*1024*1024}),e=>e.stderr.includes(error));failures++;cases.push({name,sa_steps_per_tract,weights,status:'rejected',error});continue;
 }
 const reference=JSON.parse((await run(native,[file],{timeout:120000,maxBuffer:16*1024*1024})).stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics);assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);
 assert.equal(actual.metrics.contiguous,true);assert.equal(actual.metrics.within_requested_tolerance,true);
 const changed=engine.execute({graph,options:{...request.options,iterations:1,steps:1,seeds:1}});assert.deepEqual(changed.assignments,actual.assignments);assert.deepEqual(changed.metrics.structure_evidence,actual.metrics.structure_evidence);
 const state={metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:true}},config=request.options;
 verifyLabAssignments(graph,state,actual.assignments,config);
 for(const tamper of [e=>e.steps_per_tract++,e=>e.t0_factor++,e=>e.t_final++,e=>e.population_targets='half',e=>e.objective='weighted']){
  const bad=structuredClone(state);tamper(bad.metrics.structure_evidence);assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,config),/annealing/);
 }
 successes++;cases.push({name,sa_steps_per_tract,weights,status:'completed',districts:k,summary:actual.metrics.structure_evidence});
}
assert.ok(successes>=9);assert.ok(failures>=3);
assert.throws(()=>engine.execute({graph:line,options:{...options,search:'multi'}}),/does not execute/);
for(const change of [{sa_steps_per_tract:-1},{sa_steps_per_tract:10001},{sa_t0_factor:-1},{sa_t_final:0},{sa_t_final:Infinity}])assert.throws(()=>engine.execute({graph:line,options:{...options,...change}}),/annealing|finite|expected usize/);
const realGraph=inputs.find(input=>input.name==='RI').graph;
for(const change of [{seed:99,sa_t0_factor:0,sa_t_final:1e-15},{seed:123,sa_t0_factor:0.2,sa_t_final:0.01}]){
 const request={graph:realGraph,options:{...options,...change}},file=path.resolve(`target/annealing-RI-parameters-${change.seed}.json`);await fs.writeFile(file,JSON.stringify(request));
 const actual=engine.execute(request),reference=JSON.parse((await run(native,[file],{timeout:120000,maxBuffer:16*1024*1024})).stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics);
 successes++;cases.push({name:'RI-parameter-variation',options:request.options,status:'completed',summary:actual.metrics.structure_evidence});
}
// Portable-project integration on a feasible fixture, including actual hashed
// inputs and a second catalog whose engine factory is forbidden during Open.
if(!manifest.catalog.structures.includes('simulated-annealing'))manifest.catalog.structures.push('simulated-annealing');manifest.catalog.search_compatibility['simulated-annealing']=['single'];
const catalog=structuredClone(manifest),assets=new Map();
function asset(value){const data=Buffer.from(JSON.stringify(value)),hash=sha(data),ref=`assets/${hash}.json`;assets.set(ref,data);catalog.assets[ref]=hash;return ref;}
const graphRef=asset(line),geometryRef=asset({type:'FeatureCollection',features:line.geoids.map((id,i)=>({type:'Feature',properties:{geoid:id,state:'RI',county:id.slice(0,5)},geometry:{type:'Point',coordinates:[-71+i*.01,41]}}))});
catalog.graphs['RI:2020']={...catalog.graphs['RI:2020'],graph_ref:graphRef,native_graph_sha256:'3'.repeat(64)};catalog.geometries['RI:2020']=geometryRef;
if(!catalog.catalog.structures.includes('simulated-annealing'))catalog.catalog.structures.push('simulated-annealing');catalog.catalog.search_compatibility['simulated-annealing']=['single'];
const fetcher=async ref=>{const data=assets.get(ref);assert.ok(data);return {ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
class Worker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',error:error.message}});}});}terminate(){this.terminated=true;}}
const lab=new WasmCatalog(catalog,fetcher,()=>new Worker());
const config={name:'Annealing fixture',mode:'state',states:['RI'],year:'2020',chamber:'congressional',...options,timeout_seconds:30};
for(const change of [{sa_steps_per_tract:-1},{sa_t0_factor:NaN},{sa_t_final:0}])assert.throws(()=>validateLabConfig({...config,...change}),/annealing/);
const missing={...config};delete missing.sa_t_final;assert.throws(()=>validateLabConfig(missing),/configuration/);
const job=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(job.id).status,'completed');
const saved=await createLabProject(lab,job.id),opened=new WasmCatalog(catalog,fetcher,()=>{throw new Error('Open must not execute');});
const restored=await opened.restoreProject(parseLabProject(JSON.stringify(saved)));assert.deepEqual(restored.states,saved.experiment.states);
assert.equal((await opened.api(`/api/runs/${restored.id}/RI/map`)).features.length,n);
const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.steps_per_tract++;
const count=opened.jobs.size;await assert.rejects(opened.restoreProject(bad),/annealing/);assert.equal(opened.jobs.size,count);
const diskFetcher=async ref=>{const data=await fs.readFile(path.join(root,ref));return {ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
for(const weights of ['unweighted','geographic','county']){
 const realLab=new WasmCatalog(manifest,diskFetcher,()=>new Worker());
 const realJob=realLab.submit({...config,name:'RI annealing',districts:null,weights,alpha_county:weights==='county'?2:0});
 while(realLab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(realLab.jobs.get(realJob.id).status,'completed');
 const realSaved=await createLabProject(realLab,realJob.id),realOpened=new WasmCatalog(manifest,diskFetcher,()=>{throw new Error('Open must not execute');});
 const realRestored=await realOpened.restoreProject(parseLabProject(JSON.stringify(realSaved)));
 const map=await realOpened.api(`/api/runs/${realRestored.id}/RI/map`);assert.equal(map.features.length,realRestored.states[0].metrics.units);
 assert.ok(map.features.every(feature=>feature.properties.district===realSaved.assignments.RI[feature.properties.geoid]));
}
await fs.writeFile('target/native-wasm-annealing-parity.json',JSON.stringify({wasm_sha256:sha(bytes),native_sha256:sha(await fs.readFile(native)),successes,failures,cases},null,2));
console.log(`Simulated annealing: ${successes} exact native/WASM successes, ${failures} matching failures, deterministic option checks and imported-summary tamper rejection passed.`);
