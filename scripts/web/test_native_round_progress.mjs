import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateNativeRoundProgress,nativeRoundProgressText} from '../../web/lab/native-round-progress.js';
import {createProject} from '../../web/lab/project.js';
import {preparedStateJobFromResult,executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
const root='target/native-round-progress';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),before=await instantiateEngine(await fs.readFile(root+'/before/engine.wasm'));
const catalog=JSON.parse(await fs.readFile('dist/wasm-lab-build/catalog.json','utf8')),graph=JSON.parse(await fs.readFile('dist/wasm-lab-build/'+catalog.graphs['RI:2020'].graph_ref,'utf8'));
const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:4,seed:'18446744073709551615',seeds:3,steps:5,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1.1,iterations:10,capture_rounds:true};
let cases=0,notifications=0;const records=[];
function check(progress,result,g){
 const trace=result.round_capture;assert.equal(progress.length,trace.events.length);
 for(const [i,event]of trace.events.entries()){const p=validateNativeRoundProgress(progress[i]);assert.equal(p.index,i+1);assert.equal(p.name,event.name);assert.equal(p.assigned_units,event.units);assert.equal(p.units,g.geoids.length);assert.equal(p.complete_universe,event.complete_universe);assert.match(nativeRoundProgressText(p),/Candidate or partial/);}
}
function run(request,g){const events=[];const actual=engine.execute(request,{onProgress:p=>{if(p.schema_version==='bisect-native-round-progress-v1'){validateNativeRoundProgress(p);assert.throws(()=>engine.execute(request),/current WASM operation/);events.push(structuredClone(p));}}});assert.deepEqual(actual,before.execute(request));check(events,actual,g);notifications+=events.length;cases++;return {actual,events};}
for(const [name,extra]of [['recursive',{}],['multi',{search:'multi'}],['percentile',{search:'percentile'}],['bfs',{structure:'bfs-growth'}],['cvd',{structure:'centroidal-voronoi',cvd_iters:5,cvd_metric:'geographic'}],['mka',{structure:'moving-knife',mka_orientations:4,mka_metric:'reock'}],['geosection',{structure:'ratio-optimal'}],['spectral',{structure:'spectral'}],['one-district',{districts:1}]]){
 const request={graph,options:{...options,...extra}},r=run(request,graph);await fs.writeFile(root+'/'+name+'.request.json',JSON.stringify(request));records.push({name,request,result:r.actual,events:r.events});
 const off={graph,options:{...request.options,capture_rounds:undefined}},silent=[];const plain=engine.execute(off,{onProgress:p=>silent.push(p)});assert.deepEqual(plain.assignments,r.actual.assignments);assert.deepEqual(plain.metrics,r.actual.metrics);assert.equal(silent.filter(p=>p.schema_version==='bisect-native-round-progress-v1').length,0);
}
for(const name of ['request-0-0.json','request-24-0.json','request-48-0.json'])for(const adaptive of [false,true]){
 const request=JSON.parse(await fs.readFile('target/multiscale-fixtures/'+name,'utf8'));request.input.request.options.capture_rounds=true;request.input.request.options.seed='42';request.input.total_steps=5;request.input.adaptive=adaptive?{target_accept:.3,adapt_interval:2,gamma_0:.1,coarse_tol_factor:1}:null;
 const r=run(request,request.input.request.graph);records.push({name:'multiscale-'+cases,request,result:r.actual,events:r.events});
}
// Refusal by the host latches failure rather than returning a partially captured result.
assert.throws(()=>engine.execute({graph,options},{onProgress(){throw Error('listener refused native event');}}),/listener refused/);const recovered=run({graph,options},graph);assert.equal(recovered.events[0].index,1);
const sample=recovered.events[0];for(const change of [{index:0},{index:129},{assigned_units:sample.units+1},{complete_universe:!sample.complete_universe},{name:'<script>'},{extra:true}])assert.throws(()=>validateNativeRoundProgress({...sample,...change}));
// Failed population retries may emit observations; none imply accepted output.
const retryEvents=[];assert.throws(()=>engine.execute({operation:'run-engine-build',request:{graph,options:{...options,seed:'42',structure:'bfs-growth',districts:2,balance_tolerance:0.01}}},{onProgress:p=>retryEvents.push(p)}));assert(retryEvents.length>1);assert(retryEvents.every(p=>p.index===1));
// The exact prepared job wrapper must relay native progress as well as its stage messages.
const inputs={source:{name:'captured-engine.json',text:JSON.stringify({graph,options:{...options,districts:2}})},settings:{label:'Native live snapshots',chamber:'congressional',created_at:'2026-10-18T00:00:00Z',inspect_only:false}};
const review=engine.execute({operation:'run-prepared-engine',input:{request_json:inputs.source.text,...inputs.settings,inspect_only:true}}),job=await preparedStateJobFromResult(inputs,review),jobEvents=[];
const output=executePreparedStateJob(engine,job,{onProgress:p=>jobEvents.push(p)});check(jobEvents.filter(p=>p.schema_version==='bisect-native-round-progress-v1'),output.prepared.result,graph);
await fs.writeFile(root+'/prepared.bisect',JSON.stringify(createProject({name:'Live native round progress',files:{preparedEngineInputs:inputs},operation:'run-prepared-engine',constraints:[],result:null,lastOperation:null})));
await fs.writeFile(root+'/job.bisect',JSON.stringify(createProject({name:'Live native prepared state',files:{stateRunBatchInputs:{workers:1,jobs:{name:'one-job.json',base64:Buffer.from(JSON.stringify([job])).toString('base64')}},stateBatchInputs:{year:'2020',states:['RI'],manifest:null}},operation:'run-state-batch',constraints:[],result:null,lastOperation:null})));
if(process.argv.includes('--prepare')){console.log('PASS WASM progress comparisons and source fixtures prepared; native comparison deliberately deferred until the executable build finishes.');process.exit(0);}
// Native executable is invoked only after its build is terminal.
for(const [i,r]of records.entries()){
 const request=r.request.operation?r.request:{operation:'run-prepared-engine',input:{request_json:JSON.stringify(r.request),label:'Progress parity',chamber:'congressional',created_at:'2026-10-18T00:00:00Z',inspect_only:false}};
 const file=root+'/native-'+i+'.request.json';await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(r.result,r.request.operation?JSON.parse(native.stdout):JSON.parse(native.stdout).result);
}
await fs.writeFile(root+'/records.json',JSON.stringify(records));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,previous_wasm:before.artifactIdentity,native_reference_sha256:createHash('sha256').update(await fs.readFile('target/release/examples/execute_tool_request.exe')).digest('hex'),whole_native_matches:records.length,unchanged_previous_replies:cases,notifications,per_capture_index_reset:true,failed_build_observations:retryEvents.length,host_failure_refuses_result_and_cleans_up:true,prepared_state_forwarding:true,published:false},null,2)+'\n');console.log(`PASS native round progress: ${records.length} whole native replies, ${cases} unchanged old replies, ${notifications} exact native snapshot notifications, host failure/reentry/cleanup and retry-local indices`);
