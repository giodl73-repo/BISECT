import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tuiRunRequest,tuiRunAssignmentsText,tuiRunCompareSide,tuiRunZip} from '../../web/lab/tui-run-files.js';
import {stateWorkflowOptions,stateWorkflowZip} from '../../web/lab/state-workflow-files.js';
import {tuiCompareRequest} from '../../web/lab/tui-compare-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/tui-run-compare');await fs.mkdir(root,{recursive:true});
const base=JSON.parse(await fs.readFile('target/state-workflow-wasm/browser.bisect','utf8')),files=base.files,engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const settings={...files.tuiRunSettings,form:{...files.tuiRunSettings.form,seed:'91',label:'comparison_b'}},request=tuiRunRequest(settings,files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings);request.input.workflow=stateWorkflowOptions(files,files.tuiRunWorkflowSettings,settings.created_at);
await fs.writeFile(root+'/run-b-request.json',JSON.stringify(request));const b=engine.execute(request),nativeB=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/run-b-request.json'],{encoding:'utf8',maxBuffer:128*1024*1024}));assert.deepEqual(b,nativeB);
const inputs={a:await tuiRunCompareSide(base.result),b:await tuiRunCompareSide(b)};
for(const [side,result]of [['a',base.result],['b',b]]){
 const assignments=JSON.parse(Buffer.from(inputs[side].sources['data/final_assignments.json'].base64,'base64').toString());assert.deepEqual(assignments,result.configured_run.result.assignments);
 const doc=JSON.parse(result.practitioner_export.document_json);assert(doc.plan.assignment.every((district,i)=>assignments[doc.plan.units.unit_ids[i]]===district+1));
 for(const [file,source]of Object.entries(inputs[side].sources)){if(file.startsWith('analysis/'))assert.equal(source.base64,result.workflow.files_base64[file]);const target=root+'/native/'+side+'/'+file;await fs.mkdir(path.dirname(target),{recursive:true});await fs.writeFile(target,Buffer.from(source.base64,'base64'));}
 await fs.writeFile(root+'/'+side+'-run.zip',tuiRunZip(result));await fs.writeFile(root+'/'+side+'-workflow.zip',await stateWorkflowZip(result));
}
const result=engine.execute(tuiCompareRequest(inputs)),native=JSON.parse(execFileSync('target/debug/examples/compare_reference.exe',[root+'/native/a',root+'/native/b'],{encoding:'utf8'}));assert.deepEqual(result.metrics,native);assert.equal(result.execution_performed,false);assert(result.metrics.jaccard<1);
const project=createProject({name:'Two accepted runs for comparison',files:{...files,tuiCompareInputs:inputs},operation:'compare-tui-plans',constraints:[],result,lastOperation:'compare-tui-plans'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/comparison.bisect',JSON.stringify(project));await fs.writeFile(root+'/run-b.bisect',JSON.stringify(createProject({...base,files:{...files,tuiRunSettings:settings},result:b})));
// Extract only the five fixed native comparison paths from application-produced ZIPs.
const extract="import zipfile,pathlib,sys; z=zipfile.ZipFile(sys.argv[1]); root=pathlib.Path(sys.argv[2]); paths=['data/final_assignments.json','analysis/compactness.json','analysis/summary.json','analysis/splits.json','analysis/contiguity.json']; [( (root/p).parent.mkdir(parents=True,exist_ok=True),(root/p).write_bytes(z.read(p)) ) for p in paths if p in z.namelist()]; print(z.read('data/final_assignments.json').decode())";
for(const side of ['a','b'])for(const type of ['run','workflow']){const dir=root+'/extracted/'+side+'-'+type;const exported=execFileSync('python',['-c',extract,root+'/'+side+'-'+type+'.zip',dir],{encoding:'utf8'});assert.deepEqual(JSON.parse(exported),JSON.parse(Buffer.from(inputs[side].sources['data/final_assignments.json'].base64,'base64').toString()));}
const zipNative=JSON.parse(execFileSync('target/debug/examples/compare_reference.exe',[root+'/extracted/a-workflow',root+'/extracted/b-workflow'],{encoding:'utf8'}));assert.deepEqual(zipNative,result.metrics);
const plain=engine.execute({...request,input:{...request.input,workflow:undefined}}),plainSide=await tuiRunCompareSide(plain);assert.deepEqual(Object.keys(plainSide.sources),['data/final_assignments.json']);
for(const mutate of [r=>r.configured_run.result.assignments[Object.keys(r.configured_run.result.assignments)[0]]++,r=>r.workflow.accepted_plan_source_sha256='0'.repeat(64),r=>r.workflow.file_sha256['analysis/compactness.json']='0'.repeat(64)]){const bad=structuredClone(b);mutate(bad);await assert.rejects(()=>tuiRunCompareSide(bad));}
assert.throws(()=>tuiRunAssignmentsText({...b,status:'failed'}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_generated_second_run:true,native_directory_comparison_metrics_equal:true,distinct_plan_assignments:true,zero_to_one_based_alignment:true,exact_analysis_bytes_retained:true,native_workflow_zip_folder_comparison_equal:true,plain_run_missing_analysis_preserved:true,source_binding_and_hash_tampering_refused:true,inert_project_roundtrip:true},null,2));console.log('Accepted Run comparison: native second Run, directory/reference and exported ZIP comparisons, exact source bytes, native district-label alignment, missing-analysis defaults and tamper guards passed.');
