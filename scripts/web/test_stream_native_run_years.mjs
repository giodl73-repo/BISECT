import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator,strictComparisonPairs=0;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);strictComparisonPairs++;}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNativeStateOutput} from '../../web/lab/native-state-output.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {streamNativeRunYears} from '../../web/lab/stream-native-run-years.js';
import {createLocalStateYearsOutput} from '../../web/lab/local-state-output.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/stream-native-run-years');await fs.mkdir(root,{recursive:true});
execFileSync('python',['scripts/web/prepare_native_run_years_fixtures.py'],{stdio:'inherit',windowsHide:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const planInput={manifest_json:null,states:['VT'],year:'all',data_types:[],selected_paths:[]};
const plan=engine.execute({operation:'plan-fetch-sources',input:planInput});
const files={nativeStateSourceSettings:{enabled:true,command_kind:'run'},runCommandInputs:{arguments_json:JSON.stringify(['--year','all','--states','VT','--workers','0','--version','vYears'])},stateGraphInputs:{graph:null,auxiliary:null,settings:{inspect_only:true,label:'Synthetic VT three-vintage stream',chamber:'congressional',created_at:'2026-10-20T00:00:00Z'}}};
const selection={year:'2020',states:['VT'],manifest:null};
const {directory}=nativeOutputDirectoryHarness(root);let alive=0,matches=0,downloads=0,cuts=0,preparations=0,initializations=0,mismatchAt=0,failYear=null;

const facade={execute(r,o){if(r.operation==='prepare-census-engine-input')preparations++;if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)cuts++;const v=engine.execute(r,o);const file=root+'/native.json';writeFileSync(file,JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));compareNative(r,v,native);matches++;return v;}};
class Worker extends EventTarget{
 constructor(){super();alive++;}
 postMessage(m){setTimeout(async()=>{if(this.closed)return;try{if(m.type==='initialize'){initializations++;this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:initializations===mismatchAt?{...engine.artifactIdentity,sha256:'0'.repeat(64)}:engine.artifactIdentity}}));return;}const result=m.request.operation==='run-native-state-job'?await executeNativeStateJob(facade,m.request.input):m.request.operation==='prepare-native-state-output'?await executeNativeStateOutput(facade,m.request.input):facade.execute(m.request);if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:m.id,result}}));}catch(e){if(!this.closed)this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}},1);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
const fetcher=async url=>{downloads++;const item=plan.items.find(i=>i.url===url);assert(item);if(item.year===failYear)return new Response('missing',{status:404});return new Response(await fs.readFile(root+'/fixtures/'+item.year+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));};
const archiveReader=async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options);
async function run({source=files,sourcePlan=plan,signal,onCheckpoint=()=>{},nativeOutputSettings={enabled:true,version:'vYears',layout:'state'},directoryOptions={}}={}){
 const parent=await directory('run',directoryOptions),sink=await createLocalStateYearsOutput(parent,{nativeOutputSettings});
 const result=await streamNativeRunYears(sourcePlan,selection,source,null,null,null,sink,{createWorker:()=>new Worker(),hardwareConcurrency:2,wasmUrl:'local',fetcher,archiveReader,signal,onCheckpoint});assert.equal(alive,0);return {result,base:path.join(parent.dir,sink.name)};
}
const good=await run();assert.equal(good.result.exit_code,0,JSON.stringify(good.result));assert.equal(good.result.state_summary.succeeded,3,JSON.stringify(good.result));assert.equal(downloads,6);assert.equal(preparations,3);assert.equal(cuts,3);assert.deepEqual(good.result.native_output_summary,{completed:3,failed:0});assert.deepEqual(good.result.years.map(r=>r.year),['2020','2010','2000']);
assert(good.result.years.every(row=>row.status==='completed'));
for(const row of good.result.years.filter(r=>r.result)){const child=row.result;assert.equal(child.runtime_artifact.sha256,good.result.runtime_artifact.sha256);assert.equal(child.scheduling.requested_workers,'0');assert.equal(child.scheduling.effective_workers,1);const base=path.join(good.base,row.year,child.output_folder);const marker=JSON.parse(await fs.readFile(base+'/VT/completed.json','utf8'));assert.equal(marker.native_output.status,'completed');const jobs=JSON.parse(await fs.readFile(base+'/VT/state-run-jobs.json','utf8'));assert.equal(jobs[0].scope.year,row.year);assert.equal(jobs[0].native_state.command_kind,'run');assert.equal(jobs[0].native_state.command.arguments_json,files.runCommandInputs.arguments_json);}
assert.deepEqual(JSON.parse(await fs.readFile(good.base+'/run-years-receipt.json','utf8')),good.result);
for(const flags of [['--year','2020'],['--year','all','--print-only'],['--year','all','--states','WY']]){const source=structuredClone(files);source.runCommandInputs.arguments_json=JSON.stringify(flags);const before={downloads,cuts,preparations};await assert.rejects(()=>run({source}));assert.deepEqual({downloads,cuts,preparations},before);assert.equal(alive,0);}
const absent=structuredClone(plan);absent.items=absent.items.filter(i=>i.year!=='2000');const beforeAbsent=downloads;const unsupported=await run({sourcePlan:absent});assert.equal(unsupported.result.years[2].status,'failed');assert.equal(downloads-beforeAbsent,4);
const beforeDisk=downloads;await assert.rejects(()=>run({directoryOptions:{corrupt:p=>p.endsWith('run-years-receipt.json')}}),/Saved output verification failed/);assert.equal(downloads,beforeDisk);assert.equal(alive,0);
const beforeVersion=downloads;await assert.rejects(()=>run({nativeOutputSettings:{enabled:true,version:'vWrong',layout:'state'}}),/version/);assert.equal(downloads,beforeVersion);
failYear='2010';const continued=await run();failYear=null;assert.equal(continued.result.exit_code,1);assert.deepEqual(continued.result.years.map(r=>r.status),['completed','failed','completed']);assert.equal(continued.result.state_summary.succeeded,2);
const controller=new AbortController(),cancelled=await run({signal:controller.signal,onCheckpoint:r=>{if(r.state_summary.succeeded===1)controller.abort();}});assert.equal(cancelled.result.status,'cancelled');assert.equal(cancelled.result.state_summary.succeeded,1);assert.equal(cancelled.result.years.length,1);
mismatchAt=initializations+4;const beforeMismatch=downloads;const mismatch=await run();mismatchAt=0;assert.equal(mismatch.result.status,'interrupted');assert.equal(mismatch.result.years.length,2);assert.equal(downloads-beforeMismatch,2);assert.equal(mismatch.result.state_summary.succeeded,1);assert.equal(alive,0);
const project=createProject({name:'Native Run acquire and stream all vintages',files:{...files,stateBatchInputs:selection,streamNativeOutputSettings:{enabled:true,version:'vYears',layout:'state'},fetchPlanInputs:{source:{name:'all-vintages.json',base64:Buffer.from(JSON.stringify(planInput)).toString('base64')}}},operation:'plan-fetch-sources',constraints:[],result:null,lastOperation:null});assert.equal(parseProject(JSON.stringify(project)).operation,'plan-fetch-sources');await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
await fs.writeFile(root+'/accepted.json',JSON.stringify(good));await fs.writeFile(root+'/evidence.json',JSON.stringify({strict_comparison_pairs:strictComparisonPairs,wasm:engine.artifactIdentity,synthetic_VT_sources:true,primary_runs:3,whole_native_wasm_replies:matches,original_zip_pairs_saved:3,native_output_installations:3,missing_year_recorded_without_acquisition:true,failed_year_continues:true,cancellation_retains_outputs:true,cross_year_artifact_pin:true,workers_closed:true,published:false},null,2));console.log(`PASS native Run all-vintage acquisition/streaming: ${matches} native replies, 2020/2010/2000 Census pairs and missing-pair refusal, native output installation, failed-year continuation, cancellation and artifact pin.`);
