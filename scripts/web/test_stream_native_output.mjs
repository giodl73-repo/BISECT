import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeStateSourceJob} from '../../web/lab/state-source-job.js';
import {executePreparedCensusJob} from '../../web/lab/prepared-census-job.js';
import {executeNativeStateJob} from '../../web/lab/native-state-job.js';
import {executeNativeStateOutput,validateStreamNativeOutputSettings} from '../../web/lab/native-state-output.js';
import {createLocalStateOutput} from '../../web/lab/local-state-output.js';
import {streamStateSourceBatch} from '../../web/lab/stream-state-source-batch.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {nativeOutputDirectoryHarness} from './fixtures/native-output-directory.mjs';
const root=path.resolve('target/stream-native-output');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const {directory,log}=nativeOutputDirectoryHarness(root);
const settings={enabled:true,version:'vStream',layout:'state'};
const files={nativeStateSourceSettings:{enabled:true},stateCommandInputs:{arguments_json:JSON.stringify(['--state','IA','--year','2000','--metis-engine','metis-core','--search','single','--seed','18446744073709551615','--niter','10','--balance-tolerance','5'])},stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Stream_native_run',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}},tuiRunWorkflowSettings:{analysis:true,map:false,report:false,dpi:'72'},streamNativeOutputSettings:settings};
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));for(const state of Object.values(manifest.states))state.districts['2020']=1;
const selection={year:'2020',states:['RI','VT'],manifest:{name:'one-seat-fixture.json',base64:Buffer.from(JSON.stringify(manifest)).toString('base64')}};
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]}});
const fetcher=async url=>{const item=plan.items.find(i=>i.url===url);return new Response(await fs.readFile('target/state-source-acquisition/synthetic-states/'+item.state_code+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));};
const archiveReader=async(file,options)=>extractCensusZip(new Uint8Array(await file.arrayBuffer()),options);
let alive=0,peak=0,partitions=0,preparations=0,native=0,mode='good',compare=false;
const facade={artifactIdentity:engine.artifactIdentity,execute(request,options){
 if(request.operation==='run-prepared-engine'&&request.input.inspect_only===false||request.operation==='run-tui-configured')partitions++;
 return engine.execute(request,options);
}};
class Worker extends EventTarget{
 constructor(){super();this.closed=false;alive++;peak=Math.max(peak,alive);}
 postMessage(message){setTimeout(async()=>{if(this.closed)return;try{
  if(message.type==='initialize'){this.dispatchEvent(new MessageEvent('message',{data:{type:'ready',runtimeArtifact:engine.artifactIdentity}}));return;}
  const request=message.request;let result;
  if(request.operation==='run-native-state-job'){this.state=request.input.scope.state;result=await executeNativeStateJob(facade,request.input);}
  else if(request.operation==='run-census-state-job')result=executeStateSourceJob(facade,request.input);
  else if(request.operation==='run-census-prepared-state-job')result=executePreparedCensusJob(facade,request.input);
  else if(request.operation==='prepare-native-state-output'){
   preparations++;const before=partitions;if(mode==='prepare-failure'&&this.state==='RI')throw Error('Synthetic output preparation failure');
   const packets=[],outputEngine={artifactIdentity:engine.artifactIdentity,execute(r,o){packets.push(r);return facade.execute(r,o);}};
   result=await executeNativeStateOutput(outputEngine,request.input);assert.equal(partitions,before);
   if(mode==='tamper'&&this.state==='RI')result.source_sha256='0'.repeat(64);
   if(compare)for(const packet of packets){const file=path.join(root,'native-'+native+'.json');await fs.writeFile(file,JSON.stringify(packet));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024})),engine.execute(packet));native++;}
  }else{result=facade.execute(request);if(compare&&['inspect-native-state-run-target','inspect-native-run-target'].includes(request.operation)){const file=path.join(root,'native-'+native+'.json');await fs.writeFile(file,JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8'})),result);native++;}}
  this.dispatchEvent(new MessageEvent('message',{data:{type:'result',id:message.id,result}}));
 }catch(error){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:message.id,error:error.message}}));}},1);}
 terminate(){if(!this.closed){this.closed=true;alive--;}}
}
async function run({options={},nativeSettings=settings,selected=selection,selectedPlan=plan,execution={files,runSettings:null,form:null,workflow:null},...hooks}={}){
 const parent=await directory('run',options),sink=await createLocalStateOutput(parent,{nativeOutputSettings:nativeSettings}),before=partitions;
 const receipt=await streamStateSourceBatch(selectedPlan,'2020',selected,execution.files,execution.runSettings,execution.form,execution.workflow,sink,{createWorker:()=>new Worker(),wasmUrl:'local',fetcher,archiveReader,...hooks});
 assert.equal(alive,0);if(receipt.status==='completed')assert.equal(partitions-before,receipt.states.filter(r=>r.execution_performed).length);return {receipt,parent:parent.dir,base:path.join(parent.dir,sink.name)};
}
compare=true;const good=await run();compare=false;
assert.deepEqual(good.receipt.native_output_summary,{completed:2,failed:0});assert.equal(native,8);assert.equal(peak,1);
for(const row of good.receipt.states){
 const marker=JSON.parse(await fs.readFile(path.join(good.base,row.state,'completed.json'),'utf8'));assert.equal(marker.native_output.status,'completed');assert.equal(marker.effective_seed,'18446744073709551615');
 const target=path.join(good.base,row.native_output.target_namespace);await assert.rejects(fs.stat(path.join(target,'manifest.tmp')));assert((await fs.readFile(path.join(target,'data/final_assignments.json'),'utf8')).length>2);
 const source=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile(path.join(good.base,row.state,'source-project.bisect-workbench'))));assert.deepEqual(source.files.streamNativeOutputSettings,settings);assert.equal(source.files.stateCommandInputs.arguments_json,files.stateCommandInputs.arguments_json);assert(source.files.tuiRunWorkflowSettings.analysis);assert(marker.followup);
 const install=JSON.parse(await fs.readFile(path.join(target,'browser/install-receipt.json'),'utf8'));for(const entry of install.files){const data=await fs.readFile(path.join(target,entry.path));assert.equal(data.length,entry.byte_length);}
}
const paths=[];async function collect(dir,base=dir){for(const item of await fs.readdir(dir,{withFileTypes:true})){const file=path.join(dir,item.name);if(item.isDirectory())await collect(file,base);else paths.push(path.relative(base,file).replaceAll('\\','/'));}}
await collect(path.join(good.base,settings.version));const complete=engine.execute({operation:'inspect-state-batch-selection',input:{manifest_json:null,year:'2020',states:['RI','VT'],completion:{source_name:'installed.json',paths_json:JSON.stringify({paths}),reprocess:false}}});assert.equal(complete.states.length,0);
for(const failureMode of ['prepare-failure','tamper','disk']){
 mode=failureMode;const failed=await run({options:failureMode==='disk'?{fail:p=>p.endsWith(path.join('rhode_island','plan.rplan'))}:{}});assert.equal(failed.receipt.summary.succeeded,2);assert.deepEqual(failed.receipt.native_output_summary,{completed:1,failed:1});
 const row=failed.receipt.states.find(r=>r.state==='RI');assert.equal(row.native_output.accepted_output_retained,true);assert((await fs.stat(path.join(failed.base,'RI','state-output.zip'))).size>0);assert.equal(JSON.parse(await fs.readFile(path.join(failed.base,'RI','completed.json'),'utf8')).native_output.status,'failed');
 if(failureMode==='disk')assert((await fs.stat(path.join(failed.base,settings.version,'2020/states/rhode_island/manifest.tmp'))).size>0);
}
mode='good';const controller=new AbortController();const cancelled=await run({signal:controller.signal,options:{afterClose:p=>{if(p.endsWith(path.join('vermont','manifest.tmp')))controller.abort();}}});assert.equal(cancelled.receipt.status,'cancelled');assert.equal(cancelled.receipt.summary.succeeded,1);assert.equal(cancelled.receipt.native_output_summary.completed,1);assert((await fs.stat(path.join(cancelled.base,settings.version,'2020/states/vermont/manifest.tmp'))).size>0);assert((await fs.stat(path.join(cancelled.base,'VT','state-output.zip'))).size>0);
const firstAbort=new AbortController(),firstCancelled=await run({signal:firstAbort.signal,options:{afterClose:p=>{if(p.endsWith(path.join('rhode_island','manifest.tmp')))firstAbort.abort();}}});assert.equal(firstCancelled.receipt.status,'cancelled');assert.equal(firstCancelled.receipt.summary.succeeded,0);assert.equal(firstCancelled.receipt.execution_performed,true);
const duplicateLabels=await run({nativeSettings:{...settings,layout:'labeled'}});assert.deepEqual(duplicateLabels.receipt.native_output_summary,{completed:1,failed:1});assert.equal(duplicateLabels.receipt.summary.succeeded,2);
const onePlan={...plan,items:plan.items.filter(i=>i.state_code==='RI')};const labeled=await run({selectedPlan:onePlan,nativeSettings:{...settings,layout:'labeled'}});assert.equal(labeled.receipt.native_output_summary.completed,1);assert(labeled.receipt.states[0].native_output.target_namespace.endsWith('/plans/Stream_native_run'));
const renamed=structuredClone(manifest);renamed.states.RI.name='Ocean State';const custom=await run({selectedPlan:onePlan,selected:{...selection,manifest:{name:'renamed.json',base64:Buffer.from(JSON.stringify(renamed)).toString('base64')}}});assert.equal(custom.receipt.native_output_summary.failed,1);assert.match(custom.receipt.states[0].native_output.error,/manifest state folder/);
const legacyFiles=JSON.parse(await fs.readFile('target/run-target-wasm/browser.bisect','utf8')).files,legacyForm=structuredClone(legacyFiles.tuiRunSettings);delete legacyForm.target;
const legacy=await run({selectedPlan:onePlan,execution:{files:legacyFiles,runSettings:legacyFiles.configRunSettings,form:legacyForm,workflow:{analysis:true,map:false,report:false,dpi:'72'}}});assert.equal(legacy.receipt.native_output_summary.completed,1,JSON.stringify(legacy.receipt.states));
for(const build of[false,true]){const request={graph:null,options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:null,seed:'18446744073709551615',seeds:2,steps:3,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1.1,iterations:10,capture_rounds:true}},text=JSON.stringify(build?{operation:'run-engine-build',request}:request).replace('"seed":"18446744073709551615"','"seed":18446744073709551615');const template={preparedCensusTemplateInputs:{enabled:true,use_display_geometry:false,source:{name:'template.json',text},settings:{label:'Stream template',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:false}}};const result=await run({selectedPlan:onePlan,execution:{files:template,runSettings:null,form:null,workflow:null}});assert.equal(result.receipt.native_output_summary.completed,1,JSON.stringify(result.receipt.states));}
const nationalPlan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:[],year:'2020',data_types:[],selected_paths:[]}}),national=await run({selectedPlan:nationalPlan,selected:{...selection,states:[]},fetcher:async url=>{const item=nationalPlan.items.find(i=>i.url===url);return new Response(await fs.readFile('target/state-source-acquisition/synthetic-states/'+item.state_code+'-'+(item.kind==='tiger'?'tiger':'pl94')+'.zip'));}});assert.equal(national.receipt.native_output_summary.completed,50);assert.equal(national.receipt.summary.succeeded,50);assert(JSON.stringify(national.receipt).length<256000);assert(!JSON.stringify(national.receipt).includes('files_base64'));await fs.writeFile(root+'/national.json',JSON.stringify(national));
const project=createProject({name:'Stream native output',files,operation:'plan-fetch-sources',constraints:[],result:good.receipt,lastOperation:'plan-fetch-sources'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
for(const value of [{...settings,version:'../escape'},{...settings,version:'CON'},{...settings,layout:'other'},{...settings,extra:1}])assert.throws(()=>validateStreamNativeOutputSettings(value));
const before=log.length;await assert.rejects(createLocalStateOutput(await directory('invalid'),{nativeOutputSettings:{...settings,version:'../bad'}}));assert.equal(log.length,before);
const browserFiles={...files,stateBatchInputs:selection,fetchPlanInputs:{source:{name:'two-state-selection.json',base64:Buffer.from(JSON.stringify({manifest_json:null,states:['RI','VT'],year:'2020',data_types:[],selected_paths:[]})).toString('base64')}}};await fs.writeFile(root+'/browser.bisect',JSON.stringify({...project,files:browserFiles,result:null,lastOperation:null}));await fs.writeFile(root+'/good.json',JSON.stringify(good));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,native_child_matches:native,successful_states:2,synthetic_national_states:50,yaml_and_plain_and_native_build_template_modes:true,peak_workers:peak,preparations,partitions,original_sources_and_flags_retained:true,selected_followup_retained:true,accepted_runs_retained_on_output_failure:true,preparation_tampering_and_disk_failure_disclosed:true,cancellation_retains_prior_native_outputs:true,cancelled_first_install_records_accepted_execution:true,state_and_labeled_layout:true,duplicate_labels_fail_without_overwrite:true,actual_native_completion_recognized:true,custom_manifest_mismatch_disclosed:true,inert_project_roundtrip:true},null,2));
console.log('PASS streamed native output: 8 native child matches, paired acquisition/flags/Run/follow-up/save/install, one worker at a time, no partition replay, failure retention, cancellation, both layouts and completion discovery');
