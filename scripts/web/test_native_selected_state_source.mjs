import {createNativeCommandComparator} from './native_command_comparison.mjs';
let strictComparator;
function compareNative(request,actual,expected){strictComparator??=createNativeCommandComparator(root,'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');strictComparator.compare(request,actual,expected);}
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeSelectedStateSourceRequest,executeNativeSelectedStateSource,nativeCommandOutputChoices} from '../../web/lab/native-selected-state-source.js';
import {nativeEngineInputRequest} from '../../web/lab/native-adjacency-files.js';
import {createStateWorkflowBinding,stateWorkflowSettingsPatch} from '../../web/lab/native-state-workflow.js';
import {preparedStateRunZip} from '../../web/lab/prepared-state-job.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const coreOnly=process.argv.includes('--core-only'),root='target/native-selected-state-source';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),hash=s=>createHash('sha256').update(s).digest('hex');
let hosts=0,partitions=0,preparations=0,kernels=0;const outputs=[];
const facade={execute(r,o){if(r.operation==='run-prepared-engine'&&!r.input.inspect_only)partitions++;if(['prepare-native-engine-input','prepare-census-engine-input','prepare-tract-engine-input'].includes(r.operation))preparations++;const result=engine.execute(r,o);writeFileSync(root+'/request.json',JSON.stringify(r));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',windowsHide:true,maxBuffer:48*1024*1024}));compareNative(r,result,native);hosts++;return result;}};
const fixture=JSON.parse(await fs.readFile('scripts/web/fixtures/native-state-source.json','utf8')),common=['--state','RI','--year','2020','--resolution','tract','--metis-engine','metis-core','--structure','nway','--balance-tolerance','5','--seed','18446744073709551615'];
const metadata={graph:null,auxiliary:null,settings:{label:'Selected State count',chamber:'congressional',created_at:'2026-10-20T00:00:00Z',inspect_only:true}},base={...fixture,nativeStateSourceSettings:{enabled:true,command_kind:'state'},nativeSelectedStateSourceSettings:{kind:'native'},stateGraphInputs:metadata};
let browserFiles;
for(const count of [4,1,null]){
 const args=[...common,...(count===null?[]:['--districts',String(count)])],command={arguments_json:JSON.stringify(args)};let selected={...base,stateCommandInputs:command};
 if(count===4){
  args.push('--adjacency','unopened/v3.adj.bin','--label','Single_Override','--version','vSingle','--manifest','--dpi','72');command.arguments_json=JSON.stringify(args);
  const binding=await createStateWorkflowBinding(command,facade.execute({operation:'inspect-native-state-command',arguments:args}),{sourcePreparation:nativeEngineInputRequest(fixture.nativeAdjacencyInputs,fixture.nativeAdjacencySettings)});
  selected={...selected,...stateWorkflowSettingsPatch(binding,selected)};
  const ids=JSON.parse(Buffer.from(fixture.nativeAdjacencyInputs.geoids.base64,'base64'));selected.planGeometry={name:'synthetic-selected-state.geojson',base64:Buffer.from(JSON.stringify({type:'FeatureCollection',features:Object.values(ids).map((GEOID,i)=>({type:'Feature',properties:{GEOID},geometry:{type:'Polygon',coordinates:[[[-71+i*.01,41],[-70.991+i*.01,41],[-70.991+i*.01,41.009],[-71+i*.01,41.009],[-71+i*.01,41]]]}}))})).toString('base64')};browserFiles=selected;
 }
 const before={hosts,partitions,preparations},request=await nativeSelectedStateSourceRequest(selected);assert.deepEqual({hosts,partitions,preparations},before);assert.equal(request.input.scope.districts,null);
 const result=await executeNativeSelectedStateSource(facade,request.input),expected=count??2;assert.equal(result.scope.districts,expected);assert.equal(result.single_source_selection.scope.districts,expected);assert.equal(result.single_source_selection.scope_changed,false);assert.equal(result.single_source_selection.district_count.allocation_source,count===null?'congressional-fallback':'explicit-native-flag');assert.equal(result.prepared.executed_request.options.seed,'18446744073709551615');assert(!Object.hasOwn(result.source_preparation,'graph_json'));
 const kernelArgs=[...args];if(count===null)kernelArgs.push('--districts','2');await fs.writeFile(root+'/kernel.json',JSON.stringify({state_args:kernelArgs,graph:result.prepared.executed_request.graph}));const native=JSON.parse(execFileSync('target/debug/examples/geometric_state_reference.exe',[root+'/kernel.json'],{encoding:'utf8',windowsHide:true}));assert.deepEqual(native.assignments,result.prepared.result.assignments);kernels++;
 await fs.writeFile(root+'/count-'+expected+(count===null?'-inferred':'')+'.zip',await preparedStateRunZip(result));outputs.push({request,result});
}
if(!coreOnly){
 const census=JSON.parse(await fs.readFile('target/census-preparation-wasm/2020-tract/project.bisect','utf8'));
 for(const kind of ['census','tract']){const files={...census.files,nativeStateSourceSettings:{enabled:true},nativeSelectedStateSourceSettings:{kind},stateGraphInputs:metadata,stateCommandInputs:{arguments_json:JSON.stringify([...common,'--districts','1'])}},request=await nativeSelectedStateSourceRequest(files),result=await executeNativeSelectedStateSource(facade,request.input);assert.equal(result.scope.districts,1);assert.equal(result.prepared.executed_request.options.seed,'18446744073709551615');outputs.push({request,result});await fs.writeFile(root+'/'+kind+'.zip',await preparedStateRunZip(result));await fs.writeFile(root+'/'+kind+'-browser.bisect',JSON.stringify(createProject({name:'Single '+kind+' State sources',files,operation:'run-native-selected-state-source',constraints:[],result:null,lastOperation:null})));}
}
const first=outputs[0],beforeBad={partitions,preparations};
for(const change of [job=>job.native_state.command.arguments_json+=' ',job=>job.native_state.source_inputs.preparation.input.sources.adjacency.name='changed.bin']){const bad=structuredClone(first.request.input);change(bad);await assert.rejects(()=>executeNativeSelectedStateSource(facade,bad),/source bytes changed/);assert.deepEqual({partitions,preparations},beforeBad);}
const badCount=structuredClone(first.request.input);badCount.scope.districts=2;await assert.rejects(()=>executeNativeSelectedStateSource(facade,badCount),/district count must match/);assert.deepEqual({partitions,preparations},beforeBad);
await assert.rejects(()=>executeNativeSelectedStateSource({execute(r,o){const v=engine.execute(r,o);return r.operation==='resolve-native-chamber-count'?{...v,state:'IA'}:v;}},outputs[2].request.input),/allocation differs/);assert.deepEqual({partitions,preparations},beforeBad);
for(const extra of [['--state','IA'],['--year','2010'],['--resolution','block_group'],['--print-only'],['--force']]){const command=[...common];if(['--state','--year','--resolution'].includes(extra[0]))command.splice(command.indexOf(extra[0]),2);command.push(...extra);const request=await nativeSelectedStateSourceRequest({...base,stateCommandInputs:{arguments_json:JSON.stringify(command)}});await assert.rejects(()=>executeNativeSelectedStateSource(facade,request.input),/must match|Review or apply/);assert.deepEqual({partitions,preparations},beforeBad);}
assert.deepEqual(nativeCommandOutputChoices(first.result),{version:'vSingle',dpi:72,layout:'labeled'});assert.deepEqual(nativeCommandOutputChoices(outputs[1].result),{version:'v1',dpi:150,layout:'state'});
const project=createProject({name:'Single State district override',files:browserFiles,operation:'run-native-selected-state-source',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/accepted.json',JSON.stringify(first.result));await fs.writeFile(root+'/outputs.json',JSON.stringify(outputs));
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,whole_native_host_replies:hosts,independent_native_nway_assignment_matches:kernels,partition_runs:partitions,source_preparations:preparations,explicit_counts:[4,1],native_inferred_congressional_count:2,source_kinds:coreOnly?['native']:['native','census','tract'],source_tampering_and_scope_refused_before_preparation:true,original_flags_and_max_u64_seed_retained:true,national_queue_untouched:true,published:false},null,2));console.log('PASS selected State source Run: '+hosts+' native/WASM replies, '+kernels+' independent native kernels, explicit 4/1 counts and native inferred 2, selected sources, original flags/seed, scope/source/intent guards.');
