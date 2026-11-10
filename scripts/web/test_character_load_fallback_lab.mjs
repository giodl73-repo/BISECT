import {nativeCharacterFallbackInfo} from '../../web/lab/character-fallback-source.js';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {parseProject} from '../../web/lab/project.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {alignedCharacterFixture} from './character-alignment-fixture.mjs';
const fixture='target/lab-character-alignment',out='target/character-load-fallback-lab';await fs.mkdir(out,{recursive:true});
const manifest=JSON.parse(await fs.readFile(fixture+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),base=JSON.parse(await fs.readFile(fixture+'/browser.bisect','utf8')).experiment.config;
manifest.catalog.searches.push('multi');manifest.catalog.search_compatibility['standard-bisect'].push('multi');
const fetcher=async ref=>{const b=await fs.readFile(fixture+'/'+ref);return {ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
let projects=0,matches=0,rejections=0;
async function check(request,result){await fs.writeFile(out+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/'+(request.operation?'execute_tool_request':'execute_request')+'.exe',[out+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));assert.deepEqual(result,native);matches++;}
class Worker{postMessage(m){setImmediate(async()=>{try{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{const r=engine.execute(m.request);await check(m.request,r);this.onmessage({data:{type:'result',id:m.id,result:r}});}}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
for(const kind of ['economic','housing'])for(const search of ['single','multi'])for(const national of [false,true])for(const native_build of [false,true]){
 const states=national?['AL','AK','RI']:['RI'],c={...base,weights:kind+'-character',search,mode:national?'national':'state',states,districts:national?null:3,native_build,characters:{}};delete c.multiscale;if(!native_build)delete c.native_build;
 for(const code of states){const graph=JSON.parse(await fs.readFile(fixture+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8'));c.characters[code]=alignedCharacterFixture(engine,graph,kind);}
 const prep=JSON.parse(await fs.readFile('target/character-load-fallback/'+kind+'-'+(kind==='economic'?'missing-column':'empty')+'.bisect','utf8'));nativeCharacterFallbackInfo(prep);delete c.characters.RI;c.character_absent_file_fallback=true;c.character_fallback_sources={RI:prep};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(c);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const project=await createLabProject(lab,job.id),open=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await open.restoreProject(project);
 for(const code of states){const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.character_input,c.characters[code]);if(code==='RI'){assert.deepEqual(exported.project.files.characterFallbackSource,prep);assert.equal(exported.project.files.characterPreparation,undefined);const bad=structuredClone(exported.project);bad.files.plan.metadata.jurisdiction='IA';assert.throws(()=>parseProject(JSON.stringify(bad)));rejections++;}}
 assert.deepEqual(project.experiment.config.character_fallback_sources.RI,prep);for(const mutate of [p=>delete p.experiment.config.character_absent_file_fallback,p=>p.experiment.config.character_fallback_sources.RI.result.source_sha256='0'.repeat(64),p=>p.experiment.config.character_fallback_sources.RI.result.state='IA',p=>p.experiment.config.character_fallback_sources.RI.files.nativeCharacterInputs.source.base64+='Cg==']){const bad=structuredClone(project);mutate(bad);await assert.rejects(open.restoreProject(bad));rejections++;}
 projects++;lab.destroy(new Error('Done'));open.destroy(new Error('Done'));
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({projects,native_comparisons:matches,inert_open:true,retained_empty_error_sources_in_exports:true,mixed_national_observations:true,tamper_rejections:rejections,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Retained native load fallback: ${projects} state/national single/multi projects, ${matches} Native host comparisons, ${rejections} policy/source refusals, mixed observations, companion exports and inert Open passed.`);
