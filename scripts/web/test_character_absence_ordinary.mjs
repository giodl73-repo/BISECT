import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {alignedCharacterFixture} from './character-alignment-fixture.mjs';
const fixture='target/lab-character-alignment',out='target/character-absence-ordinary';await fs.mkdir(out,{recursive:true});
const manifest=JSON.parse(await fs.readFile(fixture+'/catalog.json','utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),base=JSON.parse(await fs.readFile(fixture+'/browser.bisect','utf8')).experiment.config;
manifest.catalog.searches.push('multi');manifest.catalog.search_compatibility['standard-bisect'].push('multi');
const fetcher=async ref=>{const b=await fs.readFile(fixture+'/'+ref);return {ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
let projects=0,matches=0;
async function check(request,result){await fs.writeFile(out+'/request.json',JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/'+(request.operation?'execute_tool_request':'execute_request')+'.exe',[out+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));assert.deepEqual(result,native);matches++;}
class Worker{postMessage(m){setImmediate(async()=>{try{if(m.type==='initialize')this.onmessage({data:{type:'ready'}});else{const r=engine.execute(m.request);await check(m.request,r);this.onmessage({data:{type:'result',id:m.id,result:r}});}}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){}}
for(const kind of ['economic','housing'])for(const search of ['single','multi'])for(const national of [false,true])for(const native_build of [false,true]){
 const states=national?['AL','AK','RI']:['RI'],c={...base,weights:kind+'-character',search,mode:national?'national':'state',states,districts:national?null:3,native_build,character_absent_file_fallback:true,characters:{}};delete c.multiscale;if(!native_build)delete c.native_build;
 for(const code of states.filter(code=>national&&code==='AL')){const graph=JSON.parse(await fs.readFile(fixture+'/'+manifest.graphs[code+':2020'].graph_ref,'utf8'));c.characters[code]=alignedCharacterFixture(engine,graph,kind);}
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(c);while(lab.active)await new Promise(r=>setTimeout(r,5));assert.equal(lab.jobs.get(job.id).status,'completed',lab.jobs.get(job.id).logs.join('\n'));
 const project=await createLabProject(lab,job.id),open=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not execute');});await open.restoreProject(project);
 for(const code of states){const exported=await lab.exportPractitionerProject(job.id,code);assert.deepEqual(exported.exported.document.provenance.producer.character_input,c.characters[code]);}
 projects++;lab.destroy(new Error('Done'));open.destroy(new Error('Done'));
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({projects,native_comparisons:matches,inert_open:true,original_sparse_sources_in_exports:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Ordinary character absence/mixed-source laboratory: ${projects} single/multi state/national projects with Native Build on/off, ${matches} full Native host comparisons, original sparse observations in exports and inert Open passed.`);
