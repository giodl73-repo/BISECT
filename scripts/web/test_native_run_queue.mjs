import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createNativeStateSourceJob} from '../../web/lab/native-state-sources.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {appendStateRunJob,prepareNativeRunQueue,runStateBatch,stateRunBatchZip,canResumeStateBatch} from '../../web/lab/state-run-batch.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/native-run-queue';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8'));
const command=(year='all',workers=100,states=['VT','WY'],extra=[])=>({arguments_json:JSON.stringify(['--year',year,'--workers',String(workers),'--states',...states,'--version','vRunQueue','--output-dir','retained output path',...extra])});
const inspect=c=>engine.execute({operation:'inspect-native-run-command',arguments:JSON.parse(c.arguments_json)});
let inputs={jobs:null,workers:1,all_years:true};
for(const year of ['2020','2010','2000'])for(const [state,fips] of [['VT','50'],['WY','56']]){
 const files=structuredClone(fixture);files.nativeAdjacencySettings={state,year,unit_kind:'tract'};
 const geoids=JSON.parse(Buffer.from(files.nativeAdjacencyInputs.geoids.base64,'base64'));
 for(const k of Object.keys(geoids))geoids[k]=fips+geoids[k].slice(2);
 files.nativeAdjacencyInputs.geoids.base64=Buffer.from(JSON.stringify(geoids)).toString('base64');
 files.nativeStateSourceSettings={enabled:true,command_kind:'run'};files.runCommandInputs=command();
 files.stateGraphInputs={graph:null,auxiliary:null,settings:{inspect_only:true,label:'Synthetic native Run source',chamber:'congressional',created_at:'2026-10-20T00:00:00Z'}};
 inputs=appendStateRunJob(inputs,await createNativeStateSourceJob(files,'native'));
}
const selection={year:'2020',states:['VT'],manifest:null},original=structuredClone(inputs);
const selected=await prepareNativeRunQueue(command(),inputs,selection,inspect(command()));assert.deepEqual(inputs,original);assert.equal(selected.inputs.all_years,true);assert.deepEqual(selected.selection.states,['VT','WY']);
let alive=0,peak=0,prepared=0,cuts=0,matches=0;
const facade={artifactIdentity:engine.artifactIdentity,execute(r,o){if(r.operation==='prepare-native-engine-input')prepared++;if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)cuts++;const reply=engine.execute(r,o);const file=root+'/native.json';writeFileSync(file,JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));compareNative(r,reply,native);matches++;return reply;}};
class Worker extends EventTarget{
 constructor(){super();alive++;peak=Math.max(peak,alive);}
 postMessage(m){setTimeout(async()=>{if(this.closed)return;try{if(m.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:engine.artifactIdentity}}));return;}const result=m.request.operation==='run-native-state-job'?await executeNativeStateJob(facade,m.request.input):facade.execute(m.request);if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:m.id,result}}));}catch(e){if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}},1);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
const run=(value=selected,options={})=>runStateBatch(value.inputs,value.selection,{createWorker:()=>new Worker(),wasmUrl:'local',hardwareConcurrency:2,...options});
const result=await run();assert.equal(result.exit_code,0,JSON.stringify(result.years.map(y=>y.error)));assert.equal(result.state_summary.succeeded,6);assert.deepEqual(result.years.map(y=>y.year),['2020','2010','2000']);assert.equal(alive,0);assert.equal(peak,2);assert.equal(cuts,6);
for(const y of result.years){assert.equal(y.result.workers.requested,100);assert.equal(y.result.workers.effective,2);for(const row of y.result.states){assert.equal(row.result.native_run.recipe.command.arguments_json,command().arguments_json);assert.equal(row.source_preparation.year,y.year);assert.equal(row.result.scope.districts,1);}}
await fs.writeFile(root+'/accepted.json',JSON.stringify(result));await fs.writeFile(root+'/accepted.zip',await stateRunBatchZip(result));
assert.equal((await run(selected,{verifySaved:JSON.parse(JSON.stringify(result))})).state_summary.succeeded,6);
const forged=structuredClone(result);forged.years[0].result.native_run_command.workers='1';const beforeForge=cuts;const rejected=await run(selected,{verifySaved:forged});assert.equal(rejected.status,'interrupted');assert.match(rejected.years[0].error,/policy changed/);assert.equal(cuts,beforeForge);
const singleInputs={...inputs,all_years:false,jobs:{name:'single.json',base64:Buffer.from(JSON.stringify(JSON.parse(Buffer.from(inputs.jobs.base64,'base64')).filter(j=>j.scope.year==='2020'))).toString('base64')}};
const single=await prepareNativeRunQueue(command('2020',0),singleInputs,selection,inspect(command('2020',0)));const one=await run(single);assert.equal(one.summary.succeeded,2);assert.equal(one.workers.effective,2);
const controller=new AbortController();let partial;await assert.rejects(()=>run(single,{hardwareConcurrency:1,signal:controller.signal,onCheckpoint:c=>{partial=c;if(c.summary.succeeded===1)controller.abort();}}));partial.status='cancelled';assert(canResumeStateBatch(partial));assert.equal((await run(single,{resume:partial})).summary.succeeded,2);
for(const c of [command('2020',1,['VT'],['--print-only']),command('2010'),command('2020',1,['VT'])]){
 const invalid=structuredClone(single);invalid.inputs.run_command=c;const before={prepared,cuts};await assert.rejects(()=>run(invalid));assert.deepEqual({prepared,cuts},before);assert.equal(alive,0);
}
const damaged=structuredClone(inputs),jobs=JSON.parse(Buffer.from(damaged.jobs.base64,'base64'));jobs[0].native_state.source_inputs.settings.label='altered';damaged.jobs.base64=Buffer.from(JSON.stringify(jobs)).toString('base64');await assert.rejects(()=>prepareNativeRunQueue(command(),damaged,selection,inspect(command())),/identity changed/);
const wrong=structuredClone(selected);wrong.inputs.run_command=command('2020');const beforeWrong={prepared,cuts};const failed=await run(wrong);assert.equal(failed.exit_code,1);assert.equal(failed.years.length,3);assert.deepEqual({prepared,cuts},beforeWrong);
const yearAbort=new AbortController();let interrupted;const cancelled=await run(selected,{signal:yearAbort.signal,onCheckpoint:c=>{interrupted=c;if(c.state_summary.succeeded>=2)yearAbort.abort();}});assert.equal(cancelled.status,'cancelled');assert(canResumeStateBatch(cancelled));const beforeResume=cuts;const resumed=await run(selected,{resume:cancelled});assert.equal(resumed.state_summary.succeeded,6);assert.equal(cuts-beforeResume,4);
const raw={arguments_json:JSON.stringify(['--year','2020','--states=VT --help'])};const rawQueue=await prepareNativeRunQueue(raw,singleInputs,selection,inspect(raw));const filtered=await run(rawQueue);assert.deepEqual(filtered.states.map(r=>r.state_code),['VT']);assert.deepEqual(filtered.native_run_command.states,['VT','--help']);
const unknown={arguments_json:JSON.stringify(['--year','2020','--states=--help'])};const unknownQueue=await prepareNativeRunQueue(unknown,singleInputs,selection,inspect(unknown));const beforeUnknown={prepared,cuts};try{const empty=await run(unknownQueue);assert.equal(empty.summary.total,0);}catch(e){assert.match(e.message,/unknown|state|filter/i);}assert.deepEqual({prepared,cuts},beforeUnknown);
const missing=structuredClone(selected),remaining=JSON.parse(Buffer.from(missing.inputs.jobs.base64,'base64')).filter(j=>j.scope.year!=='2010');missing.inputs.jobs.base64=Buffer.from(JSON.stringify(remaining)).toString('base64');const continued=await run(missing);assert.deepEqual(continued.years.map(y=>y.status),['completed','failed','completed']);
const project=createProject({name:'Native Run original sources across three vintages',files:{runCommandInputs:command(),stateRunBatchInputs:inputs,stateBatchInputs:selection},operation:'inspect-native-run-command',constraints:[],result:null,lastOperation:null});assert.equal(parseProject(JSON.stringify(project)).operation,'inspect-native-run-command');await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,synthetic_binary_sources:true,native_year_order:['2020','2010','2000'],primary_runs:6,whole_native_wasm_replies:matches,source_hash_preflight:true,saved_full_replay:true,live_resume:true,failed_year_continues:true,workers_zero_auto:true,published:false},null,2));
console.log(`PASS native Run source queue: six state/vintage Runs, ${matches} whole native replies, saved replay, live Resume, zero-auto workers and preflight guards.`);
