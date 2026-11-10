import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeCharacterRequest} from '../../web/lab/native-character-files.js';
import {characterPreparationProject} from '../../web/lab/character-handoff.js';
const out='target/native-character-flow',catalog=JSON.parse(await fs.readFile('target/lab-character-alignment/catalog.json','utf8')),graph=JSON.parse(await fs.readFile('target/lab-character-alignment/'+catalog.graphs['RI:2020'].graph_ref,'utf8')),engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
for(const kind of ['economic','housing']){
 const original=JSON.parse(await fs.readFile(out+'/'+kind+'-'+(kind==='housing'?'aliases-precedence':'duplicate-row')+'.bisect','utf8')),inputs=structuredClone(original.files.nativeCharacterInputs),id=Object.keys(original.result.character_input.data.characters)[0];inputs.source.base64=Buffer.from(Buffer.from(inputs.source.base64,'base64').toString().replaceAll(id,graph.geoids[0])).toString('base64');inputs.settings.source_label='SYNTHETIC native CSV with one target-graph observation';const request=nativeCharacterRequest(inputs),result=engine.execute(request);await fs.writeFile(out+'/matched-request.json',JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[out+'/matched-request.json'],{encoding:'utf8',maxBuffer:64*1024*1024})));
 assert.equal(Object.keys(result.character_input.data.characters)[0],graph.geoids[0]);await fs.writeFile(out+'/'+kind+'-target-matched.bisect',JSON.stringify(characterPreparationProject(result,{nativeCharacterInputs:inputs})));
}
console.log('Native CSV flow fixtures: one supplied observation matches the target RI graph; remaining tracts require explicit native defaults.');
