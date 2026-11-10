import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {manifestReplayRequest,engineReplayReceiptBytes} from '../../web/lab/engine-replay-files.js';
const root='target/replay-aux-bindings-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/optional-seed-wasm/case-0.json','utf8')),graph=base.graph;
const counts=Object.fromEntries(graph.geoids.map((id,i)=>[id,{total:80,minority:i%2?60:20}]));
const demographics={schema_version:'bisect-demographic-counts-v2',state:graph.state,year:graph.year,basis:'total-population',source_label:'SYNTHETIC counts for replay binding',minority_fractions:Object.fromEntries(graph.geoids.map(id=>[id,counts[id].minority/counts[id].total])),counts};
const partisan={schema_version:'bisect-partisan-shares-v1',state:graph.state,year:graph.year,source_label:'SYNTHETIC shares',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%2?.8:.2]))};
const elections={schema_version:'bisect-election-counts-v1',state:graph.state,year:graph.year,election_year:'2022',source_label:'SYNTHETIC election counts',counts:Object.fromEntries(graph.geoids.map(id=>[id,{democratic:20,two_party:40}]))};
const csv='geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage\n'+graph.geoids.map((id,i)=>id+','+(i%2?'.8,.1,.6,.2':'.1,.7,.2,.8')).join('\n')+'\n';
const character=engine.execute({operation:'import-character-csv',state:graph.state,year:graph.year,data_year:'2022',kind:'housing',source_label:'SYNTHETIC housing values',source_base64:Buffer.from(csv).toString('base64')});
const options={...base.options,seed:'18446744073709551615',balance_tolerance:50};
const cases={demographics:{graph,options:{...options,weights:'vra-aligned'},demographics},character:{graph,options:{...options,weights:'housing-character',character_alpha:.5},character},partisan:{graph,options:{...options,weights:'partisan',dem_threshold:.55,rep_threshold:.45},partisan},elections:{graph,options:{...options,structure:'proportional-bisect'},elections},combined:{graph,options:{...options,structure:'ratio-optimal-vra',weights:'housing-character',character_alpha:.5,w_vra:.4},character,demographics}};
for(const [name,basis]of [['vap','voting-age-population'],['cvap','citizen-voting-age-population']])cases[name]={graph,options:{...options,weights:'geographic',search:'vra-recom',balance_tolerance:.5,vra_threshold:.5},demographics:{...demographics,basis}};
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const change=(source,fn)=>{const v=JSON.parse(Buffer.from(source.base64,'base64').toString());fn(v);source.base64=Buffer.from(JSON.stringify(v)).toString('base64');};
let parity=0;const evidence=[];
async function check(name,files,status='match',aux='verified'){
 const request=manifestReplayRequest(files.packageVerificationInputs,files.engineReplayInputs),result=engine.execute(request),path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024})),result);parity++;
 assert.equal(result.status,status,result.error);if(status==='binding-failed'){assert.equal(result.execution_performed,false);assert.equal(result.replay,null);}else{assert.equal(result.binding.auxiliary_inputs,aux);assert.equal(result.manifest_to_run_binding,aux==='verified'?'verified':'partial');assert.equal(result.replay.agreement.similarity,1);}
 const project=createProject({name:'Auxiliary replay '+name,files,operation:'verify-manifest-replay',constraints:[],result,lastOperation:'verify-manifest-replay'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(JSON.parse(new TextDecoder().decode(engineReplayReceiptBytes(result))),result);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));evidence.push({name,status,aux:result.binding.auxiliary_inputs});return result;
}
for(const [name,request]of Object.entries(cases)){
 const generated=engine.execute(request),exported=engine.execute({operation:'export-engine-plan',request,assignments:generated.assignments,label:'aux-'+name,chamber:'congressional',created_at:'2026-10-08T00:00:00Z'}),manifest=engine.execute({operation:'prepare-plan-report-manifest',document_json:exported.document_json,context_json:exported.context_json});
 const files={packageVerificationInputs:{manifest:source('manifest.json',manifest.manifest_json),files:{'plan.rplan':source('plan.rplan',exported.document_json),'context.rctx':source('context.rctx',exported.context_json)}},engineReplayInputs:{run:source('run.json',JSON.stringify({kind:'engine',request})),original:source('original.json',JSON.stringify(generated.assignments)),minSimilarity:1}};
 const good=await check(name,files);for(const key of ['demographics','character','partisan','elections'])assert.equal(good.binding.auxiliary_bindings[key],request[key]?'verified':'not-applicable');
 const reencoded=structuredClone(files);change(reencoded.packageVerificationInputs.files['plan.rplan'],()=>{});await check(name+'-reencoded-provenance',reencoded);
 const changed=structuredClone(files);change(changed.engineReplayInputs.run,a=>{
  if(['demographics','vap','cvap'].includes(name)){const id=graph.geoids[0];a.request.demographics.counts[id].total*=2;a.request.demographics.counts[id].minority*=2;}
  else if(name==='partisan')a.request.partisan.dem_shares[graph.geoids[0]]=.3;
  else if(name==='elections')a.request.elections.counts[graph.geoids[0]].democratic++;
  else a.request.character.source_label='Changed source description';
 });await check(name+'-changed',changed,'binding-failed');
 const relabelled=structuredClone(files);change(relabelled.engineReplayInputs.run,a=>a.request[['combined','vap','cvap'].includes(name)?'demographics':name].source_label='Different recorded source');await check(name+'-relabelled',relabelled,'binding-failed');
 if(!['demographics','vap','cvap'].includes(name)){
  const missing=structuredClone(files),key=name==='combined'?'character_input':name==='elections'?'election_input':name+'_input';change(missing.packageVerificationInputs.files['plan.rplan'],p=>delete p.provenance.producer[key]);await check(name+'-missing-provenance',missing,'match','partial');
 }
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:parity,cases:evidence,synthetic_sources:true,authenticity_claim:false},null,2));console.log(`Auxiliary replay bindings: ${parity} native/WASM matches; four source types and combined inputs, exact u64, changed counts with identical fractions, source-description/provenance mismatch, missing evidence, project and receipts passed.`);
