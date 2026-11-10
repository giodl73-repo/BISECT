import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {parseSafeJson,MAX_PROJECT_BYTES} from '../../web/lab/project.js';
import {characterPreparationProject} from '../../web/lab/character-handoff.js';
import {nativeCharacterFallbackProject} from '../../web/lab/character-fallback-source.js';
import {characterStateFips} from '../../web/lab/character-input.js';
import {characterSelection,readCharacterSelections,applyCharacterSelections,alignCharacterSelections} from '../../web/lab/character-selection.js';
const root='target/character-batch-selection';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const states='AL AK AZ AR CA CO CT DE FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY'.split(' ');
let jsonReads=0,preparations=0,refusals=0,alignedStates=0;
globalThis.Worker=class{constructor(url){assert(url.pathname.endsWith('/json-worker.js'));}postMessage(file){setImmediate(async()=>{if(this.done)return;try{const bytes=await file.arrayBuffer();jsonReads++;this.onmessage({data:{ok:true,project:parseSafeJson(new TextDecoder('utf-8',{fatal:true}).decode(bytes))}});}catch(e){this.onmessage({data:{ok:false,error:e.message}});}});}terminate(){this.done=true;}};
const file=(v,name='source.bisect')=>new File([JSON.stringify(v)],name,{type:'application/json'});
const selectionSets={};
for(const kind of ['economic','housing']){
 const observed=JSON.parse(await fs.readFile('target/native-character-flow/'+kind+'-target-matched.bisect','utf8')),fallback=JSON.parse(await fs.readFile('target/character-load-fallback/'+kind+'-'+(kind==='economic'?'missing-column':'empty')+'.bisect','utf8'));
 const files=[],graphs={};
 for(const [i,state]of states.entries()){
  const src=structuredClone(i%3===0?fallback:observed),inputs=src.files.nativeCharacterInputs;
  inputs.settings.state=state;inputs.settings.data_year='2019';inputs.source.base64=Buffer.from(Buffer.from(inputs.source.base64,'base64').toString().replaceAll('440',''+characterStateFips(state)+'0')).toString('base64');
  const report=engine.execute({operation:'prepare-native-character-source',input:{...inputs.settings,source_name:inputs.source.name,source_base64:inputs.source.base64}});
  const project=i%3===0?nativeCharacterFallbackProject(report,src.files):characterPreparationProject(report,src.files);files.push(file(project,state+'.bisect'));preparations++;
  graphs[state]={state,year:'2020',geoids:[characterStateFips(state)+'001000001',characterStateFips(state)+'001999999']};
 }
 const opts={kind,year:'2020',allowFallback:true};
 const selections=await readCharacterSelections(files,opts);assert.equal(Object.keys(selections).length,50);
 const merged=applyCharacterSelections({},selections);assert.equal(Object.keys(merged.characters).length,33);assert.equal(Object.keys(merged.preparations).length,33);assert.equal(Object.keys(merged.fallbacks).length,17);
 const before=structuredClone(merged),fetched=[];
 const result=await alignCharacterSelections(merged.characters,{...opts,states,graphFor:async code=>{assert(!merged.fallbacks[code]);fetched.push(code);return graphs[code];}});assert.equal(result.aligned,33);assert.equal(fetched.length,33);alignedStates+=result.aligned;assert.deepEqual(merged,before);
 for(const [code,input]of Object.entries(result.characters)){assert.equal(input.alignment.matched_tracts,1);assert.equal(input.alignment.missing_tract_ids.length,1);assert.deepEqual(input.alignment.source_data,merged.characters[code].data);}
 const fallbackOnly=await alignCharacterSelections({}, {...opts,states:['AL'],graphFor:()=>{throw new Error('Absent states must not fetch a graph for alignment');}});assert.equal(fallbackOnly.aligned,0);
 await assert.rejects(alignCharacterSelections({}, {...opts,allowFallback:false,states:['AL'],graphFor:()=>{throw new Error('No fetch expected');}}));refusals++;
 for(const options of [{...opts,allowFallback:false},{...opts,year:'2010'},{...opts,kind:kind==='economic'?'housing':'economic'}]){await assert.rejects(readCharacterSelections(files,options));refusals++;}
 await assert.rejects(readCharacterSelections([files[1],files[1]],opts));refusals++;
 await assert.rejects(readCharacterSelections([files[1],file({schema_version:'bisect-character-v1'})],opts));refusals++;
 const bad=structuredClone(merged.preparations.AK);bad.files.nativeCharacterInputs.source.base64=Buffer.from('changed').toString('base64');await assert.rejects(characterSelection(bad,opts));refusals++;
 const badFallback=structuredClone(merged.fallbacks.AL);badFallback.result.source_sha256='0'.repeat(64);await assert.rejects(characterSelection(badFallback,opts));refusals++;
 const replaced=applyCharacterSelections(merged,{AK:await characterSelection({...merged.fallbacks.AL,files:{nativeCharacterInputs:{...merged.fallbacks.AL.files.nativeCharacterInputs,settings:{...merged.fallbacks.AL.files.nativeCharacterInputs.settings,state:'AK'}}},result:{...merged.fallbacks.AL.result,state:'AK'}},opts)});
 assert.equal(replaced.characters.AK,undefined);assert.equal(replaced.preparations.AK,undefined);assert(replaced.fallbacks.AK);
 const plain=await characterSelection(merged.characters.AK,opts),restored=applyCharacterSelections(replaced,{AK:plain});assert(restored.characters.AK);assert.equal(restored.preparations.AK,undefined);assert.equal(restored.fallbacks.AK,undefined);assert.deepEqual(merged,before);
 selectionSets[kind]=merged;await fs.writeFile(root+'/'+kind+'-selections.json',JSON.stringify(merged));
}
for(const [prefix,kind]of [['lodes','economic'],['acs-housing','housing']]){const original=JSON.parse(await fs.readFile('target/character-handoff/'+prefix+'.bisect','utf8')),selections=await readCharacterSelections([file(original)],{kind,year:original.result.year});assert.deepEqual(selections[original.result.state].preparation,original);assert.deepEqual(selections[original.result.state].input,original.result.character_input);}
const reads=jsonReads;
for(const files of [[],Array(51).fill(file({})),[{size:MAX_PROJECT_BYTES+1,arrayBuffer:()=>{throw new Error('Must reject before read');}}],[{size:-1}]]){await assert.rejects(readCharacterSelections(files,{kind:'economic',year:'2020'}));refusals++;}assert.equal(reads,jsonReads);
await fs.writeFile(root+'/evidence.json',JSON.stringify({states:states.length,kinds:2,retained_preparations:preparations,aligned_observed_states:alignedStates,refusals,json_reads:jsonReads,lodes_acs_saved_sources:true,source_replacement_atomic:true,no_engine_execution_during_selection_or_alignment:true,original_sparse_sources_retained:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Character batch selection: ${preparations} original source projects across all 50 states and both kinds, ${alignedStates} observation alignments, ${refusals} refusals, explicit fallback, replacement and original-source preservation passed.`);
