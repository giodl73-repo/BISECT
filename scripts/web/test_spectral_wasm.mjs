import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';
const root=process.argv[2]||'dist/wasm-national',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
const native=path.resolve('target/release/examples/execute_request'+(process.platform==='win32'?'.exe':'')),nativeRun=promisify(execFile),cases=[];
const sha=value=>createHash('sha256').update(value).digest('hex');
const options={structure:'spectral',weights:'unweighted',search:'single',districts:2,seed:42,seeds:8,steps:200,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
for(const code of ['RI','IA'])for(const steps of [1,20,200])for(const weights of ['unweighted','geographic','county']){
 const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[`${code}:2020`].graph_ref),'utf8'));
 const k=manifest.catalog.states.find(state=>state.code===code).years.find(year=>year.year==='2020').districts.congressional;
 const request={graph,options:{...options,districts:k,steps,weights,alpha_county:weights==='county'?2:0}},file=path.resolve(`target/spectral-${code}-${steps}-${weights}.json`);
 await fs.writeFile(file,JSON.stringify(request));
 const reference=JSON.parse((await nativeRun(native,[file],{maxBuffer:16*1024*1024,timeout:120000})).stdout),actual=engine.execute(request);
 assert.deepEqual(actual.assignments,reference.assignments);
 assert.deepEqual(actual.metrics.district_metrics,reference.metrics.district_metrics);
 assert.deepEqual(actual.metrics.structure_evidence,reference.metrics.structure_evidence);
 for(const key of ['graph_boundary_m','weighted_boundary'])assert.ok(Math.abs(actual.metrics[key]-reference.metrics[key])<=Math.max(1e-8,Math.abs(reference.metrics[key])*1e-10));
 assert.equal(actual.metrics.structure_evidence.max_iters,steps);
 assert.equal(actual.metrics.structure_evidence.nodes.length,k-1);
 assert.ok(actual.metrics.structure_evidence.nodes.every(node=>node.iterations<=steps));
 const changed=engine.execute({graph,options:{...request.options,seed:999,iterations:1}});
 assert.deepEqual(actual.assignments,changed.assignments);assert.deepEqual(actual.metrics.structure_evidence,changed.metrics.structure_evidence);
 cases.push({state:code,steps,weights,districts:k,contiguous:actual.metrics.contiguous,deviation_percent:actual.metrics.max_deviation_percent,balanced:actual.metrics.within_requested_tolerance,assignment_sha256:sha(JSON.stringify(graph.geoids.map(id=>actual.assignments[id]))),nodes:actual.metrics.structure_evidence.nodes});
 console.log(JSON.stringify({state:code,steps,weights,contiguous:actual.metrics.contiguous,deviation_percent:actual.metrics.max_deviation_percent}));
}
const ri=JSON.parse(await fs.readFile('target/spectral-RI-200-unweighted.json','utf8'));
assert.deepEqual(engine.execute(ri).assignments,engine.execute({...ri,options:{...ri.options,weights:'geographic'}}).assignments);
assert.throws(()=>engine.execute({...ri,options:{...ri.options,search:'multi'}}),/does not execute/);
assert.throws(()=>engine.execute({...ri,graph:{...ri.graph,population:[100000000,...ri.graph.population.slice(1).map(()=>1)]}}),/no population-balanced/);
manifest.catalog.structures.push('spectral');manifest.catalog.search_compatibility.spectral=['single'];
const fetcher=async name=>{const data=await fs.readFile(path.join(root,name));return{ok:true,arrayBuffer:async()=>data.buffer.slice(data.byteOffset,data.byteOffset+data.length)};};
class Worker {postMessage(message){setImmediate(()=>{if(this.terminated)return;try{this.onmessage({data:message.type==='initialize'?{type:'ready'}:{type:'result',id:message.id,result:engine.execute(message.request)}});}catch(error){this.onmessage({data:{type:'error',error:error.message}});}});}terminate(){this.terminated=true;}}
for(const weights of ['unweighted','geographic','county']){
const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());
const config={name:'Spectral RI',mode:'state',states:['RI'],year:'2020',chamber:'congressional',...options,weights,alpha_county:weights==='county'?2:0,districts:null,timeout_seconds:30};
const run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(run.id).status,'completed');
const saved=await createLabProject(lab,run.id),restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Opening must not execute');});
await restored.restoreProject(parseLabProject(JSON.stringify(saved)));
for(const tamper of [p=>p.experiment.states[0].metrics.structure_evidence.edge_cut++,p=>p.experiment.states[0].metrics.structure_evidence.nodes[0].population_deviation++,p=>p.experiment.states[0].metrics.structure_evidence.max_iters++,p=>delete p.experiment.states[0].metrics.structure_evidence]){
 const bad=structuredClone(saved);tamper(bad);await assert.rejects(restored.restoreProject(bad),/spectral/);
}
}
await fs.writeFile('target/native-wasm-spectral-parity.json',JSON.stringify({scope:'RI/IA 2020 congressional spectral recursion; all three reported boundary weights; maximum iterations 1/20/200; seed and METIS-iteration independence; secure project evidence checks.',wasm_sha256:sha(bytes),native_sha256:sha(await fs.readFile(native)),cases},null,2));
console.log('Spectral: exact native/WASM assignments and summaries, parameter independence, invalid settings, infeasible cuts and secure Save/Open passed.');
