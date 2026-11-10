import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,validateLabConfig} from '../../web/lab/laboratory-project.js';
import {verifyLaboratoryCapture} from '../../web/lab/laboratory-round-capture.js';
const root='target/laboratory-round-capture';await fs.mkdir(root,{recursive:true});
const catalogRoot='dist/wasm-lab-build',manifest=JSON.parse(await fs.readFile(catalogRoot+'/catalog.json','utf8'));
const bytes=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(bytes);
class Worker{postMessage(m){queueMicrotask(()=>{try{this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
const fetcher=async ref=>{const b=await fs.readFile(catalogRoot+'/'+ref);return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
const fixture=JSON.parse(await fs.readFile('target/county-state-oracle/browser.bisect','utf8')),base={...fixture.experiment.config,capture_rounds:true};
let cases=0,refusals=0;
for(const state of ['RI','IA'])for(const kind of ['recursive','empty','build']){
 const config={...base,states:[state],alpha_county:.125,structure:kind==='empty'?'bfs-growth':'standard-bisect',search:kind==='empty'?'single':'multi',districts:kind==='empty'?1:null,...(kind==='build'?{native_build:true}:{})};
 const graph=JSON.parse(await fs.readFile(catalogRoot+'/'+manifest.graphs[state+':2020'].graph_ref,'utf8'));
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed');
 const project=await createLabProject(lab,job.id),row=project.experiment.states[0];assert.equal(row.metrics.requested_options.capture_rounds,true);assert.equal(row.metrics.recorded_options.capture_rounds,true);
 const request={graph,options:row.metrics.requested_options},actual=engine.execute(kind==='build'?{operation:'run-engine-build',request}:request);
 const file=root+'/'+state+'-'+kind+'.json';await fs.writeFile(file,JSON.stringify(kind==='build'?{operation:'run-engine-build',request}:request));
 const native=JSON.parse(execFileSync(kind==='build'?'target/release/examples/execute_tool_request.exe':'target/release/examples/execute_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(actual,native);assert.deepEqual(row.metrics.round_capture,actual.round_capture);
 await verifyLaboratoryCapture(graph,row,project.assignments[state],config);
 assert.equal(kind==='empty',row.metrics.round_capture.events.length===0);
 const restored=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Import must not execute an engine');});const restoredJob=await restored.restoreProject(project);assert.deepEqual(restored.jobs.get(restoredJob.id).states[0].metrics.round_capture,actual.round_capture);
 const plainConfig={...config};delete plainConfig.capture_rounds;const plainRequest={graph,options:{...request.options}};delete plainRequest.options.capture_rounds;const plain=engine.execute(kind==='build'?{operation:'run-engine-build',request:plainRequest}:plainRequest);assert.deepEqual(plain.assignments,actual.assignments);assert.deepEqual(plain.metrics,actual.metrics);assert.equal(plain.round_capture,undefined);
 for(const change of [c=>c.unit_index_json+=' ',c=>c.terminal_matches_returned_assignment=!c.terminal_matches_returned_assignment,c=>c.status='invalid',c=>{c.unit_index_json='[]';c.unit_index_sha256='0'.repeat(64);}]){const p=structuredClone(project);change(p.experiment.states[0].metrics.round_capture);await assert.rejects(()=>restored.restoreProject(p));refusals++;}
 assert.deepEqual(restored.jobs.get(restoredJob.id).states[0].metrics.round_capture,actual.round_capture);
 if(state==='RI'&&kind==='recursive')await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
 if(kind!=='empty'){for(const change of [e=>e.units++,e=>e.matches_returned_assignment=!e.matches_returned_assignment,e=>e.assignments_json+=' ']){const p=structuredClone(project);change(p.experiment.states[0].metrics.round_capture.events[0]);await assert.rejects(()=>restored.restoreProject(p));refusals++;}}
 cases++;
}
for(const value of [null,1,'yes'])assert.throws(()=>validateLabConfig({...base,capture_rounds:value}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_exact_results:cases,inert_project_roundtrips:cases,hostile_trace_refusals:refusals,capture_does_not_change_assignments_or_metrics:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Laboratory round capture: ${cases} native/WASM results and inert project roundtrips; ${refusals} hostile traces refused; no-events and capture-on/off equivalence checked.`);
