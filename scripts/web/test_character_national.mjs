import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
const root=process.argv[2]||'dist/wasm-character',manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
const fetcher=async ref=>{const bytes=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
let executions=0,states=0;
class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{if(m.type!=='initialize')executions++;this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}
const codes=[...new Set(Object.keys(manifest.graphs).map(key=>key.split(':')[0]))].sort();
for(const kind of ['economic','housing']){
 const inputs=Object.create(null);let config;
 for(const code of codes){const saved=JSON.parse(await fs.readFile(`target/character-${code}-${kind}-single.bisect`,'utf8'));config=saved.experiment.config;inputs[code]=config.characters[code];await fs.writeFile(`target/${code}-SYNTHETIC-${kind}.json`,JSON.stringify(inputs[code]));}
 config={...config,name:`National ${kind} character test`,mode:'national',states:codes,districts:null,characters:inputs};
 const lab=new WasmCatalog(manifest,fetcher,()=>new Worker()),job=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));assert.equal(lab.jobs.get(job.id).status,'completed');
 const saved=await createLabProject(lab,job.id),before=executions,restore=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not generate.');}),opened=await restore.restoreProject(structuredClone(saved));assert.equal(executions,before);assert.deepEqual(opened.config.characters,JSON.parse(JSON.stringify(inputs)));
 for(const code of codes){const map=await restore.api(`/api/runs/${opened.id}/${code}/map`),assignment=saved.assignments[code];assert.equal(map.features.length,Object.keys(assignment).length);assert.ok(map.features.every(feature=>feature.properties.district===assignment[feature.properties.geoid]));states++;}
 const missing=structuredClone(saved);delete missing.experiment.config.characters[codes[0]];await assert.rejects(restore.restoreProject(missing));
 if(codes.length>1){const wrong=structuredClone(saved);wrong.experiment.config.characters[codes[0]]=wrong.experiment.config.characters[codes[1]];await assert.rejects(restore.restoreProject(wrong));}
 const changed=structuredClone(saved);changed.experiment.config.characters[codes[0]].data_year='2019';await assert.rejects(restore.restoreProject(changed));
 await fs.writeFile(`target/character-national-${kind}.bisect`,JSON.stringify(saved));
}
await fs.writeFile('target/character-national-verification.json',JSON.stringify({national_roundtrips:2,state_maps:states,engine_runs:executions,no_generation_on_open:true},null,2)+'\n');
console.log(`National character projects: two ${codes.length}-state experiments, ${states} saved/restored maps, complete input/scope/hash validation and no generation on Open passed.`);
