import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preparedEngineRequest} from '../../web/lab/prepared-engine-files.js';
import {preparedStateJobFromResult,executePreparedStateJob} from '../../web/lab/prepared-state-job.js';
import {runStateBatch,appendStateRunJob,stateRunJobs,stateRunBatchZip,canResumeStateBatch} from '../../web/lab/state-run-batch.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/state-run-years');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8')),years=['2020','2010','2000'];
const source=text=>({name:'synthetic-multiyear-jobs.json',base64:Buffer.from(text).toString('base64')});
const mapFixture=JSON.parse(await fs.readFile('target/prepared-engine-flow/flow-0.bisect','utf8')),mapRequest=JSON.parse(mapFixture.files.preparedEngineInputs.source.text);
const jobs=[];let nativeMatches=0;
for(const year of years){
 const rows=engine.execute({operation:'inspect-state-batch-selection',input:{year,states:[],manifest_json:null}}).states;
 assert.equal(rows.length,50);
 for(const row of rows){
  const n=row.districts*4,fips=String(manifest.states[row.state_code].fips).padStart(2,'0');
  let graph={schema_version:1,state:row.state_code,year,geoids:Array.from({length:n},(_,i)=>fips+'001'+String(i).padStart(6,'0')),adjacency:Array.from({length:n},(_,i)=>[(i+n-1)%n,(i+1)%n]),population:Array(n).fill(100),edges:Array.from({length:n},(_,i)=>[Math.min(i,(i+1)%n),Math.max(i,(i+1)%n),10]),areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(100),centroids:Array.from({length:n},(_,i)=>[-100+i*.001,40])};
  if(row.state_code==='RI')graph={...structuredClone(mapRequest.graph),year};
  const options={structure:'standard-bisect',weights:'unweighted',search:'single',districts:row.districts,seed:'42',seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
  const inputs={source:{name:'SYNTHETIC-'+row.state_code+'-'+year+'.json',text:JSON.stringify({graph,options})},settings:{inspect_only:true,label:'Synthetic_'+row.state_code+'_'+year,chamber:'congressional',created_at:'2026-10-18T00:00:00Z'}};
  const review=engine.execute(preparedEngineRequest(inputs));let geometry;if(row.state_code==='RI'){const value=JSON.parse(Buffer.from(mapFixture.files.planGeometry.base64,'base64'));value.year=year;geometry={name:'SYNTHETIC-RI-'+year+'.geojson',base64:Buffer.from(JSON.stringify(value)).toString('base64')};}jobs.push(await preparedStateJobFromResult(inputs,review,geometry));
 }
}
const inputs={jobs:source(JSON.stringify(jobs)),workers:3,all_years:true},selection={year:'2020',states:[],manifest:null};
assert.equal(stateRunJobs(inputs.jobs,153).length,150);assert.throws(()=>stateRunJobs(inputs.jobs),/51/);
let alive=0,maxAlive=0,partitions=0,initializations=0,mismatchAt=0;const selections=[];
class WorkerAdapter extends EventTarget{
 constructor(){super();this.closed=false;alive++;maxAlive=Math.max(maxAlive,alive);}
 postMessage(message){setTimeout(()=>{
  if(this.closed)return;
  try{
   if(message.type==='initialize'){
    initializations++;const artifact=mismatchAt===initializations?{...engine.artifactIdentity,sha256:'0'.repeat(64)}:engine.artifactIdentity;
    this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:artifact}}));return;
   }
   const request=message.request;
   if(request.operation==='inspect-state-batch-selection')selections.push(request.input.year);
   const result=request.operation==='run-prepared-state-job'?executePreparedStateJob({execute:r=>{if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)partitions++;return engine.execute(r);}},request.input):engine.execute(request);
   this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
  }catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}
 },0);}
 terminate(){if(!this.closed){alive--;this.closed=true;}}
}
const execute=(data=inputs,scope=selection,options={})=>runStateBatch(data,scope,{createWorker:()=>new WorkerAdapter(),hardwareConcurrency:4,wasmUrl:'local',...options});
const checkpoints=[],result=await execute(inputs,selection,{onCheckpoint:c=>checkpoints.push(c)});
assert.deepEqual(result.requested_years,years);assert.deepEqual(result.years.map(r=>r.year),years);assert.deepEqual(selections,years);
assert.equal(result.summary.completed,3);assert.equal(result.exit_code,0);assert.equal(result.state_summary.succeeded,150);assert.equal(alive,0);assert.equal(maxAlive,3);
const oracleFile=path.join(root,'native.json');
for(const row of result.years)for(const state of row.result.states){
 const job=jobs.find(j=>j.scope.year===row.year&&j.scope.state===state.state_code);
 await fs.writeFile(oracleFile,JSON.stringify(preparedEngineRequest(job.prepared_engine)));
 const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[oracleFile],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});
 assert.equal(native.status,0,native.stderr);assert.deepEqual(state.result.prepared,JSON.parse(native.stdout));nativeMatches++;
}
await fs.writeFile(path.join(root,'all-years.zip'),await stateRunBatchZip(result));
const riJobs=jobs.filter(j=>j.scope.state==='RI'),riInputs={...inputs,jobs:source(JSON.stringify(riJobs))},riSelection={...selection,states:['RI']};
const missing=await execute({...riInputs,jobs:source(JSON.stringify(riJobs.filter(j=>j.scope.year!=='2010')))},riSelection);
assert.deepEqual(missing.years.map(r=>r.status),['completed','failed','completed']);assert.equal(missing.exit_code,1);assert.match(missing.years[1].error,/RI\/2010/);
const partial=await execute(riInputs,{...riSelection,states:['RI','VT']});
assert.equal(partial.state_summary.succeeded,3);assert.equal(partial.state_summary.failed,3);assert.equal(partial.exit_code,1);
for(const row of partial.years){const absent=row.result.states.find(s=>s.state_code==='VT');assert.equal(absent.status,'failed');assert.equal(absent.execution_performed,false);assert.equal(row.result.states.find(s=>s.state_code==='RI').status,'ok');}
await fs.writeFile(path.join(root,'missing-jobs.json'),JSON.stringify(partial));
const beforeEmpty=partitions,empty=await execute({...riInputs,jobs:source('[]')},riSelection);
assert.equal(empty.execution_performed,false);assert.equal(empty.state_summary.failed,3);assert.equal(partitions,beforeEmpty);
const controller=new AbortController();
const cancelled=await execute(riInputs,riSelection,{signal:controller.signal,onCheckpoint:c=>{if(c.years[0]?.result?.summary.completed)controller.abort(new Error('Test cancellation'));}});
assert.equal(cancelled.status,'cancelled');assert.equal(cancelled.years.length,1);assert.equal(cancelled.state_summary.succeeded,1);assert.equal(alive,0);assert.equal(canResumeStateBatch(cancelled),true);
await fs.writeFile(path.join(root,'cancelled.zip'),await stateRunBatchZip(cancelled));
await assert.rejects(()=>stateRunBatchZip({...result,years:result.years.slice(1)}),/all three years/);
await assert.rejects(()=>stateRunBatchZip({...result,years:[...result.years].reverse()}),/scope/);
initializations=0;mismatchAt=2;const before=partitions,changed=await execute({...riInputs,workers:1},riSelection);
assert.equal(changed.status,'interrupted');assert.equal(changed.years.length,2);assert.equal(partitions-before,1);assert.equal(alive,0);mismatchAt=0;
await assert.rejects(()=>execute(riInputs,riSelection,{resume:result}),/unchanged checkpoint/);
const duplicate=source(JSON.stringify([...riJobs,riJobs[0]]));assert.throws(()=>stateRunJobs(duplicate,153),/distinct/);
const packet=createProject({name:'SYNTHETIC all-state/all-vintage prepared engine Run',operation:'run-state-batch',lastOperation:'run-state-batch',constraints:[],files:{stateRunBatchInputs:inputs,stateBatchInputs:selection},result});
assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,'all-years.bisect'),JSON.stringify(packet));
const ri=await execute(riInputs,riSelection);
await fs.writeFile(path.join(root,'browser.bisect'),JSON.stringify(createProject({name:'SYNTHETIC Rhode Island three-year Run',operation:'run-state-batch',lastOperation:'run-state-batch',constraints:[],files:{stateRunBatchInputs:riInputs,stateBatchInputs:riSelection},result:ri})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_host_sha256:createHash('sha256').update(await fs.readFile('target/release/examples/execute_tool_request.exe')).digest('hex'),native_host_matches:nativeMatches,synthetic_state_year_jobs:150,native_order:true,native_congressional_allocations:true,failed_year_continues:true,cancellation_retains_received_outputs:true,changed_artifact_refused_before_next_year_partition:true,workers_closed:true,project_roundtrip:true,scope:'Synthetic supplied graphs for all50 states/all3 native vintages, with actual WASM execution and full native prepared-engine receipt comparisons. Original historical source acquisition and statewide original-data capacities are not proved.'},null,2)+'\n');
console.log('Multi-year Run:150 synthetic state/year native receipt matches; native allocations/order, per-year failure continuation, cancellation retention, same-artifact gates, worker limits, source projects and ZIPs passed.');
