// Explicit retained original Census inputs, never substitute synthetic fixtures.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createPreparedCensusJob,executePreparedCensusJob} from '../../web/lab/prepared-census-job.js';
import {preparedStateRunMap,preparedStateRunZip} from '../../web/lab/prepared-state-job.js';
import {preparedEngineReplayInputs,preparedEnginePlanSources} from '../../web/lab/prepared-engine-results.js';
import {engineReplayRequest} from '../../web/lab/engine-replay-files.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';

const root='target/prepared-census-real';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const host=path.resolve('target/release/examples/execute_tool_request.exe');
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
let comparisons=0,executions=0;
const oracle={execute(request,options){
 console.log('WASM '+request.operation);const actual=engine.execute(request,options);executions++;
 if(request.operation!=='prepare-census-engine-input'){
  console.log('Native '+request.operation);writeFileSync(root+'/request.json',JSON.stringify(request));
  const expected=JSON.parse(execFileSync(host,[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:192*1024*1024,stdio:['ignore','pipe','pipe']}));
  assert.deepEqual(actual,expected);comparisons++;
 }
 return actual;
}};
const results=[];
for(const [kind,units,tolerance]of [['county',5,50],['block-group',792,0.5],['block',25649,0.5]]){
 console.log('Original RI 2020 '+kind);const start=performance.now();
 const prior=JSON.parse(await fs.readFile(`target/real-census-preparation/RI/${kind}/run.bisect-workbench`,'utf8'));
 const files=structuredClone(prior.files);
 const template=JSON.stringify({operation:'run-engine-build',request:{graph:null,options:{structure:kind==='block-group'?'nway':'standard-bisect',weights:'geographic',search:'single',districts:null,seed:'18446744073709551615',seeds:2,steps:100,percentile:0,alpha_county:0,balance_tolerance:tolerance,area_swing:1.1,iterations:10,metis_trials:3,...(kind==='block-group'?{metis_ufactor:30,metis_objective:'volume'}:{}),capture_rounds:true}}}).replace('"18446744073709551615"','18446744073709551615');
 files.preparedCensusTemplateInputs={enabled:true,use_display_geometry:false,source:{name:'ri-'+kind+'-template.json',text:template},settings:{label:'Original RI full-template '+kind,chamber:'congressional',created_at:'2026-10-17T00:00:00Z',inspect_only:false}};
 const job=createPreparedCensusJob(files);job.scope.districts=2;
 const output=executePreparedCensusJob(oracle,job);await fs.writeFile(root+'/'+kind+'-output.json',JSON.stringify(output));
 const prepared=output.source_preparation,result=output.result,graph=JSON.parse(prepared.graph_json),assignments=result.prepared.result.assignments;
 assert.equal(prepared.units,units);assert.equal(prepared.population,'1097379');
 assert.equal(result.source_review.engine_request.options.seed,'18446744073709551615');
 assert.equal(result.template_source.source.text,template);assert(result.request_source.source.text.includes('18446744073709551615'));
 assert(result.prepared.export,JSON.stringify(result.prepared.export_error));assert.equal(result.geometry_error,undefined);
 // Independently recompute population and graph contiguity from final assignments.
 const populations=[0,0],members=[[],[]];
 for(const [i,id]of graph.geoids.entries()){const d=assignments[id];assert(d===1||d===2);populations[d-1]+=graph.population[i];members[d-1].push(i);}
 for(const [d,vertices]of members.entries()){
  assert(vertices.length);const visited=new Set([vertices[0]]),pending=[vertices[0]];
  while(pending.length){for(const v of graph.adjacency[pending.pop()])if(assignments[graph.geoids[v]]===d+1&&!visited.has(v)){visited.add(v);pending.push(v);}}
  assert.equal(visited.size,vertices.length);assert(Math.abs(populations[d]/(1097379/2)-1)*100<=tolerance);
 }
 assert.equal(populations[0]+populations[1],1097379);
 const map=preparedStateRunMap(result);assert.equal(map.units,units);
 for(const feature of map.features)assert.equal(feature.district+1,assignments[feature.id]);
 const nativeSources=preparedEnginePlanSources(result.prepared);assert.equal(Buffer.from(nativeSources.auditContextText.base64,'base64').toString(),result.prepared.export.context_json);
 const replay=oracle.execute(engineReplayRequest(await preparedEngineReplayInputs(result.prepared)));assert.equal(replay.status,'match');
 await fs.writeFile(root+'/'+kind+'-prepared-run.zip',preparedStateRunZip(result));
 // Original source files plus result must fit the real Save/Open boundary.
 const before=executions,archive=createWorkbenchArchive({name:'Original RI '+kind+' full template',files,operation:'run-state-batch',constraints:[],result:{schema_version:'bisect-state-run-batch-v1',status:'completed',states:[{state_code:'RI',status:'ok',result,source_preparation:prepared}]},lastOperation:'run-state-batch'});
 const restored=parseWorkbenchArchive(archive);assert.equal(executions,before);assert.deepEqual(restored.files.preparedCensusTemplateInputs,files.preparedCensusTemplateInputs);assert.deepEqual(restored.result.states[0].result,result);
 await fs.writeFile(root+'/'+kind+'.bisect-workbench',archive);
 const receipt={state:'RI',year:'2020',unit_kind:kind,units,population:prepared.population,tolerance_percent:tolerance,district_populations:populations,graph_sha256:prepared.graph_sha256,geometry_sha256:prepared.geometry_sha256,template_sha256:digest(template),source_identities:prepared.sources,accepted_seed:result.prepared.executed_request.options.seed,graph_bytes:Buffer.byteLength(prepared.graph_json),geometry_bytes:Buffer.byteLength(prepared.geometry_json),archive_bytes:Buffer.byteLength(archive),wasm_memory_bytes:engine.memoryBytes(),elapsed_seconds:(performance.now()-start)/1000,native_comparisons:comparisons};
 results.push(receipt);await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_host_comparisons:comparisons,real_source_preparation:true,native_comparison_uses_actual_prepared_graph:true,original_sources_retained:true,independent_population_and_contiguity_checks:true,all_units_mapped:true,accepted_replay_matches:true,inert_save_open_roundtrip:true,cases:results},null,2));console.log(JSON.stringify(receipt));
}
console.log(`Original Census full-template flow: ${results.length} RI resolutions through preparation/binding/native build, ${comparisons} Native comparisons, independent population/contiguity, all-unit maps, exact replay and inert source-preserving Save/Open passed.`);
