import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {isLabelBuildCheckpoint} from './fixtures/label-build-progress.mjs';
import {labelBuildRequest,startLabelBuildCheckpoint,retainLabelBuildCheckpoint,labelBuildZip,validateLabelBuildInputs} from '../../web/lab/label-build-files.js';
import {createBuildArchive,parseBuildArchive} from '../../web/lab/label-build-archive.js';
import {capturedBatchRoundResult,capturedRoundFiles,capturedRoundMapInputs} from '../../web/lab/round-capture-files.js';
import {validateNativeMapInputs} from '../../web/lab/map-export-files.js';
import {parseNativeMapPlanJson,createProject,parseProject} from '../../web/lab/project.js';
const root='target/batch-round-capture-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:12},(_,i)=>'44001'+String(i).padStart(6,'0')),population:Array(12).fill(100),areas:Array(12).fill(10000),exterior_perimeters:Array(12).fill(400),centroids:Array.from({length:12},(_,i)=>[-71.5+(i%4)*0.01,41.5+Math.floor(i/4)*0.01]),adjacency:Array.from({length:12},()=>[]),edges:[]};
for(let i=0;i<12;i++)for(const j of [i%4<3?i+1:-1,i+4<12?i+4:-1])if(j>=0){graph.adjacency[i].push(j);graph.adjacency[j].push(i);graph.edges.push([i,j,100]);}
const yaml='name: capture_batch\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  seed: 42\n  search: single\n  balance_tolerance: 50\nyears: ["2020", "2010"]\n';
const job=(year,districts=3)=>({graph:{...graph,year},districts,chamber:'congressional',demographics:null});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={config:source('capture.yml',yaml),jobs:source('jobs.json',JSON.stringify([job('2020'),job('2010',1)])),registry:null,settings:{label:'capture_batch',profile:'open',years:['2020','2010'],force:false,created_at:'2026-10-08T00:00:00Z',capture_rounds:true}};
const request=labelBuildRequest(inputs);assert.equal(request.input.capture_rounds,true);
let checkpoint=startLabelBuildCheckpoint(request);const checkpoints=[];
const result=engine.execute(request,{onProgress:event=>{if(!isLabelBuildCheckpoint(event))return;checkpoint=retainLabelBuildCheckpoint(checkpoint,event);checkpoints.push(checkpoint);}});
await fs.writeFile(root+'/request.json',JSON.stringify(request));
const native=JSON.parse(execFileSync(process.env.BISECT_NATIVE_HOST||'target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:80*1024*1024}));assert.deepEqual(result,native);
const row=result.states.find(r=>r.year==='2020'),view=capturedBatchRoundResult(result,row.plan_path),files=await capturedRoundFiles(view,0),map=await capturedRoundMapInputs(view,0);assert.equal(map.geometry,null);assert.equal(map.round_key_basis,'native-index-map');assert.deepEqual(map.round_assignments,files.round);assert.equal(capturedBatchRoundResult(plainPlaceholder(),row.plan_path),null);function plainPlaceholder(){return {...result,states:result.states.map(r=>({...r,round_capture:undefined}))};}
assert.equal(result.status,'built');assert.deepEqual(checkpoint.files,result.files);
for(const row of result.states){
 const capture=JSON.parse(result.files[row.round_capture.path]),plan=JSON.parse(result.files[row.plan_path]);assert.equal(capture.schema_version,'bisect-native-round-capture-v1');assert.equal(row.round_capture.accepted_attempt,row.build_report.accepted_attempt);
 assert.equal(row.round_capture.events,capture.events.length);assert.equal(row.round_capture.status,capture.status);
 const index=JSON.parse(capture.unit_index_json);for(const [i,id]of Object.entries(index))assert.equal(plan.plan.units.unit_ids[Number(i)],id);
 if(row.year==='2020'){assert(capture.events.length>0);assert.equal(capture.terminal_matches_returned_assignment,true);for(const [i,region]of Object.entries(JSON.parse(capture.events.at(-1).assignments_json)))assert.equal(plan.plan.assignment[Number(i)]+1,region);}else{assert.equal(capture.status,'no-events-emitted');assert.equal(capture.events.length,0);}
}
let plainIdentity;const plain=engine.execute(labelBuildRequest({...inputs,settings:{...inputs.settings,capture_rounds:false}}),{onProgress:e=>{if(isLabelBuildCheckpoint(e))plainIdentity=e.request_sha256;}});assert.notEqual(plainIdentity,checkpoint.checkpoint_request_sha256);
for(const row of result.states){const other=plain.states.find(s=>s.state===row.state&&s.year===row.year);assert.equal(Object.hasOwn(other,'round_capture'),false);assert.deepEqual(other.metrics,row.metrics);assert.deepEqual(other.build_report,row.build_report);const a=JSON.parse(plain.files[row.plan_path]),b=JSON.parse(result.files[row.plan_path]);assert.deepEqual(a.plan.assignment,b.plan.assignment);assert.deepEqual(a.plan.units,b.plan.units);assert.equal(b.provenance.producer.engine_options.capture_rounds,true);assert.equal(Object.hasOwn(a.provenance.producer.engine_options,'capture_rounds'),false);assert.equal(plain.files[row.plan_path.replace('plan.rplan','final_assignments.json')],result.files[row.plan_path.replace('plan.rplan','final_assignments.json')]);}
assert.equal(Object.keys(plain.files).some(p=>p.endsWith('/round_capture.json')),false);
const bad=job('2020');bad.graph.population=bad.graph.population.map((_,i)=>i===0?100000:1);const mixed=engine.execute({...request,input:{...request.input,jobs:[bad,job('2010')]}});assert.equal(mixed.status,'partial-failure');assert.equal(Object.hasOwn(mixed.states[0],'round_capture'),false);assert(mixed.states[1].round_capture.events>0);assert.equal(Object.keys(mixed.files).some(p=>p.includes('/2020/rhode_island/')),false);
const cancelled={...checkpoints[0],status:'cancelled'};assert(Object.keys(cancelled.files).some(p=>p.endsWith('/round_capture.json')));assert.equal(cancelled.years.length,0);
const archive=createBuildArchive({name:'Captured batch',inputs,result,lastOperation:'run-label-build'});assert.deepEqual(parseBuildArchive(archive).result,JSON.parse(JSON.stringify(result)));
const project=createProject({name:'Captured batch',files:{labelBuildInputs:inputs},operation:'run-label-build',constraints:[],result,lastOperation:'run-label-build'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
assert.deepEqual(parseBuildArchive(createBuildArchive({name:'Cancelled capture',inputs,result:cancelled,lastOperation:'run-label-build'})).result,JSON.parse(JSON.stringify(cancelled)));
for(const bad of ['yes',null,1])assert.throws(()=>validateLabelBuildInputs({...inputs,settings:{...inputs.settings,capture_rounds:bad}}));
for(const bad of ['yes',null,1])assert.throws(()=>engine.execute({...request,input:{...request.input,capture_rounds:bad}}));
const skipped=engine.execute({...request,input:{...request.input,jobs:[],registry_json:JSON.stringify({capture_batch:{built:['2020','2010'],analyzed:[],reported:[]}})}});assert.equal(skipped.status,'skipped');assert.equal(skipped.states.length,0);assert.equal(Object.keys(skipped.files).some(p=>p.endsWith('/round_capture.json')),false);
const maxSeed=engine.execute({...request,input:{...request.input,yaml_text:request.input.yaml_text.replace('seed: 42','seed: 18446744073709551615')}});const maxRow=maxSeed.states.find(r=>r.year==='2020');assert(maxRow.round_capture);const maxView=capturedBatchRoundResult(maxSeed,maxRow.plan_path),maxInputs=await capturedRoundMapInputs(maxView,0);validateNativeMapInputs(maxInputs);assert.equal(parseNativeMapPlanJson(maxSeed.files[maxRow.plan_path]).provenance.producer.engine_options.seed,maxRow.build_report.attempts.at(-1).seed);assert.equal(Buffer.from(maxInputs.plan.base64,'base64').toString(),maxSeed.files[maxRow.plan_path]);for(const raw of ['{"bad":18446744073709551615}','{"provenance":{"producer":{"engine_options":{"seed":18446744073709551616}}}}}','{"provenance":{"producer":{"engine_options":{"seed":-9007199254740993}}}}}'])assert.throws(()=>parseNativeMapPlanJson(raw));
const legacy={...inputs,settings:{...inputs.settings}};delete legacy.settings.capture_rounds;assert.equal(Object.hasOwn(labelBuildRequest(legacy).input,'capture_rounds'),false);

if(process.argv.includes('--local-national')){
 const inputProject=JSON.parse(await fs.readFile('target/label-build-national/inputs.bisect','utf8')),original=labelBuildRequest(inputProject.files.labelBuildInputs),selected=original.input.jobs.filter(j=>['CA','TX','RI'].includes(j.graph.state));assert.equal(selected.length,3);
 const request={...original,input:{...original.input,jobs:selected,capture_rounds:true}};await fs.writeFile(root+'/national-subset-request.json',JSON.stringify(request));
 let checkpoint=startLabelBuildCheckpoint(request);const captured=engine.execute(request,{onProgress:e=>{if(!isLabelBuildCheckpoint(e))return;checkpoint=retainLabelBuildCheckpoint(checkpoint,e);if(e.kind==='state-completed')console.log('Real '+e.state.state+' capture: '+e.state.status);}});
 const native=JSON.parse(execFileSync('target/debug/examples/execute_tool_request.exe',[root+'/national-subset-request.json'],{encoding:'utf8',maxBuffer:100*1024*1024}));assert.deepEqual(captured,native);assert.deepEqual(checkpoint.files,captured.files);assert.equal(captured.status,'built');
 const baseline=JSON.parse(await fs.readFile('target/label-build-national/browser-complete.bisect-build','utf8'));
 for(const row of captured.states){const prior=baseline.result.states.find(s=>s.state===row.state&&s.year===row.year);assert(prior);assert.deepEqual(row.build_report,prior.build_report);assert.deepEqual(row.metrics,prior.metrics);assert.deepEqual(JSON.parse(captured.files[row.plan_path]).plan.assignment,JSON.parse(baseline.result.files[prior.plan_path]).plan.assignment);const view=capturedBatchRoundResult(captured,row.plan_path);assert(view.result.round_capture.events.length>0);assert.equal(row.round_capture.accepted_attempt,row.build_report.accepted_attempt);await capturedRoundFiles(view,0);}
 await fs.writeFile(root+'/national-subset-result.json',JSON.stringify(captured));
 console.log('Real CA/TX/RI captured batch matches native observer and prior completed browser assignments, metrics and accepted retry histories.');
}
await fs.writeFile(root+'/captured.bisect',JSON.stringify(project));await fs.writeFile(root+'/captured.bisect-build',archive);await fs.writeFile(root+'/captured.zip',labelBuildZip(result));await fs.writeFile(root+'/result.json',JSON.stringify(result));
console.log('Batch round capture: native/WASM exact two-year package and checkpoint parity, complete/no-event metadata, accepted-attempt/final-plan binding, capture-on/off assignment/metric/report equality, legacy settings, rejected nonboolean flags, exact ZIP and inert ordinary/build/cancelled archive retention passed.');
