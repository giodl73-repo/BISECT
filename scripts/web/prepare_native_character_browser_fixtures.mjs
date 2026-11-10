import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
import {NATIVE_CHARACTER_DEFAULTS,nativeCharacterRequest} from '../../web/lab/native-character-files.js';
const root='target/native-character-source';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),fixtures=JSON.parse(await fs.readFile('scripts/web/fixtures/native-character-loader-original.json','utf8'));
for(const f of fixtures){const inputs={source:{name:'fixture.csv',base64:Buffer.from(f.text).toString('base64')},settings:{...NATIVE_CHARACTER_DEFAULTS,kind:f.kind}},request=nativeCharacterRequest(inputs),result=engine.execute(request);assert.equal(result.native_status,f.expected.native_status);if(result.native_status==='error')assert.equal(result.native_error,f.expected.native_error);else assert.deepEqual(result.records,f.expected.records);await fs.writeFile(root+'/'+f.kind+'-'+f.name+'.bisect',JSON.stringify(createProject({name:f.kind+' '+f.name,files:{nativeCharacterInputs:inputs},operation:request.operation,constraints:[],result,lastOperation:request.operation})));}
console.log('Prepared browser fixtures against original native loader golden values.');
