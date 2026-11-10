import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preflightStateSourceFlags} from '../../web/lab/state-source-preflight.js';
import {acquireStateSourceJob,acquireStateSourceBatch} from '../../web/lab/state-source-acquisition.js';
import {createStateWorkflowBinding,stateWorkflowSettingsPatch} from '../../web/lab/native-state-workflow.js';
import {stateCommandRequest} from '../../web/lab/state-command-files.js';
const root='target/state-source-preflight';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let hostMatches=0,fetches=0,workers=0,terminations=0;const operations=[];
const inspectNativeCommand=request=>{assert.equal(request.operation,'inspect-native-state-command');operations.push(request.operation);const result=engine.execute(request);writeFileSync(root+'/request.json',JSON.stringify(request));const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024}));compareNative(request,result,host);hostMatches++;return result;};
const command=extra=>({arguments_json:JSON.stringify(['--state','RI','--metis-engine','metis-core','--search','single','--seed','18446744073709551615',...extra])});
const files={nativeStateSourceSettings:{enabled:true},stateCommandInputs:command([]),stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Preflight',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}}};
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]}});
const options={inspectNativeCommand,retryAttempts:1,fetcher:async()=>{fetches++;return new Response('missing',{status:404});}},scope={state:'RI',year:'2020'};
for(const args of [['--print-only'],['--reset'],['--force'],['--debug'],['--time-partition'],['--version','vNoHandoff'],['--adjacency','never-open.adj.bin']]){
 const selected={...files,stateCommandInputs:command(args)},before=fetches;
 await assert.rejects(()=>acquireStateSourceJob(plan,scope,selected,null,null,null,options),/before source acquisition/);
 let checkpoints=0;await assert.rejects(()=>acquireStateSourceBatch(plan,'2020',{jobs:null,workers:1},selected,null,null,null,{...options,onCheckpoint:()=>checkpoints++}),/before source acquisition/);
 assert.equal(fetches,before);assert.equal(checkpoints,0);
}
const original=command(['--version','vPreflight','--label','Bound','--dpi','72','--output-dir','never/open/path']);
const binding=await createStateWorkflowBinding(original,inspectNativeCommand(stateCommandRequest(original))),bound={...files,...stateWorkflowSettingsPatch(binding,files)};
const cached=await preflightStateSourceFlags(bound,{inspectNativeCommand}),requestsBefore=hostMatches;
await preflightStateSourceFlags(bound,{inspectNativeCommand:cached});assert.equal(hostMatches,requestsBefore);
const request=stateCommandRequest(binding.source_command),reply=cached(request);reply.fields=[];assert(cached(request).fields.length>0);
assert.throws(()=>cached(stateCommandRequest(command(['--reset']))),/changed after source preflight/);
const corrupted=structuredClone(bound);corrupted.stateWorkflowBinding.settings.output_dir_intent='forged';const beforeCorrupt=fetches;
await assert.rejects(()=>acquireStateSourceJob(plan,scope,corrupted,null,null,null,options),/settings or original flags changed/);assert.equal(fetches,beforeCorrupt);
const dryOriginal=command(['--manifest','--print-only']),dryBinding=await createStateWorkflowBinding(dryOriginal,inspectNativeCommand(stateCommandRequest(dryOriginal)));
await assert.rejects(()=>acquireStateSourceJob(plan,scope,{...files,...stateWorkflowSettingsPatch(dryBinding,files)},null,null,null,options),/Remaining State flags/);assert.equal(fetches,beforeCorrupt);
await assert.rejects(()=>acquireStateSourceJob(plan,scope,bound,null,null,null,options),/unavailable|missing|404/);assert(fetches>beforeCorrupt);
const beforeBatchHosts=hostMatches,beforeBatchFetch=fetches;
const batch=await acquireStateSourceBatch(plan,'2020',{jobs:null,workers:1},bound,null,null,null,options);assert.equal(batch.summary.failed,2);assert.equal(batch.summary.queued,0);assert.equal(hostMatches-beforeBatchHosts,2);assert(fetches>beforeBatchFetch);
class InspectionWorker extends EventTarget{
 constructor(mode='good'){super();workers++;this.mode=mode;this.dead=false;}
 postMessage(message){queueMicrotask(()=>{if(this.dead||this.mode==='hold')return;try{let data;if(message.type==='initialize'){if(this.mode==='initialize-error')throw new Error('Owned initialization failure');data={type:'ready',runtimeArtifact:this.mode==='wrong-artifact'?{...engine.artifactIdentity,byte_length:1}:engine.artifactIdentity};}else data={type:'result',id:message.id,result:inspectNativeCommand(message.request)};this.dispatchEvent(new MessageEvent('message',{data}));}catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}});}
 terminate(){assert.equal(this.dead,false);this.dead=true;terminations++;}
}
await preflightStateSourceFlags(files,{createInspectionWorker:()=>new InspectionWorker(),expectedArtifact:engine.artifactIdentity});
await assert.rejects(()=>preflightStateSourceFlags(files,{createInspectionWorker:()=>new InspectionWorker('wrong-artifact'),expectedArtifact:engine.artifactIdentity}),/selected WASM artifact/);
await assert.rejects(()=>preflightStateSourceFlags(files,{createInspectionWorker:()=>new InspectionWorker('initialize-error')}),/Owned initialization failure/);
const controller=new AbortController(),pending=preflightStateSourceFlags(files,{createInspectionWorker:()=>new InspectionWorker('hold'),signal:controller.signal});controller.abort(new Error('Owned cancellation'));await assert.rejects(pending,/Owned cancellation/);
assert.equal(workers,terminations);
const disabled=await preflightStateSourceFlags({...files,nativeStateSourceSettings:{enabled:false}},{createInspectionWorker:()=>{throw Error('Must stay inert');}});assert.equal(disabled,undefined);
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,whole_native_host_replies:hostMatches,unsupported_flags_refused_before_fetch:true,forged_binding_refused_before_fetch:true,bulk_preflight_before_checkpoints:true,valid_binding_reaches_explicit_fetch:true,private_snapshot_cache_preserves_exact_requests:true,worker_artifact_mismatch_init_error_and_cancel_terminate:true,workers,terminations,partition_runs:0,published:false},null,2));
console.log('PASS source preflight: '+hostMatches+' complete native/WASM inspection matches; single/bulk refusal before fetch/checkpoints, bound settings, immutable cache, artifact binding and worker cleanup; no partitions.');
