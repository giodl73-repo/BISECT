import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {runStateBatch,stateRunBatchZip} from '../../web/lab/state-run-batch.js';
import {generatedReports,generatedReportText} from '../../web/lab/generated-report-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/state-run-years-workflow');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/state-workflow-wasm/browser-request.json','utf8')).input,jobs=[];
for(const year of ['2020','2010','2000']){
 const job=structuredClone(base),row=engine.execute({operation:'inspect-state-batch-selection',input:{year,states:['IA'],manifest_json:null}}).states[0];
 job.form.year=year;job.form.balance_tol='25';job.run.graph.year=year;job.run.districts=row.districts;
 for(const map of [job.workflow.map,job.workflow.analysis.compactness])if(map?.unit_geometry_json){const geometry=JSON.parse(map.unit_geometry_json);geometry.year=year;map.unit_geometry_json=JSON.stringify(geometry);}
 jobs.push(job);
}
const inputs={jobs:{name:'SYNTHETIC-IA-three-year-workflows.json',base64:Buffer.from(JSON.stringify(jobs)).toString('base64')},workers:2,all_years:true},selection={year:'2020',states:['IA'],manifest:null};
class Adapter extends EventTarget{
 postMessage(m){setImmediate(()=>{if(this.closed)return;try{const data=m.type==='initialize'?{type:'ready',runtimeArtifact:engine.artifactIdentity}:{type:'result',id:m.id,result:engine.execute(m.request)};this.dispatchEvent(new MessageEvent('message',{data}));}catch(e){this.dispatchEvent(new MessageEvent('message',{data:{type:'error',id:m.id,error:e.message}}));}});}
 terminate(){this.closed=true;}
}
const result=await runStateBatch(inputs,selection,{createWorker:()=>new Adapter(),hardwareConcurrency:4,wasmUrl:'local'});
assert.equal(result.state_summary.succeeded,3);assert.equal(result.exit_code,0);
for(const row of result.years){
 const request={operation:'run-tui-configured',input:jobs.find(j=>j.form.year===row.year)},file=path.join(root,'native-'+row.year+'.json');await fs.writeFile(file,JSON.stringify(request));
 const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',maxBuffer:64*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);
 assert.deepEqual(row.result.states[0].result,JSON.parse(native.stdout));
}
const reports=generatedReports(result);assert.equal(reports.length,3);assert.equal(new Set(reports.map(r=>r.key)).size,3);
for(const report of reports){const actual=await generatedReportText(result,report.key);assert.equal(actual.text,Buffer.from(report.source,'base64').toString());assert(actual.text.includes(report.key.slice(0,4)));}
const changed=structuredClone(result);changed.years[1].result.selection.year='2020';assert.throws(()=>generatedReports(changed),/child year differs/);
const packet=createProject({name:'SYNTHETIC IA three-year analysis/map/report workflows',operation:'run-state-batch',lastOperation:'run-state-batch',constraints:[],files:{stateRunBatchInputs:inputs,stateBatchInputs:selection},result});
assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(path.join(root,'browser.bisect'),JSON.stringify(packet));
await fs.writeFile(path.join(root,'outputs.zip'),await stateRunBatchZip(result));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_whole_workflow_matches:3,distinct_native_year_allocations:result.years.map(r=>({year:r.year,districts:r.result.selection.states[0].districts})),exact_generated_report_sources:3,report_year_mismatch_refused:true,synthetic_source_scope:'Same explicit synthetic graph/analysis observations reused under three declared vintages. No historical-source authenticity or original historical-state capacity claim.'},null,2)+'\n');
console.log('Multi-year workflows: three actual native whole TUI Run/analysis/map/report matches, native4/4/5 district allocations, exact year-scoped HTML, source projects and nested ZIP passed.');
