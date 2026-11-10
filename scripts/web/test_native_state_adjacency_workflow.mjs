import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {createStateWorkflowBinding,stateWorkflowSettingsPatch,recheckStateWorkflowBinding,hasStateWorkflowChoices} from '../../web/lab/native-state-workflow.js';
import {createNativeStateSourceJob} from '../../web/lab/native-state-sources.js';
import {executeNativeStateJob,validateNativeStateJob} from '../../web/lab/native-state-job.js';
import {preparedStateRunZip} from '../../web/lab/prepared-state-job.js';
import {acquireStateSourceJob} from '../../web/lab/state-source-acquisition.js';
import {stateSourceWorkbench} from '../../web/lab/local-state-output.js';
import {parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/native-state-adjacency-workflow';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const hash=text=>createHash('sha256').update(text).digest('hex');
let hosts=0,cuts=0,preparations=0,fetches=0;const operations=[];
const facade={execute(r,o){operations.push(r.operation);if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)cuts++;if(['prepare-native-engine-input','prepare-native-adjacency-state-run'].includes(r.operation))preparations++;const result=engine.execute(r,o);writeFileSync(root+'/request.json',JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024}));compareNative(r,result,native);hosts++;return result;}};
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),preparation=nativeEngineInputRequest(fixture.nativeAdjacencyInputs,fixture.nativeAdjacencySettings);
const args=['--state','RI','--year','2020','--resolution','tract','--metis-engine','metis-core','--structure','nway','--districts','2','--balance-tolerance','5','--seed','18446744073709551615','--adjacency','unopened/source/v3.adj.bin'];
const command={arguments_json:JSON.stringify(args)},review=facade.execute({operation:'inspect-native-state-command',arguments:args});assert(hasStateWorkflowChoices(review));
await assert.rejects(()=>createStateWorkflowBinding(command,review),/Select the native adjacency/);
const binding=await createStateWorkflowBinding(command,review,{sourcePreparation:preparation}),files={...fixture,stateCommandInputs:command,stateGraphInputs:{graph:null,auxiliary:null,settings:{label:'Selected binary override',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}}},patch=stateWorkflowSettingsPatch(binding,files),selected={...files,...patch};
assert.equal(binding.source_override.adjacency_path_intent,'unopened/source/v3.adj.bin');assert.equal(binding.source_override.preparation_sha256,hash(JSON.stringify(preparation)));assert(!binding.engine_command.arguments_json.includes('--adjacency'));assert.equal(cuts,0);assert.equal(preparations,0);
await recheckStateWorkflowBinding(facade,binding,binding.engine_command,{sourcePreparation:preparation,forExecution:true});
const job=await createNativeStateSourceJob(selected,'native');job.scope.districts=2;validateNativeStateJob(job);const accepted=await executeNativeStateJob(facade,job);assert.equal(cuts,1);assert.equal(preparations,1);assert.equal(accepted.result.prepared.executed_request.options.seed,'18446744073709551615');assert.equal(accepted.source_preparation.units,4);assert.equal(accepted.result.native_state.recipe.workflow_binding.source_command.arguments_json,command.arguments_json);
assert.equal(operations.filter(op=>op==='prepare-native-adjacency-state-run').length,1);assert.equal(operations.filter(op=>op==='prepare-native-engine-input').length,0);
// Independently parsed native CLI State compositor and n-way kernel, not the
// prepared request compiler. The separate native-host preparation reads the
// selected binary bytes; no desktop path discovery is claimed here.
const graph=accepted.result.prepared.executed_request.graph;await fs.writeFile(root+'/kernel.json',JSON.stringify({state_args:args,graph}));const independent=JSON.parse(execFileSync('target/debug/examples/geometric_state_reference.exe',[root+'/kernel.json'],{encoding:'utf8',windowsHide:true}));assert.deepEqual(accepted.result.prepared.result.assignments,independent.assignments);
await fs.writeFile(root+'/accepted.json',JSON.stringify(accepted));await fs.writeFile(root+'/accepted.zip',await preparedStateRunZip(accepted.result));
const restored=parseWorkbenchArchive(new TextDecoder().decode(stateSourceWorkbench(job,accepted)));assert.deepEqual(restored.files.stateWorkflowBinding,binding);assert.deepEqual(restored.files.nativeAdjacencyInputs,fixture.nativeAdjacencyInputs);assert.equal(restored.result,null);await fs.writeFile(root+'/source-project.bisect',JSON.stringify(restored));
const project=createProject({name:'Selected adjacency override',files,operation:'inspect-native-state-command',constraints:[],result:null,lastOperation:null});await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));assert.deepEqual(parseProject(JSON.stringify(createProject({...project,files:selected}))).files.stateWorkflowBinding,binding);
for(const mutate of [p=>p.input.sources.adjacency.base64=p.input.sources.adjacency.base64.replace('UkFESg','AkFESg'),p=>p.input.sources.geoids.name='changed.json',p=>p.input.state='VT',p=>p.input.year='2010']){const bad=structuredClone(job);mutate(bad.native_state.source_inputs.preparation);bad.source_sha256=hash(JSON.stringify(bad.native_state));const before={cuts,preparations};await assert.rejects(()=>executeNativeStateJob(facade,bad),/adjacency|must match|source scope changed/i);assert.deepEqual({cuts,preparations},before);}
const changed=structuredClone(selected);changed.nativeAdjacencyInputs.adjacency.name='renamed.adj.bin';await assert.rejects(()=>createNativeStateSourceJob(changed,'native'),/changed/);
const wrongName={arguments_json:JSON.stringify([...args.slice(0,-2),'--adjacency','unopened/wrong.adj.bin'])};await assert.rejects(()=>createStateWorkflowBinding(wrongName,facade.execute({operation:'inspect-native-state-command',arguments:JSON.parse(wrongName.arguments_json)}),{sourcePreparation:preparation}),/exact filename/);
const graphJob=structuredClone(job);delete graphJob.native_state.source_inputs;graphJob.native_state.graph_inputs={...files.stateGraphInputs,graph:{name:'graph.json',text:JSON.stringify(graph)}};graphJob.request_sha256=accepted.result.native_state.request_sha256;graphJob.source_sha256=hash(JSON.stringify(graphJob.native_state));await assert.rejects(()=>executeNativeStateJob(facade,graphJob),/selected native binary source recipe/);
const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:['RI'],year:'2020',data_types:[],selected_paths:[]}});await assert.rejects(()=>acquireStateSourceJob(plan,{state:'RI',year:'2020'},selected,null,null,null,{fetcher:()=>{fetches++;throw Error('Must not fetch');}}),/cannot acquire/);assert.equal(fetches,0);
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,whole_native_host_replies:hosts,independent_native_nway_assignment_matches:1,partition_runs:cuts,selected_binary_preparations:preparations,original_command_and_exact_seed_retained:true,all_selected_companions_and_scope_hash_bound:true,changed_sources_refused_before_preparation:true,graph_template_substitution_refused:true,acquisition_substitution_refused_before_fetch:true,inert_source_restore:true,path_opened:false,published:false},null,2));
console.log('PASS native State --adjacency handoff: '+hosts+' whole native/WASM replies, independent native n-way assignments, exact selected binary/companion binding, original flags/seed/ZIP/source restoration and source substitution guards.');
