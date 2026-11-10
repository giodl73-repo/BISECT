import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject,parseSafeJson} from '../../web/lab/project.js';
import {nativeCharacterWeightRequest,validateNativeCharacterWeightInputs,readNativeCharacterWeightTarget} from '../../web/lab/native-character-weight-files.js';
import {nativeCharacterEngineRequest,NATIVE_CHARACTER_ENGINE_DEFAULTS} from '../../web/lab/native-character-engine-files.js';
import {nativeCharacterLabWeights,verifyNativeCharacterLabSource} from '../../web/lab/native-character-lab.js';
import {verifyNativeCharacterProducerSource} from '../../web/lab/native-character-producer.js';

globalThis.Worker=class{constructor(url){assert(url.pathname.endsWith('/json-worker.js'));}postMessage(file){setImmediate(async()=>{if(this.done)return;try{this.onmessage({data:{ok:true,project:parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(await file.arrayBuffer()))}});}catch(e){this.onmessage({data:{ok:false,error:e.message}});}});}terminate(){this.done=true;}};
const root='target/native-character-units';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const catalog=JSON.parse(await fs.readFile('target/lab-character-alignment/catalog.json','utf8'));
const original=JSON.parse(await fs.readFile('target/lab-character-alignment/'+catalog.graphs['RI:2020'].graph_ref,'utf8'));
const options={structure:'standard-bisect',weights:'economic-character',search:'single',districts:3,seed:'18446744073709551615',seeds:3,steps:5,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,character_alpha:.5};
const kinds=['county','tract','block-group','block','precinct','imported'];
let evaluations=0,hostMatches=0,filesystemWeights=0,stateMatches=0,reviews=0,exports=0,builds=0,refusals=0,roundtrips=0;
async function host(request){await fs.writeFile(root+'/request.json',JSON.stringify(request));return JSON.parse(execFileSync(path.resolve('target/release/examples/'+(request.operation?'execute_tool_request':'execute_request')+'.exe'),[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:64*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function parity(request){const result=engine.execute(request);assert.deepEqual(result,await host(request));hostMatches++;return result;}
for(const unit_kind of kinds)for(const year of ['precinct','imported'].includes(unit_kind)?['2018','2024']:['2000','2010','2020'])for(const kind of ['economic','housing'])for(const profile of ['partial','tract-only','nan']){
 const custom=['precinct','imported'].includes(unit_kind),width={county:5,tract:11,'block-group':12,block:15}[unit_kind];
 const geoids=original.geoids.map((id,i)=>custom?(i===0?'é/area':i===1?'𝒜':i===2?'\uE000':'unit-'+i):'44'+String(i+1).padStart(width-2,'0'));
 const graph={...original,year,unit_kind,geoids,...(custom?{county_ids:geoids.map((_,i)=>i<18?'County α':'County β')}:{})};
 const header=kind==='economic'?'geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11\n':'geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n';
 const ids=profile==='tract-only'?[original.geoids[0]]:geoids.filter((_,i)=>profile==='nan'||i%2===0);
 const text=header+ids.map((id,i)=>kind==='economic'?id+','+(profile==='nan'?'NaN':'1000')+',10,20,30,100,200,100,50,50':id+','+(profile==='nan'?'NaN':i%2?'.2':'.8')+',.1,.6,.2').join('\n')+'\n';
 const source={source:{name:'fixture.csv',base64:Buffer.from(text).toString('base64')},settings:{state:'RI',year,data_year:'2019',kind,source_label:'SYNTHETIC exact-ID '+unit_kind+' '+profile}};
 const target={state:'RI',year,unit_kind,...(custom?{county_ids:graph.county_ids}:{}),geoids,edges:graph.edges};
 const loaded=await readNativeCharacterWeightTarget({name:'graph.json',size:Buffer.byteLength(JSON.stringify(graph)),arrayBuffer:async()=>new TextEncoder().encode(JSON.stringify(graph)).buffer});assert.deepEqual(loaded,target);
 const cwd=path.resolve(root+'/native-files/'+unit_kind+'-'+year+'-'+kind+'-'+profile),folder=cwd+'/data/2020/'+(kind==='economic'?'lodes':'acs_housing');await fs.mkdir(folder,{recursive:true});await fs.writeFile(folder+'/'+(kind==='economic'?'fixture_wac_tract.csv':'fixture_housing_2020.csv'),text);
 for(const alpha of [0,.5,1]){
  const inputs={target,alpha,allow_absent_fallback:false},request=nativeCharacterWeightRequest(source,inputs),evaluation=await parity(request);evaluations++;
  const oracle=JSON.parse(execFileSync(path.resolve('target/release/examples/native_character_weights_reference.exe'),[path.resolve(root+'/request.json')],{cwd,encoding:'utf8'}));for(const [key,value]of Object.entries(oracle))assert.deepEqual(evaluation[key],value);filesystemWeights++;
  assert.equal(evaluation.native_status,'loaded');assert.equal(evaluation.geographic_fallback,false);assert.deepEqual(JSON.parse(evaluation.target_json),target);assert.equal(evaluation.target_sha256,createHash('sha256').update(evaluation.target_json).digest('hex'));
  const expectedMatched=ids.filter(id=>geoids.includes(id)).length;assert.equal(evaluation.matched_tracts,expectedMatched);assert.equal(evaluation.missing_tract_ids.length,geoids.length-expectedMatched);
  if(unit_kind==='tract'&&year==='2020'&&kind==='economic'&&profile==='partial'&&alpha===.5){
   const legacy=structuredClone(evaluation),legacyTarget=structuredClone(target);delete legacyTarget.unit_kind;legacy.target_json=JSON.stringify(legacyTarget);legacy.target_sha256=createHash('sha256').update(legacy.target_json).digest('hex');
   const config={year,weights:kind+'-character',character_alpha:alpha,search:'single',native_character_sources:{RI:source}},evidence={method:'native-character-file-v1',evaluation:legacy,scope:'edge-weighting'};
   await verifyNativeCharacterLabSource(graph,{metrics:{weighting_evidence:evidence}},config);
   assert.throws(()=>nativeCharacterLabWeights({...graph,unit_kind:'precinct',county_ids:geoids.map(()=> 'County A')},config,evidence));
  }
  if(alpha!==.5)continue;
  for(const search of ['single','multi']){
   const direct={graph,options:{...options,weights:kind+'-character',search},native_character_source:request.input.source},result=await parity(direct);
   const reference=JSON.parse(execFileSync(path.resolve('target/release/examples/native_character_source_run_reference.exe'),[path.resolve(root+'/request.json')],{cwd,encoding:'utf8',stdio:['ignore','pipe','pipe']}));assert.deepEqual(result.assignments,reference.assignments);assert.equal(result.metrics.weighted_boundary,reference.weighted_boundary);assert.equal(result.metrics.weighted_boundary_class,reference.weighted_boundary_class);stateMatches++;
  }
  const rawRequest={graph,options:{...options,weights:kind+'-character'}},files={nativeCharacterInputs:source,nativeCharacterWeightInputs:inputs,nativeCharacterEngineInputs:{source:{name:'request.json',text:JSON.stringify(rawRequest)},settings:{...NATIVE_CHARACTER_ENGINE_DEFAULTS,label:'Synthetic '+unit_kind,chamber:'congressional',created_at:'2026-10-17T00:00:00Z',inspect_only:true}}};
  if(profile==='partial'){
   const preview=await parity(nativeCharacterEngineRequest(files.nativeCharacterInputs,files.nativeCharacterEngineInputs));assert.equal(preview.csv_loading_performed,false);reviews++;
   files.nativeCharacterEngineInputs.settings.inspect_only=false;
   const bundle=await parity(nativeCharacterEngineRequest(files.nativeCharacterInputs,files.nativeCharacterEngineInputs));assert.equal(bundle.export_error,null);assert.equal(bundle.export.document.plan.units.unit_kind,unit_kind);assert.equal(String(bundle.export.document.plan.units.year),year);
   assert.deepEqual((await verifyNativeCharacterProducerSource(bundle.export.document,bundle.export.context)).input,source);exports++;
   if(year===(custom?'2024':'2020')){
    const buildRequest=nativeCharacterEngineRequest(source,{...files.nativeCharacterEngineInputs,source:{name:'build.json',text:JSON.stringify({operation:'run-engine-build',request:rawRequest})}}),built=await parity(buildRequest);
    assert.equal(built.export_error,null);assert.equal(built.export.document.plan.units.unit_kind,unit_kind);assert.equal(built.executed_request.graph.unit_kind,unit_kind);assert.equal(built.export.document.provenance.producer.engine_options.seed,built.executed_request.options.seed);builds++;
   }
   const project=createProject({name:'SYNTHETIC '+unit_kind+' '+year+' '+kind,files,operation:'run-native-character-engine',constraints:[],result:bundle,lastOperation:'run-native-character-engine'});assert.deepEqual(parseProject(JSON.stringify(project)),project);roundtrips++;await fs.writeFile(root+'/'+unit_kind+'-'+year+'-'+kind+'.bisect',JSON.stringify(project));
  }
 }
}
// The source namespace is explicit; neither unsafe IDs nor implicit county or
// parent matching is permitted. Every Rust refusal must agree with Native host.
const sample=JSON.parse(await fs.readFile(root+'/precinct-2024-housing.bisect','utf8'));
const base=nativeCharacterWeightRequest(sample.files.nativeCharacterInputs,sample.files.nativeCharacterWeightInputs);
for(const mutate of [r=>delete r.input.target.county_ids,r=>r.input.target.county_ids.pop(),r=>r.input.target.geoids[0]='__proto__',r=>r.input.target.geoids[0]='control\n',r=>r.input.target.geoids[0]='é'.repeat(101),r=>r.input.target.geoids[1]=r.input.target.geoids[0],r=>r.input.target.unit_kind='tract',r=>r.input.target.unit_kind='unknown',r=>r.input.target.year='2020',r=>r.input.source.year='20x4',r=>r.input.source.source_label='',r=>r.input.target.edges[0][2]=0,r=>r.input.target.unknown=true]){
 const request=structuredClone(base);mutate(request);assert.throws(()=>engine.execute(request));await assert.rejects(host(request));refusals++;
 const inputs={target:request.input.target,alpha:request.input.alpha,allow_absent_fallback:request.input.allow_absent_fallback};if(!['2020'].includes(inputs.target.year)&&request.input.source.year==='2024'&&request.input.source.source_label)assert.throws(()=>validateNativeCharacterWeightInputs(inputs));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({unit_kinds:kinds,canonical_census_years:['2000','2010','2020'],custom_unit_years:['2018','2024'],synthetic_fixtures:true,weight_evaluations:evaluations,full_native_host_matches:hostMatches,native_filesystem_alignment_weighter_matches:filesystemWeights,native_filesystem_state_kernel_matches:stateMatches,nonexecuting_reviews:reviews,physical_exports:exports,native_build_exports:builds,source_project_roundtrips:roundtrips,refusals,legacy_declared_tract_target_opens_with_original_hash:true,legacy_target_cannot_change_namespace:true,exact_id_alignment_no_parent_inference:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Native CSV unit namespaces: ${evaluations} weight evaluations, ${hostMatches} full Native matches, ${filesystemWeights} filesystem weighter and ${stateMatches} filesystem/State kernel matches, ${exports} physical exports and ${refusals} refusals passed.`);
