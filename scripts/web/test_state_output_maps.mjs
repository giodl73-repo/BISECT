import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {stateRunBatchMap} from '../../web/lab/state-run-batch.js';
const root=path.resolve('target/state-output-maps');await fs.mkdir(root,{recursive:true});
const project=JSON.parse(await fs.readFile('target/state-national-analysis-handoff/browser.bisect','utf8')),jobs=JSON.parse(Buffer.from(project.files.stateRunBatchInputs.jobs.base64,'base64'));
const sha=b=>createHash('sha256').update(b).digest('hex');let nativeMatches=0,files=0,refusals=0;
for(const year of project.result.years){
 const request=path.join(root,'native-'+year.year+'.json');await fs.writeFile(request,JSON.stringify({operation:'run-tui-configured',input:jobs.find(j=>j.form.year===year.year)}));
 const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[request],{encoding:'utf8',maxBuffer:64*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);const result=JSON.parse(native.stdout);assert.deepEqual(year.result.states[0].result,result);nativeMatches++;
 for(const map_type of ['districts','political','demographic','compactness'])for(const format of ['png','svg']){
  const map=await stateRunBatchMap(year.result,'IA',{map_type,format});assert.equal(map.name,'IA-'+map_type+'.'+format);assert.deepEqual(Buffer.from(map.bytes),Buffer.from(result.workflow.files_base64['maps/'+map_type+'.'+format],'base64'));await fs.writeFile(path.join(root,year.year+'-'+map.name),map.bytes);files++;
 }
}
const batch=project.result.years[0].result;
for(const change of [
 b=>b.states[0].result.workflow.files_base64['maps/political.png']=Buffer.from('bad').toString('base64'),
 b=>b.states[0].result.workflow.file_sha256['maps/political.png']='0'.repeat(64),
 b=>b.states[0].result.workflow.stages.find(s=>s.name==='map:political').status='failed',
 b=>b.states[0].result.workflow.stages.find(s=>s.name==='map:political').receipt.map_type='demographic',
 b=>b.states[0].result.workflow.stages.find(s=>s.name==='map:political').receipt.state='RI',
 b=>b.states[0].result.workflow.stages.find(s=>s.name==='map:political').receipt.year=2000,
 b=>b.states[0].result.workflow.stages.find(s=>s.name==='map:political').receipt.plan_source_sha256='0'.repeat(64),
 b=>b.states[0].result.practitioner_export.document_json+=' ',
 b=>b.states[0].result.practitioner_export.context_json+=' ',
 b=>b.selection.year='2000',
 b=>b.status='running',
 b=>{const assignments=b.states[0].result.configured_run.result.assignments;assignments[Object.keys(assignments)[0]]=99;}
]){const bad=structuredClone(batch);change(bad);await assert.rejects(()=>stateRunBatchMap(bad,'IA',{map_type:'political'}));refusals++;}
for(const options of [{map_type:'rounds'},{format:'html'}]){await assert.rejects(()=>stateRunBatchMap(batch,'IA',options));refusals++;}
const changedSvg=structuredClone(batch);changedSvg.states[0].result.workflow.files_base64['maps/compactness.svg']=Buffer.from('<svg/>').toString('base64');await assert.rejects(()=>stateRunBatchMap(changedSvg,'IA',{map_type:'compactness',format:'svg'}),/identity/);refusals++;
for(const status of ['cancelled','interrupted']){const partial=structuredClone(batch);partial.status=status;partial.states[0].result.workflow.status='partial';assert((await stateRunBatchMap(partial,'IA',{map_type:'compactness'})).bytes.length>0);}
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:project.result.runtime_artifact.sha256,native_reference_sha256:sha(await fs.readFile('target/release/examples/execute_tool_request.exe')),native_whole_workflow_matches:nativeMatches,exact_native_output_files:files,types:4,years:3,formats:['png','svg'],refusals,cancelled_and_interrupted_outputs_retained:true,scope:'Supplied synthetic graphs and observations. Checks retained native map identity and exact bytes, not source authenticity, plan certification or historical capacity.'},null,2)+'\n');console.log(`State output maps: ${files} exact native files across four types/three years, ${nativeMatches} whole native matches, ${refusals} identity/scope guards and retained partial outputs.`);
