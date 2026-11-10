import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readTuiLibraryFolder,tuiLibraryRequest,tuiLibraryCompareInputs,validateTuiLibraryInputs} from '../../web/lab/tui-library-files.js';
import {tuiCompareRequest} from '../../web/lab/tui-compare-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/tui-library-compare');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const assignmentA='{"unit0":18446744073709551615,"unit1":2,"unit2":3}',assignmentB='{"unit0":18446744073709551615,"unit1":9,"unit2":9}';
const sources={};for(const [name,assignment] of [['a',assignmentA],['b',assignmentB]]){
 sources['plans/'+name+'/manifest.json']=JSON.stringify({label:'Plan '+name.toUpperCase(),state_code:'RI',chamber:'congressional',year:'2020',num_districts:2});
 sources['plans/'+name+'/analysis/summary.json']='{"max_deviation_pct":0.2}';sources['plans/'+name+'/data/final_assignments.json']=assignment;
}
for(const [name,text] of Object.entries(sources)){const file=path.join(root,name);await fs.mkdir(path.dirname(file),{recursive:true});await fs.writeFile(file,text);}
let reads=0;const file=(name,text)=>({name:path.basename(name),webkitRelativePath:name,size:Buffer.byteLength(text),arrayBuffer:async()=>{reads++;return new TextEncoder().encode(text).buffer;}});
const inputs=await readTuiLibraryFolder(Object.entries(sources).map(([name,text])=>file(name,text))),before=structuredClone(inputs),listed=engine.execute(tuiLibraryRequest(inputs));assert.equal(listed.plans.length,2);assert(!JSON.stringify(tuiLibraryRequest(inputs)).includes('unit0'));
const sideB=tuiLibraryCompareInputs(inputs,'plans/b','Plan B').a,compared=tuiLibraryCompareInputs(inputs,'plans/a','Plan A',sideB);assert.deepEqual(inputs,before);
assert.equal(Buffer.from(compared.a.sources['data/final_assignments.json'].base64,'base64').toString(),assignmentA);
const result=engine.execute(tuiCompareRequest(compared)),native=JSON.parse(execFileSync('target/debug/examples/compare_reference.exe',[root+'/plans/a',root+'/plans/b'],{encoding:'utf8'}));assert.deepEqual(result.metrics,native);assert.equal(result.metrics.jaccard,1/3);
await fs.writeFile(root+'/native-request.json',JSON.stringify(tuiCompareRequest(compared)));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/native-request.json'],{encoding:'utf8'})),result);
const dropped=structuredClone(compared);delete dropped.a.sources['data/final_assignments.json'];assert.equal(engine.execute(tuiCompareRequest(dropped)).metrics.jaccard,0);
compared.a.sources['data/final_assignments.json'].name='renamed.json';assert.deepEqual(inputs,before);
const legacy=structuredClone(inputs);for(const p of legacy.plans)delete p.assignments;validateTuiLibraryInputs(legacy);assert.equal(tuiLibraryCompareInputs(legacy,'plans/a','Plan A').a.sources['data/final_assignments.json'],undefined);
const readCount=reads;await assert.rejects(()=>readTuiLibraryFolder([file('plans/a/manifest.json','{}'),{...file('plans/a/data/final_assignments.json','{}'),size:8*1024*1024+1}]),/8 MiB/);assert.equal(reads,readCount);
await assert.rejects(()=>readTuiLibraryFolder(Array.from({length:3},(_,i)=>({...file('plans/'+i+'/data/final_assignments.json','{}'),size:6*1024*1024}))),/16 MiB/);assert.equal(reads,readCount);
const assignmentFile=file('plans/a/data/final_assignments.json','{}');await assert.rejects(()=>readTuiLibraryFolder([assignmentFile,assignmentFile]),/Duplicate/);
await assert.rejects(()=>readTuiLibraryFolder([{...assignmentFile,size:1,arrayBuffer:async()=>new Uint8Array([255]).buffer}]));
const empty=await readTuiLibraryFolder([file('plans/empty/data/final_assignments.json','')]);assert.equal(empty.plans[0].assignments.base64,'');
const project=createProject({name:'Native library assignment handoff',files:{tuiLibraryInputs:inputs,tuiCompareInputs:{a:{label:'Plan A',sources:{}},b:sideB}},operation:'inspect-tui-plan-library',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/expected.json',JSON.stringify(result));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,native_directory_metrics_match:true,correct_score:1/3,without_handoff_assignments_score:0,exact_u64_assignment_bytes:true,immutable_handoff:true,old_library_projects_supported:true,assignment_8MiB_and_total_16MiB_pre_read_limits:true,empty_duplicate_utf8_guards:true,published:false},null,2));console.log('PASS TUI library comparison handoff: exact u64 assignment bytes, actual native directory score, legacy sources, immutable selection and pre-read size/UTF-8/duplicate guards.');
