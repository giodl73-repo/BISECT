import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {STATE_GRAPH_DEFAULTS,stateGraphRequest,stateGraphDefaultCommand,stateGraphFromPreparedResult,preparedEngineFromStateGraph,validateStateGraphInputs,readStateGraphSource} from '../../web/lab/state-graph-files.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const coreOnly=process.argv.includes('--core-only');
const root=path.resolve('target/native-state-graph'+(coreOnly?'-core':''));await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),old=JSON.parse(await fs.readFile('target/native-state-run/records.json','utf8'));
const records=[];let matches=0,runs=0,sourcePreparations=0,realStateRuns=0;
async function native(request,name){const file=root+'/'+name+'.request.json';await fs.writeFile(file,JSON.stringify(request));const p=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(p.status,0,p.stderr);return JSON.parse(p.stdout);}
async function check(name,command,inputs,{execution,run=false}={}){
 const request=stateGraphRequest(command,inputs),result=engine.execute(request);assert.equal(result.execution_performed,false);assert.equal(result.filesystem_reads_performed,false);assert.equal(result.state_graph_source.graph_source_name,inputs.graph.name);
 const selected=await preparedEngineFromStateGraph(command,inputs,result);assert.equal(selected.settings.inspect_only,true);
 if(!process.argv.includes('--prepare')){const expected=await native(request,name);compareNative(request,result,expected);matches++;}
 let actualRun=null;
 if(execution||run){const runRequest={operation:'run-prepared-engine',input:{request_json:selected.source.text,...selected.settings,inspect_only:false}};actualRun=engine.execute(runRequest);assert.equal(actualRun.execution_performed,true);assert(actualRun.export?.document_json,actualRun.export_error);if(execution){assert.deepEqual(actualRun.result,execution.result);assert.deepEqual(actualRun.export,execution.export);assert.deepEqual(actualRun.executed_request,execution.executed_request);runs++;}else{assert.equal(actualRun.result.metrics.population,1097379);assert.equal(actualRun.executed_request.graph.geoids.length,250);realStateRuns++;}
 if(!process.argv.includes('--prepare')){compareNative(runRequest,actualRun,await native(runRequest,name+'.run'));matches++;}}
 records.push({name,request,result,actualRun});return {selected,result};
}
// The new envelope must preserve all established ordinary State configurations.
for(const row of old){
 const template=JSON.parse(row.request.input.prepared.request_json);if(template.operation?.includes('multiscale'))continue;
 const r=template.request??template,aux=Object.fromEntries(['demographics','partisan','elections','character','native_character_source','coi'].filter(k=>r[k]!=null).map(k=>[k,r[k]]));
 const inputs={graph:{name:row.name+'.graph.json',text:JSON.stringify(r.graph)},auxiliary:Object.keys(aux).length?{name:row.name+'.data.json',text:JSON.stringify(aux)}:null,settings:{...row.request.input.prepared,inspect_only:true}};delete inputs.settings.request_json;delete inputs.settings.context_source;
 await check(row.name,{arguments_json:JSON.stringify(row.request.input.arguments)},inputs,{execution:row.execution});
}
// Actual source-to-graph preparation, not hand-constructed graph result receipts.
if(!coreOnly){
for(const kind of ['tract','block-group','block','ri']){
 const p=JSON.parse(await fs.readFile('target/native-engine-input-wasm/'+kind+'-prepared.bisect','utf8')),request=nativeEngineInputRequest(p.files.nativeAdjacencyInputs,p.files.nativeAdjacencySettings),prepared=engine.execute(request);sourcePreparations++;
 const next=await stateGraphFromPreparedResult(prepared),args=JSON.parse(next.command.arguments_json);assert.equal(args[args.indexOf('--resolution')+1],kind==='block-group'?'block_group':kind==='ri'?'tract':kind);assert.equal(next.inputs.graph.text,prepared.graph_json);
 await check('native-'+kind,{arguments_json:JSON.stringify([...args,'--search','single','--districts','2','--balance-tolerance',kind==='ri'?'5':'25'])}, {...next.inputs,settings:{...next.inputs.settings,created_at:'2026-10-20T00:00:00Z'}},{run:kind==='ri'});
 if(kind==='tract')await fs.writeFile(root+'/native-source.bisect',JSON.stringify(createProject({name:'Native graph to State flags',files:{nativeAdjacencyInputs:p.files.nativeAdjacencyInputs,nativeAdjacencySettings:p.files.nativeAdjacencySettings},operation:'prepare-native-engine-input',constraints:[],result:prepared,lastOperation:'prepare-native-engine-input'})));
}
for(const kind of ['tract','block-group','block']){
 const request=JSON.parse(await fs.readFile('target/census-preparation-wasm/2020-'+kind+'/native-request.json','utf8')),prepared=engine.execute(request);sourcePreparations++;
 const next=await stateGraphFromPreparedResult(prepared);assert(next.geometry);assert.equal(Buffer.from(next.geometry.base64,'base64').toString(),prepared.geometry_json);assert.equal(next.inputs.graph.text,prepared.graph_json);await check('census-'+kind,next.command,next.inputs);
}
const contextRequest=JSON.parse(await fs.readFile('target/context-engine-input/tract-prepared.request.json','utf8')),context=engine.execute(contextRequest);sourcePreparations++;
await assert.rejects(stateGraphFromPreparedResult(context),/original RCTX/);const nextContext=await stateGraphFromPreparedResult(context,{context_source:contextRequest.input});assert.deepEqual(nextContext.inputs.context_source,contextRequest.input);const contextCommand={arguments_json:JSON.stringify([...JSON.parse(nextContext.command.arguments_json),'--search','single'])};const compiledContext=await check('context',contextCommand,nextContext.inputs);await assert.rejects(preparedEngineFromStateGraph(contextCommand,{...nextContext.inputs,context_source:{...nextContext.inputs.context_source,source_name:'changed.rctx'}},compiledContext.result),/RCTX source changed/);
}
const sample=records.find(r=>r.name==='max-seed'),inputs={...structuredClone(STATE_GRAPH_DEFAULTS),graph:{name:sample.request.input.graph_source_name,text:sample.request.input.graph_json},settings:{label:sample.request.input.label,chamber:sample.request.input.chamber,created_at:sample.request.input.created_at,inspect_only:true}},command={arguments_json:JSON.stringify(sample.request.input.arguments)};
const selected=await preparedEngineFromStateGraph(command,inputs,sample.result);assert.match(selected.source.text,/18446744073709551615/);
for(const key of ['graph_json','auxiliary_json']){const request=stateGraphRequest(command,inputs);request.input[key]=key==='graph_json'?'{}':'{"options":{}}';if(key==='auxiliary_json')request.input.auxiliary_source_name='untrusted.json';assert.throws(()=>engine.execute(request));}
for(const text of ['{"coi":null,"coi":null}','{"__proto__":{}}'])assert.throws(()=>stateGraphRequest(command,{...inputs,auxiliary:{name:'ambiguous.json',text}}),/Duplicate|Unsafe/);
await assert.rejects(preparedEngineFromStateGraph(command,{...inputs,graph:{...inputs.graph,text:inputs.graph.text+' '}},sample.result),/changed/);await assert.rejects(preparedEngineFromStateGraph(command,inputs,{...sample.result,prepared_request_json:sample.result.prepared_request_json+' '}),/changed/);
assert.throws(()=>validateStateGraphInputs({...inputs,extra:1}));assert.throws(()=>stateGraphDefaultCommand({...inputs,graph:{name:'county.json',text:JSON.stringify({...JSON.parse(inputs.graph.text),unit_kind:'county'})}}),/Other unit kinds/);
await assert.rejects(readStateGraphSource({name:'large.json',size:25*1024*1024+1,arrayBuffer(){throw Error('Must not read oversized file');}}),/at most25MiB/);
const project=createProject({name:'State graph input source',files:{stateGraphInputs:inputs,stateCommandInputs:command},operation:'prepare-native-state-graph-run',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/graph.bisect',JSON.stringify(project));await fs.writeFile(root+'/records.json',JSON.stringify(records));await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,compiled_cases:records.length,unchanged_prepared_state_runs_and_exports:runs,whole_native_host_matches:matches,real_source_graph_preparations:sourcePreparations,real_ri_tract_population:realStateRuns?1097379:null,real_ri_tract_units:realStateRuns?250:null,real_state_runs_and_exports:realStateRuns,core_only:coreOnly,three_native_census_resolutions:!coreOnly,selected_rctx_retained:!coreOnly,no_manual_engine_template:true,inert_preparation:true,source_hash_and_markup_guards:true,duplicate_and_unsafe_browser_json_refused:true,oversized_pre_read_guard:true,published:false},null,2)+'\n');console.log(`PASS native State graph: ${records.length} compilations, ${runs} unchanged runs/exports, ${sourcePreparations} real source preparations, ${matches} whole host matches`);
