import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {multiscaleCharacterFixture} from './multiscale-character-fixture.mjs';
const out='target/multiscale-character-refusals';await fs.mkdir(out,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input;
let refusals=0;
for(const kind of ['economic','housing']){
 const input=structuredClone(base);input.request.options.weights=kind+'-character';input.request.options.character_alpha=.5;
 input.request.character=multiscaleCharacterFixture(engine,input.request.graph,kind);
 const cases=[x=>delete x.request.character,x=>delete x.request.character.data.characters[x.request.graph.geoids[0]],x=>x.request.character.state='ZZ',x=>x.request.character.year='2010',x=>x.request.character.data.kind=kind==='economic'?'housing':'economic',x=>x.request.options.weights='geographic',x=>x.request.options.character_alpha=1.01];
 if(kind==='economic')cases.push(x=>x.request.character.data.raw_counts[x.request.graph.geoids[0]].c000++);
 for(const mutate of cases){const bad=structuredClone(input);mutate(bad);const request={operation:'run-multiscale',input:bad};
  assert.throws(()=>engine.execute(request));await fs.writeFile(out+'/request.json',JSON.stringify(request));
  assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[out+'/request.json'],{stdio:'pipe'}));refusals++;
 }
}
await fs.writeFile(out+'/evidence.json',JSON.stringify({refusals,native_and_wasm_refused:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));
console.log(`Character multiscale: ${refusals} missing, incomplete, foreign-scope, wrong-kind, unused, invalid-blend and inconsistent-raw-count inputs refused by Native host and WASM.`);
