import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {stateRunPractitionerInputs} from '../../web/lab/state-run-practitioner.js';
import {runStateBatch} from '../../web/lab/state-run-batch.js';
import {tuiCompareRequest} from '../../web/lab/tui-compare-files.js';
import {textComparisonRequest} from '../../web/lab/text-comparison-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/state-comparison-handoff');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),sha=b=>createHash('sha256').update(b).digest('hex');let matches=0,refusals=0,scopeRefusals=0;
const workflow=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8')),prepared=JSON.parse(await fs.readFile('target/prepared-state-batch/batch.bisect','utf8')),plain=JSON.parse(await fs.readFile('target/state-run-batch/browser.bisect','utf8')),census=JSON.parse(await fs.readFile('target/state-source-batch/browser.bisect','utf8'));
async function check(request){const actual=engine.execute(request),file=path.join(root,'native-request.json');await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',timeout:120000,maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(actual,JSON.parse(native.stdout));matches++;return actual;}
class Adapter extends EventTarget{postMessage(m){setImmediate(()=>{if(this.closed)return;try{this.dispatchEvent(new MessageEvent('message',{data:m.type==='initialize'?{type:'ready',runtimeArtifact:engine.artifactIdentity}:{type:'result',id:m.id,result:engine.execute(m.request)}}));}catch(e){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}});}terminate(){this.closed=true;}}
const job=structuredClone(JSON.parse(Buffer.from(workflow.files.stateRunBatchInputs.jobs.base64,'base64'))[0]);job.form.seed='43';job.form.label='alternative_seed_43';const inputs={workers:1,jobs:{name:'alternative-state-run.json',base64:Buffer.from(JSON.stringify([job])).toString('base64')}};
const alternate=await runStateBatch(inputs,{year:'2020',states:['IA'],manifest:null},{createWorker:()=>new Adapter(),hardwareConcurrency:4,wasmUrl:'local'});assert.equal(alternate.summary.succeeded,1);assert.deepEqual(alternate.states[0].result,await check({operation:'run-tui-configured',input:job}));
const original=workflow.result.years[0].result,first=await stateRunPractitionerInputs(original,workflow.files.stateRunBatchInputs,'IA','compare-a');assert(first.files.textComparisonInputs.context);assert.deepEqual(first.files.tuiCompareInputs.b.sources,{});
const records=[];
async function pair(name,a,ai,as,b,bi,bs,expectExact){
 const selectedA=await stateRunPractitionerInputs(a,ai,as,'compare-a'),selectedB=await stateRunPractitionerInputs(b,bi,bs,'compare-b',selectedA.files);assert.deepEqual(selectedB.files.tuiCompareInputs.a,selectedA.files.tuiCompareInputs.a);assert.equal(selectedB.exactCompatible,expectExact);
 if(!expectExact&&selectedB.files.textComparisonInputs.a&&selectedB.files.textComparisonInputs.b)assert.equal(selectedB.files.textComparisonInputs.context,null);
 const result=await check(tuiCompareRequest(selectedB.files.tuiCompareInputs));
 const packet=createProject({name:'Retained State comparison '+name,operation:'compare-tui-plans',lastOperation:'compare-tui-plans',constraints:[],files:{...selectedB.files,stateRunBatchInputs:bi},result});assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(packet));
 if(expectExact){const exact=await check(textComparisonRequest(selectedB.files.textComparisonInputs));await fs.writeFile(path.join(root,name+'-exact.json'),JSON.stringify(exact));}
 else if(selectedB.files.textComparisonInputs.a&&selectedB.files.textComparisonInputs.b){const request=textComparisonRequest(selectedB.files.textComparisonInputs);assert.throws(()=>engine.execute(request),/identical ordered units/);const file=path.join(root,'scope-refusal.json');await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',timeout:120000,maxBuffer:1024*1024});assert.notEqual(native.status,0);assert.match(native.stderr,/identical ordered units/);scopeRefusals++;}
 records.push({name,exact_compatible:expectExact,a:as,b:bs});return selectedB;
}
const changed=await pair('different-seed',original,workflow.files.stateRunBatchInputs,'IA',alternate,inputs,'IA',true);assert.deepEqual(changed.files.textComparisonInputs.context,first.files.textComparisonInputs.context);
// Native comparisons align ordered IDs/kind/state/year/universe hash; source_id
// is descriptive metadata and must not discard an otherwise matching context.
const metadataFiles=structuredClone(first.files),metadataPlan=JSON.parse(Buffer.from(metadataFiles.textComparisonInputs.a.base64,'base64'));metadataPlan.plan.units.source_id+='-declared-alias';metadataFiles.textComparisonInputs.a.base64=Buffer.from(JSON.stringify(metadataPlan)).toString('base64');
const metadataPair=await stateRunPractitionerInputs(alternate,inputs,'IA','compare-b',metadataFiles);assert(metadataPair.exactCompatible);assert.deepEqual(metadataPair.files.textComparisonInputs.context,first.files.textComparisonInputs.context);await check(textComparisonRequest(metadataPair.files.textComparisonInputs));
await pair('configured-self',original,workflow.files.stateRunBatchInputs,'IA',original,workflow.files.stateRunBatchInputs,'IA',true);
await pair('prepared-self',prepared.result,prepared.files.stateRunBatchInputs,'IA',prepared.result,prepared.files.stateRunBatchInputs,'IA',true);
await pair('mixed-universe',prepared.result,prepared.files.stateRunBatchInputs,'IA',original,workflow.files.stateRunBatchInputs,'IA',false);
await pair('cross-vintage',original,workflow.files.stateRunBatchInputs,'IA',workflow.result.years[1].result,workflow.files.stateRunBatchInputs,'IA',false);
const noExport=structuredClone(prepared.result);delete noExport.states.find(r=>r.state_code==='IA').result.prepared.export;
const noPhysical=await pair('no-physical-export',noExport,prepared.files.stateRunBatchInputs,'IA',original,workflow.files.stateRunBatchInputs,'IA',false);assert.equal(noPhysical.files.textComparisonInputs.a,null);
await pair('different-states',prepared.result,prepared.files.stateRunBatchInputs,'RI',prepared.result,prepared.files.stateRunBatchInputs,'VT',false);
await pair('census-configured',census.result,census.files.stateRunBatchInputs,'RI',plain.result,plain.files.stateRunBatchInputs,'RI',false);
for(const kind of ['compactness','summary','splits','contiguity'])for(const field of ['files_base64','file_sha256']){const bad=structuredClone(original);bad.states[0].result.workflow[field]['analysis/'+kind+'.json']=field==='files_base64'?Buffer.from('{}').toString('base64'):'0'.repeat(64);await assert.rejects(()=>stateRunPractitionerInputs(bad,workflow.files.stateRunBatchInputs,'IA','compare-a'),/hash/);refusals++;}
await assert.rejects(()=>stateRunPractitionerInputs(original,workflow.files.stateRunBatchInputs,'IA','compare-c'));refusals++;
await fs.writeFile(path.join(root,'alternate.bisect'),JSON.stringify(createProject({name:'Alternative State seed43',operation:'run-state-batch',lastOperation:'run-state-batch',files:{stateRunBatchInputs:inputs,stateBatchInputs:{year:'2020',states:['IA'],manifest:null}},constraints:[],result:alternate})));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile('target/release/examples/execute_tool_request.exe')),whole_native_matches:matches,analysis_hash_refusals:refusals,matching_native_exact_scope_refusals:scopeRefusals,retained_plan_a_sources_unchanged:true,retained_context_only_when_units_match:true,prepared_without_export_comparison:true,records,scope:'Native descriptive TUI comparison accepts supplied assignment scopes; exact RPLAN comparison rejects differing ordered units/state/year without a crosswalk. Synthetic supplied snapshots; no source authenticity, certification, optimum or historical capacity claim.'},null,2)+'\n');console.log(`State comparison: ${matches} exact native replies, ${refusals} binding refusals, ${scopeRefusals} matching exact scope refusals; retained sources/context, different-seed runs and no-export prepared plans passed.`);
