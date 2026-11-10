import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {alignCharacterInput,validateCharacterInput,characterWeights} from '../../web/lab/character-input.js';
import {sparseCharacterFixture} from './character-alignment-fixture.mjs';
const root='target/character-alignment';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input.request.graph,catalog=JSON.parse(await fs.readFile('dist/wasm-lab-build/catalog.json','utf8')),graphs=[base];for(const code of ['RI','IA'])graphs.push(JSON.parse(await fs.readFile('dist/wasm-lab-build/'+catalog.graphs[code+':2020'].graph_ref,'utf8')));
let alignments=0,weighted=0,refusals=0;
async function native(request){await fs.writeFile(root+'/request.json',JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));}
for(const g of graphs)for(const kind of ['economic','housing'])for(const profile of ['partial','unmatched','full']){
 const source=sparseCharacterFixture(engine,g,kind,profile),request={operation:'align-character-input',input:source,geoids:g.geoids},aligned=engine.execute(request);assert.deepEqual(aligned,alignCharacterInput(source,g));assert.deepEqual(aligned,await native(request));validateCharacterInput(aligned,g,kind);alignments++;
 assert.deepEqual(aligned.alignment.source_data,source.data);assert.equal(aligned.alignment.ignored_tract_ids.length,1);
 for(const alpha of [0,.5,1]){
  const oracle={input:source,geoids:g.geoids,edges:g.edges,alpha};await fs.writeFile(root+'/oracle.json',JSON.stringify(oracle));const expected=JSON.parse(execFileSync('target/release/examples/character_alignment_reference.exe',[root+'/oracle.json'],{encoding:'utf8',maxBuffer:64*1024*1024}));
  assert.deepEqual(aligned.data.characters,expected.characters);const built=engine.execute({operation:'build-character-weights',input:aligned,geoids:g.geoids,edges:g.edges,alpha});assert.deepEqual(built.edges,expected.edges);
  const js=characterWeights(g,{characters:{[g.state]:aligned},weights:kind+'-character',character_alpha:alpha});for(const [u,v,w]of expected.edges)assert(Math.abs(js.weight(u,v,g.edges.find(e=>e[0]===u&&e[1]===v)[2])-w)<=Math.max(1,Math.abs(w))*1e-12);weighted++;
 }
 for(const mutate of [x=>x.data.characters[g.geoids[0]][kind==='housing'?'pct_owner':'commercial_intensity']=.99,x=>x.alignment.matched_tracts++,x=>x.alignment.policy='automatic',x=>x.alignment.ignored_tract_ids=[],x=>x.alignment.missing_tract_ids.push(g.geoids[0]),x=>x.alignment.source_data.characters[Object.keys(source.data.characters)[0]][kind==='housing'?'pct_owner':'jobs_per_resident']=-1]){
  const bad=structuredClone(aligned);mutate(bad);assert.throws(()=>validateCharacterInput(bad,g));const r={operation:'build-character-weights',input:bad,geoids:g.geoids,edges:g.edges,alpha:.5};assert.throws(()=>engine.execute(r));await assert.rejects(native(r));refusals++;
 }
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({alignments,native_alignment_weighter_cases:weighted,tamper_refusals:refusals,retained_original_observations:true,wasm_sha256:engine.artifactIdentity.sha256,scope:'Native align_lodes_to_adjacency/align_housing_to_adjacency and weighters on synthetic supplied observations over synthetic and actual RI/IA parent tract graphs; source acquisition/discovery/absence fallback not claimed.'},null,2));
console.log(`Character alignment: ${alignments} JS/Native-host/WASM alignments, ${weighted} independent native alignment/weighter cases and ${refusals} receipt/source refusals passed.`);
