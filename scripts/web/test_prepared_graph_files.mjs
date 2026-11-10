import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {PREPARED_GRAPH_DEFAULTS,validatePreparedGraphInputs,preparedGraphRequest,preparedGraphFromNative,preparedEngineFromGraphBinding} from '../../web/lab/prepared-graph-files.js';
import {preparedEngineRequest} from '../../web/lab/prepared-engine-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/prepared-graph-files';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let matches=0,cases=0,refusals=0,executions=0,lastInputs,lastBinding;const formats=new Set(),kinds=new Set();
async function parity(request){const actual=engine.execute(request);executions++;await fs.writeFile(root+'/request.json',JSON.stringify(request));const expected=JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:128*1024*1024,stdio:['ignore','pipe','pipe']}));assert.deepEqual(actual,expected);matches++;return actual;}
for(const name of ['county-prepared.bisect','tract-prepared.bisect','block-group-prepared.bisect','block-prepared.bisect','v2-supplemental.bisect','ri-prepared.bisect']){
 const source=JSON.parse(await fs.readFile('target/native-engine-input-wasm/'+name,'utf8')),native=await parity(nativeEngineInputRequest(source.files.nativeAdjacencyInputs,source.files.nativeAdjacencySettings));
 formats.add(native.format_version);kinds.add(native.unit_kind??'tract');if(name==='ri-prepared.bisect')assert.equal(native.units,250);
 const selected=await preparedGraphFromNative(native);assert.equal(selected.graph.text,native.graph_json);assert.equal(selected.districts,2);
 for(const build of [false,true]){
  const request={graph:null,options:{structure:'nway',weights:'geographic',search:'single',districts:null,seed:'18446744073709551615',seeds:2,steps:3,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,metis_objective:'volume',metis_trials:2,metis_ufactor:30,capture_rounds:true}};
  const spec=build?{operation:'run-engine-build',request}:request;
  const text=JSON.stringify(spec).replace('"18446744073709551615"','18446744073709551615');
  const inputs={...selected,template:{name:'full-template.json',text},settings:{...selected.settings,label:'Native full-options '+name,chamber:'house',created_at:'2026-10-17T00:00:00Z',inspect_only:false}};
  const binding=await parity(preparedGraphRequest(inputs));assert.equal(binding.execution_performed,false);assert.equal(binding.review.native_build,build);
  const before=executions,prepared=await preparedEngineFromGraphBinding(inputs,binding);assert.equal(executions,before);assert.equal(prepared.settings.inspect_only,true);assert.equal(prepared.settings.chamber,'house');assert.equal(prepared.source.text,binding.bound_request_json);assert(prepared.source.text.includes('18446744073709551615'));
  const reviewed=await parity(preparedEngineRequest(prepared));assert.equal(reviewed.engine_request.options.seed,'18446744073709551615');
  const output=await parity(preparedEngineRequest({...prepared,settings:{...prepared.settings,inspect_only:false}}));assert.equal(output.export.document.plan.units.unit_kind,source.files.nativeAdjacencySettings.unit_kind??'tract');assert.equal(output.export.document.metadata.chamber,'house');
  const files={...source.files,preparedGraphInputs:inputs,preparedEngineInputs:prepared},project=createProject({name:'Native graph full request',files,operation:'bind-prepared-engine-graph',constraints:[],result:binding,lastOperation:'bind-prepared-engine-graph'});
  assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,files);assert.equal(executions,before+2);
  cases++;lastInputs=inputs;lastBinding=binding;
  if(name==='tract-prepared.bisect'&&!build){await fs.writeFile(root+'/binding.bisect',JSON.stringify(project));await fs.writeFile(root+'/bound-output.json',JSON.stringify(output));await fs.writeFile(root+'/template.json',text);}
  if(name==='ri-prepared.bisect'&&!build){await fs.writeFile(root+'/real-binding.bisect',JSON.stringify(project));await fs.writeFile(root+'/real-bound-output.json',JSON.stringify(output));await fs.writeFile(root+'/real-template.json',text);}
 }
}
const tract=JSON.parse(await fs.readFile('target/native-engine-input-wasm/tract-prepared.bisect','utf8')),selected=await preparedGraphFromNative(tract.result);
for(const build of [false,true]){
 const input={request:{graph:null,options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:null,seed:'18446744073709551615',seeds:1,steps:3,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1.1,iterations:10}},fine_level:'tract',coarse_level:'county',total_steps:3,alpha:.3,percentile:0,block_groups:null,adaptive:null};
 const inputs={...selected,template:{name:'multiscale-template.json',text:JSON.stringify({operation:build?'run-multiscale-build':'run-multiscale',input}).replace('"18446744073709551615"','18446744073709551615')},settings:{...selected.settings,created_at:'2026-10-17T00:00:00Z'}};
 const binding=await parity(preparedGraphRequest(inputs)),prepared=await preparedEngineFromGraphBinding(inputs,binding);assert.equal(binding.review.run_kind,'multiscale');assert.equal(binding.review.native_build,build);await parity(preparedEngineRequest(prepared));const output=await parity(preparedEngineRequest({...prepared,settings:{...prepared.settings,inspect_only:false}}));assert.equal(output.run_kind,'multiscale');assert.equal(output.native_build,build);assert(output.export);cases++;
}
for(const kind of ['precinct','imported'])for(const build of [false,true]){
 const prior=JSON.parse(await fs.readFile('target/prepared-engine-flow/declared-'+kind+'.bisect','utf8')),graph=prior.result.executed_request.graph;
 const request={graph:null,options:{structure:'nway',weights:'geographic',search:'single',districts:null,seed:'18446744073709551615',seeds:2,steps:3,percentile:0,alpha_county:0,balance_tolerance:50,area_swing:1.1,iterations:10}};
 const inputs={...structuredClone(PREPARED_GRAPH_DEFAULTS),districts:2,graph:{name:'manual-'+kind+'-graph.json',text:JSON.stringify(graph)},template:{name:'custom-template.json',text:JSON.stringify(build?{operation:'run-engine-build',request}:request).replace('"18446744073709551615"','18446744073709551615')},settings:{...PREPARED_GRAPH_DEFAULTS.settings,created_at:'2026-10-17T00:00:00Z'}};
 const binding=await parity(preparedGraphRequest(inputs)),prepared=await preparedEngineFromGraphBinding(inputs,binding);assert.equal(binding.unit_kind,kind);assert.equal(binding.year,graph.year);await parity(preparedEngineRequest(prepared));const output=await parity(preparedEngineRequest({...prepared,settings:{...prepared.settings,inspect_only:false}}));assert.equal(output.export.document.plan.units.unit_kind,kind);cases++;
}
for(const mutate of [r=>r.bound_request_json+=' ',r=>r.template_sha256='0'.repeat(64),r=>r.graph_sha256='0'.repeat(64),r=>r.districts=3,r=>r.execution_performed=true,r=>r.review.execution_performed=true,r=>r.review.schema_version='unknown',r=>r.review.csv_loading_performed=true,r=>r.review.units++,r=>r.review.engine_request.options.districts=3]){
 const bad=structuredClone(lastBinding);mutate(bad);await assert.rejects(preparedEngineFromGraphBinding(lastInputs,bad));refusals++;
}
for(const mutate of [v=>v.graph.text+=' ',v=>v.template.text+=' ',v=>v.districts=3]){const changed=structuredClone(lastInputs);mutate(changed);await assert.rejects(preparedEngineFromGraphBinding(changed,lastBinding));refusals++;}
for(const districts of [0,501,1.5]){assert.throws(()=>validatePreparedGraphInputs({...PREPARED_GRAPH_DEFAULTS,districts}));refusals++;}
const opaque={...structuredClone(PREPARED_GRAPH_DEFAULTS),graph:{name:'opaque-graph.json',text:'globalThis.graphExecuted=true;'},template:{name:'opaque-template.json',text:'globalThis.templateExecuted=true;'}};
const opened=parseProject(JSON.stringify(createProject({name:'Opaque binding sources',files:{preparedGraphInputs:opaque},operation:'bind-prepared-engine-graph',constraints:[],result:null,lastOperation:null})));assert.equal(opened.files.preparedGraphInputs.template.text,opaque.template.text);assert.equal(globalThis.templateExecuted,undefined);assert.throws(()=>engine.execute(preparedGraphRequest(opaque)));refusals++;
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,cases,native_host_comparisons:matches,refusals,native_binary_versions:[...formats].sort(),four_census_resolutions:kinds.size===4,explicit_multiscale_cases:2,manual_unit_kinds:['precinct','imported'],original_ri_tracts:250,original_binary_sources_and_template_retained:true,numeric_max_u64_preserved:true,all_handoffs_nonexecuting:true,handoff_forces_explicit_review:true,hash_and_scope_guards:true,inert_project_and_workbench_roundtrip:true},null,2));
console.log(`Native prepared graph templates: ${cases} graph cases, ${matches} Native preparation/binding/review/run matches, ${refusals} refusals; exact u64/options, original binary sources, explicit review and inert Save/Open passed.`);
