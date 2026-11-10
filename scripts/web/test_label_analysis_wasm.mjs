import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelAnalysisRequest,labelAnalysisZip,labelAnalysisFromBuild,readLabelAnalysisSource,labelAnalysisTypes} from '../../web/lab/label-analysis-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/label-analysis-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const input=JSON.parse(await fs.readFile('target/analysis-package-wasm/request.json','utf8')).input;
const expected=JSON.parse(await fs.readFile('target/analysis-package-wasm/result.json','utf8'));
// A second synthetic jurisdiction keeps the gate independent of downloaded state fixtures.
const riPlan=JSON.parse(input.document_json);riPlan.plan.units.state='RI';riPlan.metadata.jurisdiction='RI';
riPlan.plan.units.unit_ids=riPlan.plan.units.unit_ids.map(id=>'44'+id.slice(2));
const units=riPlan.plan.units;
const universe={canonical_order:units.canonical_order,state:units.state,unit_ids:units.unit_ids,unit_kind:units.unit_kind,year:units.year};
if(units.source_id!==undefined)universe.source_id=units.source_id;
units.unit_universe_hash='sha256:'+createHash('sha256').update(JSON.stringify(Object.fromEntries(Object.entries(universe).sort(([a],[b])=>a.localeCompare(b))))).digest('hex');
const ri=JSON.stringify(riPlan);
const sparse={...input,document_json:ri,context_json:null,geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,compactness:null};
const build={label:'analysis_fixture',year:'2020',created:'2026-10-07T00:00:00Z',version:'0.1',config_path:'config/analysis_fixture.yaml',config_sha256:'a'.repeat(64),command:'fixture',output_dir:'runs/analysis_fixture/2020',metis_engine:'rust',algorithm:{},states:{washington:{status:'ok',districts:3},rhode_island:{status:'ok',districts:JSON.parse(ri).plan.k},iowa:{status:'failed',error:'Fixture failed build'}},summary:{total:3,succeeded:2,failed:1}};
const request={operation:'run-label-analysis',input:{build_index_json:JSON.stringify(build,null,2)+'\n',created:'2026-10-07T00:00:00Z',states:{washington:input,rhode_island:sparse}}};
const sha=t=>createHash('sha256').update(t).digest('hex');
const out=engine.execute(request),index=JSON.parse(out.index_json);
assert.equal(index.run_index_sha256,sha(request.input.build_index_json));
assert.equal(index.states.washington.status,'ok');
for(const kind of index.types)assert.equal(index.states.washington[kind],'ok');
assert.equal(index.states.rhode_island.status,'unavailable');assert.equal(index.states.rhode_island.splits,'ok');assert.equal(index.states.rhode_island.compactness,'unavailable');
assert.equal(index.states.iowa.status,'skipped');assert.equal(out.registry_promoted,false);
for(const [name,text] of Object.entries(expected.files))assert.equal(out.files[`analysis/analysis_fixture/2020/washington/${name}`],text);
for(const [path,text] of Object.entries(out.files))assert.equal(out.file_sha256[path],sha(text));
const badGeometry={...request,input:{...request.input,states:{washington:{...input,compactness:{...input.compactness,unit_geometry_json:'{}'}}}}};
const failed=engine.execute(badGeometry);const failedIndex=JSON.parse(failed.index_json);assert.equal(failedIndex.states.washington.status,'failed');assert.equal(failedIndex.states.washington.compactness,'failed');assert.match(failedIndex.states.washington.error,/missing field `type`/);assert.equal(failed.files['analysis/analysis_fixture/2020/washington/partisan.json'],expected.files['partisan.json']);
const whitespace=engine.execute({...request,input:{...request.input,build_index_json:JSON.stringify(build)}});assert.notEqual(whitespace.index_sha256,out.index_sha256);assert.equal(JSON.parse(whitespace.index_json).run_index_sha256,sha(JSON.stringify(build)));
let rejections=0;
for(const patch of [{states:{}},{states:{outside:input}},{states:{rhode_island:input}},{states:{iowa:input}},{build_index_json:JSON.stringify({...build,year:'2010'})},{build_index_json:JSON.stringify({...build,label:'../escape'})},{build_index_json:JSON.stringify({...build,states:{washington:{status:'ok',districts:2}}})}]){assert.throws(()=>engine.execute({...request,input:{...request.input,...patch}}));rejections++;}
for(const [i,[req,result]]of [[request,out],[badGeometry,failed]].entries()){const path=`${root}/native-request-${i}.json`;await fs.writeFile(path,JSON.stringify(req));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:16*1024*1024})),result);}
await fs.writeFile(root+'/result.json',JSON.stringify(out));
const source={name:'batch.json',base64:Buffer.from(JSON.stringify(request.input)).toString('base64')},inputs={source,created:request.input.created};
assert.deepEqual(labelAnalysisRequest(inputs),request);assert.ok(countResultTables(out).some(t=>t.title==='State and analyzer outcomes'));assert.equal(countResultTables(out).find(t=>t.title==='Recorded state seeds').rows.find(r=>r[0]==='washington')[2],'18446744073709551615');assert.ok(countResultTables(out).some(t=>t.title==='Native diagnostic: washington / election'));
const selectedInputs={...inputs,states:['WASHINGTON']},selectedRequest=labelAnalysisRequest(selectedInputs),selectedResult=engine.execute(selectedRequest);
assert.deepEqual(Object.keys(selectedRequest.input.states),['washington']);assert.equal(selectedRequest.input.build_index_json,request.input.build_index_json);assert.equal(selectedInputs.source,inputs.source);assert.equal(JSON.parse(selectedResult.index_json).states.rhode_island.status,'skipped');
await fs.writeFile(root+'/selected-request.json',JSON.stringify(selectedRequest));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/selected-request.json'],{encoding:'utf8',maxBuffer:16*1024*1024})),selectedResult);
assert.deepEqual(labelAnalysisRequest({...inputs,states:[]}),request);
for(const states of [['missing'],['washington','WASHINGTON'],['../washington'],Array(61).fill('washington'),null])assert.throws(()=>labelAnalysisRequest({...inputs,states}));
for(const [i,type]of labelAnalysisTypes.entries()){
 const req=labelAnalysisRequest({...selectedInputs,types:[type]}),result=engine.execute(req),path=`${root}/type-request-${i}.json`;await fs.writeFile(path,JSON.stringify(req));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:16*1024*1024})));
 assert.deepEqual(JSON.parse(result.index_json).types,[type]);assert.deepEqual(result.states[0].steps.map(step=>step.name),[type]);assert.equal(countResultTables(result).find(t=>t.title==='State batch analysis').rows.find(r=>r[0]==='Requested analyzers')[1],type);assert.equal(countResultTables(result).find(t=>t.title==='State and analyzer outcomes').rows.find(r=>r[0]==='washington').slice(2,8).filter(v=>v==='Not selected').length,5);
 for(const [name,text]of Object.entries(result.files))if(name!==result.index_path&&!name.endsWith('/analysis-package.json'))assert.equal(text,out.files[name]);
}
assert.deepEqual(engine.execute({...request,input:{...request.input,types:['all']}}),out);assert.deepEqual(labelAnalysisRequest({...inputs,types:[]}),request);
const onlyStatistics=engine.execute({...badGeometry,input:{...badGeometry.input,types:['statistics']}});assert.equal(JSON.parse(onlyStatistics.index_json).states.washington.status,'ok');assert.deepEqual(onlyStatistics.states[0].steps.map(step=>step.name),['statistics']);assert.equal(Object.keys(onlyStatistics.files).some(name=>name.endsWith('/compactness.json')),false);
for(const types of [['unknown'],['statistics','statistics'],['all','statistics'],[],['../statistics']])assert.throws(()=>engine.execute({...request,input:{...request.input,types}}));
for(const types of [['unknown'],['statistics','statistics'],['all'],null])assert.throws(()=>labelAnalysisRequest({...inputs,types}));
const project=createProject({name:'State batch analyses',files:{labelAnalysisInputs:inputs},operation:'run-label-analysis',constraints:[],result:out,lastOperation:'run-label-analysis'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/project.bisect',JSON.stringify(project));
await fs.writeFile(root+'/analysis.zip',await labelAnalysisZip(out));
await assert.rejects(labelAnalysisZip({...out,files:{...out.files,[out.index_path]:'{}'}}),/index/);
await assert.rejects(labelAnalysisZip({...out,files:{...out.files,'analysis/analysis_fixture/2020/washington/../escape':'{}'}}),/count|path/);
const filePath='analysis/analysis_fixture/2020/washington/partisan.json';await assert.rejects(labelAnalysisZip({...out,files:{...out.files,[filePath]:'{}'}}),/hash/);
let oversizedRead=false;await assert.rejects(readLabelAnalysisSource({size:25*1024*1024+1,arrayBuffer:()=>{oversizedRead=true;}}));assert.equal(oversizedRead,false);
const generated={schema_version:'bisect-label-build-v1',label:build.label,files:{['runs/'+build.label+'/2020/index.json']:request.input.build_index_json}};
for(const [name,input]of Object.entries(request.input.states)){generated.files[`runs/${build.label}/2020/${name}/plan.rplan`]=input.document_json;generated.files[`runs/${build.label}/2020/${name}/context.rctx`]=input.context_json||'{}';}
const prepared=labelAnalysisFromBuild(generated,`runs/${build.label}/2020/washington/plan.rplan`);assert.equal(prepared.created,'');const preparedRequest=labelAnalysisRequest({...prepared,created:request.input.created});assert.deepEqual(Object.keys(preparedRequest.input.states),['washington','rhode_island']);assert.equal(preparedRequest.input.build_index_json,request.input.build_index_json);
const selectedPath=`runs/${build.label}/2020/washington/plan.rplan`;
const recorded=(configured,engine)=>{const next=structuredClone(generated),plan=JSON.parse(input.document_json);plan.provenance={...plan.provenance,producer:{...plan.provenance?.producer,algorithm_configuration:{execution_tolerance_percent:configured},engine_options:{balance_tolerance:engine}}};next.files[selectedPath]=JSON.stringify(plan);return next;};
for(const tolerance of [0,0.5,25,100]){const next=recorded(tolerance,tolerance),handoff=labelAnalysisRequest({...labelAnalysisFromBuild(next,selectedPath),created:request.input.created});assert.equal(handoff.input.states.washington.tolerance_percent,tolerance);assert.equal(handoff.input.states.washington.document_json,next.files[selectedPath]);}
for(const tolerance of [-1,101,'25',null])assert.throws(()=>labelAnalysisFromBuild(recorded(tolerance,tolerance),selectedPath),/recorded population tolerances/);
assert.throws(()=>labelAnalysisFromBuild(recorded(25,0.5),selectedPath),/conflicting/);
assert.equal(preparedRequest.input.states.washington.tolerance_percent,0.5);
const hugeSeedBuild=request.input.build_index_json.replace('"algorithm": {}','"algorithm": {"seed":18446744073709551615}');const nativeSeed=labelAnalysisRequest({...inputs,source:{...source,base64:Buffer.from(JSON.stringify({...request.input,build_index_json:hugeSeedBuild})).toString('base64')}});assert.equal(nativeSeed.input.build_index_json,hugeSeedBuild);
await fs.writeFile(root+'/evidence.json',JSON.stringify({two_state_execution:true,state_selection:true,type_selection:true,unselected_analyzers_not_executed:true,original_batch_preserved:true,exact_native_package_files:true,native_host_parity_cases:9,unavailable_and_skipped:true,failed_outputs_retained:true,raw_build_index_hash:true,rejections},null,2));
console.log(`Label analysis: two states, native flattened index, exact package files, raw-byte chain, retained failures, ${rejections} rejection cases and nine native/WASM parity cases including state and type selection passed.`);
