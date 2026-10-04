import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject,verifyLabAssignments} from '../../web/lab/laboratory-project.js';
const root='dist/wasm-national',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':'')),nativeRun=promisify(execFile);
const sha=value=>createHash('sha256').update(value).digest('hex');
const n=12,line={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array(n).fill([-71,41])};
const inputs=[{name:'line-k2',graph:line,k:2},{name:'line-k3',graph:line,k:3},{name:'line-zero',graph:{...line,population:[0,...Array(10).fill(100),0]},k:2},{name:'infeasible',graph:{...line,population:[100000,...Array(11).fill(1)]},k:2}];
for(const code of ['RI','IA'])inputs.push({name:code,graph:JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8')),k:manifest.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional});
const options={structure:'standard-bisect',weights:'geographic',search:'smc-percentile',districts:2,seed:42,seeds:8,steps:20,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:10,smc_particles:16,smc_resample_threshold:0.5};
const cases=[];
function compare(actual,expected){
  if(typeof actual==='number'&&typeof expected==='number')assert.ok(Math.abs(actual-expected)<=Math.max(1e-9,Math.abs(expected)*1e-10));
  else if(Array.isArray(actual)){assert.equal(actual.length,expected.length);actual.forEach((value,i)=>compare(value,expected[i]));}
  else if(actual&&typeof actual==='object'){assert.deepEqual(Object.keys(actual).sort(),Object.keys(expected).sort());for(const key of Object.keys(actual))compare(actual[key],expected[key]);}
  else assert.equal(actual,expected);
}
async function check(name,graph,o){
  const request={graph,options:o},file=path.resolve('target/smc-parity-request.json');await fs.writeFile(file,JSON.stringify(request));
  let actual,error;try{actual=engine.execute(request);}catch(e){error=e.message;}
  if(error){await assert.rejects(nativeRun(native,[file],{timeout:120000,maxBuffer:32*1024*1024}),e=>e.stderr.includes(error));cases.push({name,options:o,status:'matching-rejection',error});return;}
  const reference=JSON.parse((await nativeRun(native,[file],{timeout:120000,maxBuffer:32*1024*1024})).stdout);
  assert.deepEqual(actual.assignments,reference.assignments);compare(actual.metrics,reference.metrics);
  const state={metrics:{...actual.metrics,district_count:o.districts,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};
  verifyLabAssignments(graph,state,actual.assignments,o);
  assert.ok(actual.metrics.structure_evidence.ranked_particles.find(p=>p.particle===actual.metrics.structure_evidence.selected_particle).weight>0);
  for(const mutate of [e=>e.selected_edge_cut++,e=>e.particles++,e=>e.ranked_particles[0].weight=2,e=>e.selected_particle=o.smc_particles,e=>e.rng='unknown']){
    const bad=structuredClone(state);mutate(bad.metrics.structure_evidence);assert.throws(()=>verifyLabAssignments(graph,bad,actual.assignments,o),/SMC/);
  }
  const changed=engine.execute({graph,options:{...o,iterations:1,seeds:1,steps:1}});assert.deepEqual(changed.assignments,actual.assignments);
  cases.push({name,options:o,status:'completed',contiguous:actual.metrics.contiguous,balance_passed:actual.metrics.within_requested_tolerance,maximum_deviation_percent:actual.metrics.max_deviation_percent,summary:actual.metrics.structure_evidence});
}
for(const {name,graph,k}of inputs)for(const weights of ['geographic','unweighted','county'])for(const percentile of [0,0.5,1])await check(name,graph,{...options,districts:k,weights,percentile,alpha_county:weights==='county'?2:0});
for(const threshold of [0,1])for(const particles of [8,32])await check('resampling',inputs[4].graph,{...options,smc_particles:particles,smc_resample_threshold:threshold,percentile:0.5,seed:99});
for(const overrides of [{smc_particles:0},{smc_particles:10001},{smc_resample_threshold:-1},{smc_resample_threshold:1.01},{balance_tolerance:5},{search:'multi'},{structure:'nway'}])assert.throws(()=>engine.execute({graph:line,options:{...options,...overrides}}));
manifest.wasm_sha256=sha(bytes);manifest.catalog.searches.push('smc-percentile');manifest.catalog.search_compatibility['standard-bisect'].push('smc-percentile');
const assets=new Map(),catalog=structuredClone(manifest);
function asset(value){const data=Buffer.from(JSON.stringify(value)),hash=sha(data),ref=`assets/${hash}.json`;assets.set(ref,data);catalog.assets[ref]=hash;return ref;}
catalog.graphs['RI:2020']={...catalog.graphs['RI:2020'],graph_ref:asset(line),native_graph_sha256:'3'.repeat(64)};
catalog.geometries['RI:2020']=asset({type:'FeatureCollection',features:line.geoids.map((id,i)=>({type:'Feature',properties:{geoid:id,state:'RI',county:id.slice(0,5)},geometry:{type:'Point',coordinates:[-71+i*.01,41]}}))});
const fetcher=async ref=>{const data=assets.get(ref)||await fs.readFile(path.join(root,ref));return {ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
class Worker{postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(e){this.onmessage({data:{type:'error',error:e.message}});}});}terminate(){this.terminated=true;}}
const config={name:'SMC fixture',mode:'state',states:['RI'],year:'2020',chamber:'congressional',...options,timeout_seconds:30};
for(const source of [catalog,manifest]){
 const lab=new WasmCatalog(source,fetcher,()=>new Worker()),run=lab.submit({...config,districts:source===catalog?2:null});
 while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(run.id).status,'completed');
 const saved=await createLabProject(lab,run.id),opened=new WasmCatalog(source,fetcher,()=>{throw new Error('Opening must not execute engine');});
 const restored=await opened.restoreProject(parseLabProject(JSON.stringify(saved)));
 assert.equal(restored.config.smc_particles,16);assert.equal(restored.config.smc_resample_threshold,0.5);
 assert.equal((await opened.api(`/api/runs/${restored.id}/RI/map`)).features.length,restored.states[0].metrics.units);
 const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.selected_particle=16;
 await assert.rejects(opened.restoreProject(bad),/SMC/);
}
await fs.writeFile('target/native-wasm-smc-parity.json',JSON.stringify({wasm_sha256:sha(bytes),native_sha256:sha(await fs.readFile(native)),scope:'Exact assignments with numerical metric/weight tolerance; native shared SMC implementation, independent ledger/metrics checks, fixture and RI project/map roundtrips. No distribution certification.',cases},null,2));
console.log(JSON.stringify({cases:cases.length,successes:cases.filter(c=>c.status==='completed').length,matching_rejections:cases.filter(c=>c.status!=='completed').length,constraint_misses:cases.filter(c=>c.status==='completed'&&(!c.contiguous||!c.balance_passed)).length}));
