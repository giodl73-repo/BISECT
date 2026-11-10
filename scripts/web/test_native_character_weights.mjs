import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeCharacterWeightRequest,validateNativeCharacterWeightInputs} from '../../web/lab/native-character-weight-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/native-character-weights';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),fixtures=JSON.parse(await fs.readFile('scripts/web/fixtures/native-character-loader-original.json','utf8'));const originalCount=fixtures.length;
const eh='geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11\n',hh='geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n',validEconomic='44001000200,1000,10,20,30,100,200,100,50,50\n';
for(const [kind,name,text]of [['economic','matched-negative',eh+'44001000100,-1,-2,-3,-4,-5,-6,-7,-8,-9\n'+validEconomic],['economic','matched-nan',eh+'44001000100,NaN,10,20,30,100,200,100,50,50\n'+validEconomic],['economic','matched-infinity',eh+'44001000100,inf,10,20,30,inf,200,100,50,50\n'+validEconomic],['economic','matched-large-raw',eh+'44001000100,1e30,10,20,30,100,200,100,50,50\n'+validEconomic],['housing','matched-nan',hh+'44001000100,NaN,.1,.6,.2\n'],['housing','matched-infinity',hh+'44001000100,inf,-inf,.6,.2\n'],['housing','matched-negative-zero',hh+'44001000100,-0,-0,.6,.2\n'],['housing','matched-zero',hh+'44001000100,0,0,0,0\n']]){
 const cwd=path.resolve('target/native-character-weight-numeric/'+kind+'-'+name),folder=cwd+'/data/2020/'+(kind==='economic'?'lodes':'acs_housing');await fs.mkdir(folder,{recursive:true});await fs.writeFile(folder+'/'+(kind==='economic'?'fixture_wac_tract.csv':'fixture_housing_2020.csv'),text);
 const source={name:'fixture.csv',base64:Buffer.from(text).toString('base64')},settings={state:'RI',year:'2020',data_year:'2020',kind,source_label:'SYNTHETIC native numeric boundary'},report=engine.execute({operation:'prepare-native-character-source',input:{...settings,source_name:source.name,source_base64:source.base64}});assert.equal(report.native_status,'loaded');fixtures.push({kind,name,cwd,expected:{native_status:'loaded'},project:{files:{nativeCharacterInputs:{source,settings}}}});
}
let cases=0,nativeComparisons=0,hostComparisons=0,refusals=0,nonfiniteEdges=0,loadedWithoutMatches=0;
const target={state:'RI',year:'2020',geoids:['44001000100','44001000200','44001999999'],edges:[[0,1,100],[0,2,75],[1,2,25]]};
let sample;
for(const fixture of fixtures)for(const alpha of [0,.25,.5,.75,1]){
 const original=fixture.project??JSON.parse(await fs.readFile('target/native-character-source/'+fixture.kind+'-'+fixture.name+'.bisect','utf8')),inputs={target,alpha,allow_absent_fallback:true},request=nativeCharacterWeightRequest(original.files.nativeCharacterInputs,inputs),result=engine.execute(request);
 await fs.writeFile(root+'/request.json',JSON.stringify(request));const host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));assert.deepEqual(result,host);hostComparisons++;
 const native=JSON.parse(execFileSync(path.resolve('target/release/examples/native_character_weights_reference.exe'),[path.resolve(root+'/request.json')],{cwd:fixture.cwd??path.resolve('target/native-character-loader/'+fixture.kind+'-'+fixture.name),encoding:'utf8'}));
 if(native.native_error)native.native_error=native.native_error.replace(/data[\\/]2020[\\/](?:lodes|acs_housing)[\\/]fixture_(?:wac_tract|housing_2020)\.csv/g,'fixture.csv');
 for(const [key,value]of Object.entries(native))assert.deepEqual(result[key],value,fixture.kind+'/'+fixture.name+'/'+alpha+'/'+key);nativeComparisons++;
 assert.equal(result.native_status,fixture.expected.native_status);assert.equal(result.execution_performed,false);assert.match(result.target_sha256,/^[a-f0-9]{64}$/);nonfiniteEdges+=result.edges.filter(e=>['nan','positive-infinity','negative-infinity'].includes(e.class)).length;
 if(fixture.kind==='housing'&&fixture.name==='matched-nan'&&alpha===1)assert(result.edges.some(e=>e.class==='nan'&&e.weight===null),'Native alpha one still propagates a matched NaN through floating-point multiplication');
 if(result.native_status==='loaded'&&!result.matched_tracts)loadedWithoutMatches++;
 const project=createProject({name:fixture.kind+' '+fixture.name+' native weighting',files:{...original.files,nativeCharacterWeightInputs:inputs},operation:request.operation,constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 if(alpha===.5)await fs.writeFile(root+'/'+fixture.kind+'-'+fixture.name+'.bisect',JSON.stringify(project));
 const disabled=structuredClone(request);disabled.input.allow_absent_fallback=false;
 if(result.native_status==='loaded'){assert.deepEqual(engine.execute(disabled),result);}else{assert.throws(()=>engine.execute(disabled),/explicit geographic fallback permission/);refusals++;}
 sample??=request;cases++;
}
for(const mutate of [r=>r.input.target.state='IA',r=>r.input.target.year='2010',r=>r.input.target.geoids[0]='"44001000100"',r=>r.input.target.geoids[1]=r.input.target.geoids[0],r=>r.input.target.edges.push(r.input.target.edges[0]),r=>r.input.target.edges[0][2]=-1,r=>r.input.target.edges[0][1]=3,r=>r.input.target.edges[0][0]=1,r=>r.input.alpha=-.1,r=>r.input.alpha=1.1,r=>r.input.alpha=-0,r=>r.input.source.source_base64='invalid!',r=>r.input.target.unknown=true]){const bad=structuredClone(sample);mutate(bad);assert.throws(()=>engine.execute(bad));refusals++;}
for(const mutate of [i=>i.target.state='IA',i=>i.target.geoids[0]='__proto__',i=>i.target.edges[0][2]=0,i=>i.alpha=-0,i=>i.allow_absent_fallback=1]){const bad=structuredClone(sample.input);delete bad.source;mutate(bad);assert.throws(()=>validateNativeCharacterWeightInputs(bad));refusals++;}
const nan=JSON.parse(await fs.readFile(root+'/housing-matched-nan.bisect','utf8')).result;assert(nan.edges.some(e=>e.class==='nan'&&e.weight===null));assert(nan.records.some(r=>r.classes.includes('nan')));assert(nonfiniteEdges>0);assert(loadedWithoutMatches>0);
await fs.writeFile(root+'/evidence.json',JSON.stringify({fixture_sources:fixtures.length,original_sources:originalCount,additional_matched_numeric_sources:8,alpha_values:5,cases,native_filesystem_alignment_weighter_comparisons:nativeComparisons,native_host_comparisons:hostComparisons,refusals,nonfinite_edge_outcomes:nonfiniteEdges,loaded_sources_without_target_matches:loadedWithoutMatches,project_source_target_roundtrips:true,numeric_classes_preserved:true,scope:'Actual native filesystem loaders, native alignment and CLI weighters versus WASM evaluation; no partition execution or full engine dispatch claim.',wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Native character weighting: ${cases} original-source/alpha cases, ${nativeComparisons} filesystem/native-alignment/weighter matches, ${hostComparisons} full Native host matches, ${refusals} refusals and explicit nonfinite classes passed.`);
