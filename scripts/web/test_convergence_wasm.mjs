import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
const year=process.argv[3]||'2020';
const root=process.argv[2]||'dist/wasm-national',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':'')),nativeRun=promisify(execFile),cases=[];
const sha=value=>createHash('sha256').update(value).digest('hex');
const options={structure:'standard-bisect',weights:'geographic',search:'convergence',districts:2,seed:42,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
function score(graph,assignments,o){
 let sum=0;
 for(const [u,v,length]of [...graph.edges].sort((a,b)=>a[0]-b[0]||a[1]-b[1])){
  const du=assignments[graph.geoids[u]],dv=assignments[graph.geoids[v]];if(du===dv)continue;
  let first=1,count=o.districts;
  while(true){const left=Math.floor(count/2),middle=first+left;
   if((du<middle)!==(dv<middle)){sum+=(o.weights==='unweighted'?1:o.weights==='county'&&graph.geoids[u].slice(0,5)===graph.geoids[v].slice(0,5)?length*Math.max(1,o.alpha_county):length)/Math.sqrt(Math.min(left,count-left));break;}
   if(du<middle)count=left;else{first=middle;count-=left;}
  }
 }
 return sum;
}
function replay(request,result){
 let best,tail=0,rejected=0,attempted=0;
 while(tail<request.options.seeds&&attempted<request.options.steps){
  const index=attempted++;let candidate;
  try{candidate=engine.execute({...request,options:{...request.options,search:'single',seed:request.options.seed+index}});}catch{}
  if(!candidate||!candidate.metrics.contiguous||!candidate.metrics.within_requested_tolerance){rejected++;tail++;continue;}
  const cost=score(request.graph,candidate.assignments,request.options);
  if(!best||cost<best.cost){best={cost,index,candidate};tail=0;}else tail++;
 }
 assert.ok(best);assert.deepEqual(result.assignments,best.candidate.assignments);
 const evidence=result.metrics.structure_evidence;
 assert.equal(evidence.attempted,attempted);assert.equal(evidence.rejected,rejected);
 assert.equal(evidence.selected_index,best.index);assert.equal(evidence.selected_seed,String(request.options.seed+best.index));
 assert.equal(evidence.consecutive_non_improving,tail);assert.equal(evidence.halt,tail>=request.options.seeds?'threshold':'seed-limit');
 assert.equal(evidence.best_normalized_cut,best.cost);
}
const available=manifest.catalog.states.filter(state=>manifest.graphs[`${state.code}:${year}`]&&state.years.find(y=>y.year===year)?.districts.congressional>1);
const codes=[...['RI','IA'].filter(code=>available.some(s=>s.code===code)),...available.map(s=>s.code).filter(code=>!['RI','IA'].includes(code))].slice(0,2);
if(!codes.length){console.log('Convergence real-state gate: no multi-district congressional graph in this export; native unit tests and tuning rejection gates remain separate.');process.exit(0);}
const refinements=[{},...['cut','volume'].flatMap(metis_objective=>[1,3].map(metis_trials=>({metis_objective,metis_trials})))];
for(const refinement of refinements)for(const code of codes)for(const weights of ['unweighted','geographic','county'])for(const cap of [1,20]){
 const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:${year}`].graph_ref),'utf8'));
 const k=manifest.catalog.states.find(state=>state.code===code).years.find(entry=>entry.year===year).districts.congressional;
 const request={graph,options:{...options,...refinement,districts:k,weights,steps:cap,alpha_county:weights==='county'?2:0}},file=path.resolve(`target/convergence-${code}-${weights}-${cap}${refinement.metis_objective?`-${refinement.metis_objective}-${refinement.metis_trials}`:""}.json`);
 await fs.writeFile(file,JSON.stringify(request));
 let reference,error;try{reference=JSON.parse((await nativeRun(native,[file],{maxBuffer:16*1024*1024,timeout:120000})).stdout);}catch(e){error=e.stderr;}
 if(error){const message=JSON.parse(error.trim().replace(/^Error: /,''));assert.throws(()=>engine.execute(request),e=>e.message===message);cases.push({state:code,weights,cap,...refinement,status:'matching-rejection',error:message});continue;}
 const actual=engine.execute(request);assert.deepEqual(actual.assignments,reference.assignments);assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics);assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);replay(request,actual);
 cases.push({state:code,weights,cap,...refinement,status:'completed',evidence:actual.metrics.structure_evidence,deviation_percent:actual.metrics.max_deviation_percent,assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id])))});
 console.log(JSON.stringify({state:code,weights,cap,halt:actual.metrics.structure_evidence.halt,attempted:actual.metrics.structure_evidence.attempted}));
}
for(const legacy of cases.filter(c=>!c.metis_objective)){
 const tuned=cases.find(c=>c.state===legacy.state&&c.weights===legacy.weights&&c.cap===legacy.cap&&c.metis_objective==='cut'&&c.metis_trials===1);
 assert.equal(tuned.status,legacy.status);assert.equal(tuned.assignment_sha256,legacy.assignment_sha256);
 const {metis_refinement,...summary}=tuned.evidence;assert.deepEqual(summary,legacy.evidence);
}
assert.ok(cases.some(c=>c.evidence?.halt==='threshold'));assert.ok(cases.some(c=>c.evidence?.halt==='seed-limit'));
const ri=JSON.parse(await fs.readFile(`target/convergence-${codes[0]}-geographic-20.json`,'utf8'));
assert.throws(()=>engine.execute({...ri,graph:{...ri.graph,population:[100000000,...ri.graph.population.slice(1).map(()=>1)]}}),/no feasible candidate/);
manifest.catalog.searches.push('convergence');manifest.catalog.search_compatibility['standard-bisect'].push('convergence');
const fetcher=async name=>{const data=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
class Worker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',error:error.message}});}});}terminate(){this.terminated=true;}}
for(const refinement of refinements)for(const cap of [1,20]){
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
 const config={name:'Convergence RI',mode:'state',states:[codes[0]],year,chamber:'congressional',...options,...refinement,steps:cap,districts:null,timeout_seconds:30};
 const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(run.id).status,'completed');
 const saved=await createLabProject(lab,run.id),restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not execute');});
 await restored.restoreProject(parseLabProject(JSON.stringify(saved)));
 if(refinement.metis_objective){
  for(const [key,value]of Object.entries({objective:'fake',internal_trials_per_candidate:101,iterations:999,scope:'fake',internal_trial_selection:'fake',post_refinement:'fake'})){
   const bad=structuredClone(saved);bad.experiment.states[0].metrics.structure_evidence.metis_refinement[key]=value;
   await assert.rejects(restored.restoreProject(bad),/convergence/);
  }
 }
 for(const tamper of [p=>p.experiment.states[0].metrics.structure_evidence.best_normalized_cut++,p=>p.experiment.states[0].metrics.structure_evidence.threshold++,p=>p.experiment.states[0].metrics.structure_evidence.selected_seed='0',p=>p.experiment.states[0].metrics.structure_evidence.halt='fake',p=>delete p.experiment.states[0].metrics.structure_evidence]){
  const bad=structuredClone(saved);tamper(bad);await assert.rejects(restored.restoreProject(bad),/convergence/);
 }
}
await fs.writeFile('target/native-wasm-convergence-parity.json',JSON.stringify({scope:`${codes.join('/')} ${year} congressional; three boundary-weight modes; threshold 3; seed limits 1/20; seed 42; legacy plus cut/volume and one/three internal trials; native/WASM parity plus independent replay of every sampled candidate, capped/converged project roundtrips and tamper rejection.`,wasm_sha256:sha(bytes),native_sha256:sha(await fs.readFile(native)),cases},null,2));
console.log('Convergence: native/WASM parity, independent candidate replay, distinct limit/threshold stops, no-feasible rejection and secure Save/Open passed.');
