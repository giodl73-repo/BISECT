import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {stateRunPractitionerInputs} from '../../web/lab/state-run-practitioner.js';
import {reproducibilityRequest,reproducibilityZip} from '../../web/lab/reproducibility-files.js';
import {doctorRequest} from '../../web/lab/doctor-files.js';
import {engineReplayRequest} from '../../web/lab/engine-replay-files.js';
import {textAuditRequest,improvementRequest} from '../../web/lab/package-files.js';
import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/state-practitioner-handoff');await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),sha=b=>createHash('sha256').update(b).digest('hex'),source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const profile=source('us-congressional-project-v1.json',await fs.readFile('crates/rplan-audit/profiles/us-congressional-project-v1.json','utf8'));
const base={auditProfileText:profile,auditCertificateText:source('stale.json','{}'),auditLineageText:source('stale-lineage.json','{}'),textAuditSettings:{generated_at_utc:'2026-10-18T00:00:00Z'},improvementSettings:{tolerance_percent:25,label:'selected_state_improved',generated_at_utc:'2026-10-18T01:00:00Z'}};
const workflow=JSON.parse(await fs.readFile('target/state-run-years-workflow/browser.bisect','utf8')),prepared=JSON.parse(await fs.readFile('target/prepared-state-batch/batch.bisect','utf8')),census=JSON.parse(await fs.readFile('target/state-source-batch/browser.bisect','utf8'));
const plain=JSON.parse(await fs.readFile('target/state-run-batch/browser.bisect','utf8'));
const requests={audit:f=>textAuditRequest('audit-rplan-text',f,[],f.textAuditSettings),doctor:f=>doctorRequest(f.doctorInputs),improve:f=>improvementRequest(f,f.improvementSettings),replay:f=>engineReplayRequest(f.engineReplayInputs),reproducibility:f=>reproducibilityRequest(f,f.reproducibilityInputs)};
let matches=0,refusals=0;const records=[];
for(const [kind,packet,batch]of [...workflow.result.years.map(y=>['configured',workflow,y.result]),['prepared',prepared,prepared.result],['census',census,census.result],['without-workflow',plain,plain.result]])for(const row of batch.states.filter(r=>r.status==='ok'))for(const action of Object.keys(requests)){
 const selected=await stateRunPractitionerInputs(batch,packet.files.stateRunBatchInputs,row.state_code,action,base);assert.equal(selected.state,row.state_code);assert.equal(selected.year,batch.selection.year);
 if(action!=='replay'){assert.deepEqual(selected.files.auditProfileText,profile);assert.equal(selected.files.auditCertificateText,undefined);assert.equal(selected.files.auditLineageText,undefined);const native=row.result.practitioner_export??row.result.prepared.export;assert.equal(Buffer.from(selected.files.auditPlanText.base64,'base64').toString(),native.document_json);assert.equal(Buffer.from(selected.files.auditContextText.base64,'base64').toString(),native.context_json);}
 if(action==='reproducibility'){assert.equal(selected.files.reproducibilityInputs.manifest===null,kind==='prepared');if(kind==='prepared'||kind==='without-workflow')assert.deepEqual(selected.files.reproducibilityInputs.analysis,{});}
 if(action==='improve')assert.deepEqual(selected.files.improvementSettings,base.improvementSettings);
 const request=requests[action](selected.files),file=path.join(root,'request.json');await fs.writeFile(file,JSON.stringify(request));const actual=engine.execute(request),native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',timeout:120000,maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(actual,JSON.parse(native.stdout));matches++;if(action==='reproducibility'){await fs.writeFile(path.join(root,kind+'-'+batch.selection.year+'-'+row.state_code+'-package.zip'),reproducibilityZip(actual));assert.equal(actual.source_plan_sha256,sha(request.document_json));assert.equal(actual.manifest_supplied,kind!=='prepared');}
 if(action==='replay'){assert.equal(actual.status,'match');assert.equal(actual.agreement.similarity,1);}
 const project=createProject({name:'State '+kind+' '+row.state_code+' '+batch.selection.year+' '+action,operation:selected.operation,lastOperation:selected.operation,files:{...packet.files,...selected.files},constraints:[],result:actual});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(root,kind+'-'+batch.selection.year+'-'+row.state_code+'-'+action+'.bisect'),JSON.stringify(project));records.push({kind,year:batch.selection.year,state:row.state_code,action,schema:actual.schema_version});
}
const first=workflow.result.years[0].result;
for(const change of [r=>r.report_manifest.manifest_json+=' ',r=>r.report_manifest.plan_source_sha256='0'.repeat(64),r=>r.report_manifest.context_source_sha256='0'.repeat(64),r=>r.report_manifest.manifest_sha256='0'.repeat(64),r=>r.workflow.file_sha256['analysis/summary.json']='0'.repeat(64)]){const bad=structuredClone(first);change(bad.states[0].result);await assert.rejects(()=>stateRunPractitionerInputs(bad,workflow.files.stateRunBatchInputs,'IA','reproducibility'),/binding|hash mismatch/);refusals++;}
const stalePackage={reproducibilityInputs:{label:'retained_package_label',manifest:source('stale-manifest.json','{}'),analysis:{'old.json':source('old.json','{}')}}};const preparedPackage=await stateRunPractitionerInputs(prepared.result,prepared.files.stateRunBatchInputs,prepared.result.states[0].state_code,'reproducibility',stalePackage);assert.deepEqual(preparedPackage.files.reproducibilityInputs,{label:'retained_package_label',manifest:null,analysis:{}});assert.deepEqual(stalePackage.reproducibilityInputs.analysis,{'old.json':source('old.json','{}')});

for(const change of [
 b=>b.states[0].result.practitioner_export.document_json+=' ',
 b=>b.states[0].result.practitioner_export.context_json+=' ',
 b=>b.states[0].result.practitioner_export.document.metadata.label='changed',
 b=>b.states[0].result.workflow.accepted_plan_source_sha256='0'.repeat(64),
 b=>b.states[0].result.workflow.file_sha256['context.rctx']='0'.repeat(64),
 b=>b.states[0].result.form.year='2010',
 b=>b.states[0].result.configured_run.engine_request.graph.state='RI',
 b=>{const a=b.states[0].result.configured_run.result.assignments;a[Object.keys(a)[0]]=99;},
 b=>b.states.push(structuredClone(b.states[0])),
 b=>b.states[0].status='failed'
]){const bad=structuredClone(first);change(bad);await assert.rejects(()=>stateRunPractitionerInputs(bad,workflow.files.stateRunBatchInputs,'IA','audit'));refusals++;}
const wrong=structuredClone(workflow.files.stateRunBatchInputs);wrong.jobs.base64=Buffer.from('[]').toString('base64');await assert.rejects(()=>stateRunPractitionerInputs(first,wrong,'IA','audit'),/job bytes/);refusals++;
await assert.rejects(()=>stateRunPractitionerInputs(first,workflow.files.stateRunBatchInputs,'IA','unknown'));refusals++;
const defaults=await stateRunPractitionerInputs(first,workflow.files.stateRunBatchInputs,'IA','improve');assert.equal(defaults.files.improvementSettings.tolerance_percent,25);assert.equal(defaults.files.improvementSettings.generated_at_utc,'');
// Replay remains available when a prepared run lacks a physical export.
const noExport=structuredClone(prepared.result);delete noExport.states[0].result.prepared.export;await assert.rejects(()=>stateRunPractitionerInputs(noExport,prepared.files.stateRunBatchInputs,noExport.states[0].state_code,'audit'),/no physical/);refusals++;
await assert.rejects(()=>stateRunPractitionerInputs(noExport,prepared.files.stateRunBatchInputs,noExport.states[0].state_code,'reproducibility'),/no physical/);refusals++;
const replay=await stateRunPractitionerInputs(noExport,prepared.files.stateRunBatchInputs,noExport.states[0].state_code,'replay');assert(replay.files.engineReplayInputs.run);
for(const status of ['cancelled','interrupted']){const partial=structuredClone(first);partial.status=status;assert.equal((await stateRunPractitionerInputs(partial,workflow.files.stateRunBatchInputs,'IA','doctor')).state,'IA');}
let originalRiPackage=false;
if(process.argv.includes('--local-live-ri')){const packet=await parseWorkbenchArchiveBytes(new Uint8Array(await fs.readFile('target/live-state-relay-workflow/fresh.bisect-workbench')));const selected=await stateRunPractitionerInputs(packet.result,packet.files.stateRunBatchInputs,'RI','reproducibility');const request=reproducibilityRequest(selected.files,selected.files.reproducibilityInputs);assert.match(request.manifest_json,/18446744073709551615/);await fs.writeFile(path.join(root,'original-ri-request.json'),JSON.stringify(request));const actual=engine.execute(request),native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.join(root,'original-ri-request.json')],{encoding:'utf8',timeout:120000,maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(actual,JSON.parse(native.stdout));await fs.writeFile(path.join(root,'original-ri-package.zip'),reproducibilityZip(actual));await fs.writeFile(path.join(root,'original-ri-package.json'),JSON.stringify(actual));originalRiPackage=true;}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile('target/release/examples/execute_tool_request.exe')),whole_native_tool_matches:matches,original_ri_native_package_with_exact_max_u64_seed:originalRiPackage,exact_plan_context_sources:true,prepared_without_export_replay:true,reproducibility_manifest_analysis_replaced:true,prepared_package_no_fabricated_manifest_or_analysis:true,package_zips_native_verified:true,preserved_profile_and_improvement_settings:true,new_settings_use_accepted_tolerance:true,source_projects_roundtrip:true,refusals,records,scope:'Retained synthetic configured/prepared/Census snapshots, plus optional local original RI2020 package evidence with exact max-u64 engine_seed when its flag is true. Handoff establishes source correspondence, not authenticity or certification. Audit uses an explicit supplied profile; replay and improvement run only on explicit execution.'},null,2)+'\n');console.log(`State practitioner handoff: ${matches} exact native audit/Doctor/improve/replay/package replies, source projects and ${refusals} refusals; replay without physical export and partial outputs retained.`);
