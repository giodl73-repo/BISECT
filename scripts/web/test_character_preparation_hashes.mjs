import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {preparedCharacterInput,verifyCharacterPreparation} from '../../web/lab/character-handoff.js';
import {alignCharacterInput} from '../../web/lab/character-input.js';
const out='target/character-preparation-hashes';await fs.mkdir(out,{recursive:true});const manifest=JSON.parse(await fs.readFile('target/lab-character-alignment/catalog.json','utf8')),g=JSON.parse(await fs.readFile('target/lab-character-alignment/'+manifest.graphs['RI:2020'].graph_ref,'utf8'));let checked=0,refusals=0;
for(const prefix of ['lodes','acs-housing']){
 const p=JSON.parse(await fs.readFile('target/character-handoff/'+prefix+'.bisect','utf8')),input=preparedCharacterInput(p);await verifyCharacterPreparation(p,input);await verifyCharacterPreparation(p,alignCharacterInput(input,g));checked+=2;
 for(const mutate of [x=>{const key=prefix==='lodes'?'lodesInputs':'acsHousingInputs',src=x.files[key].source;src.base64=Buffer.from(Buffer.from(src.base64,'base64').toString()+'\n').toString('base64');},x=>x.result.csv_text+='\n',x=>x.result.source_name='changed.txt',x=>x.result.source_sha256='0'.repeat(64),x=>x.result.csv_sha256='0'.repeat(64),x=>x.result.character_input.source_label='changed']){const bad=structuredClone(p);mutate(bad);await assert.rejects(verifyCharacterPreparation(bad,input));refusals++;}
 const nested=structuredClone(p);nested.files={characterPreparation:p};assert.throws(()=>preparedCharacterInput(nested));refusals++;
}
let accessed=false;assert.throws(()=>preparedCharacterInput({get operation(){accessed=true;return 'prepare-lodes-source';}}));assert.equal(accessed,false);
await fs.writeFile(out+'/evidence.json',JSON.stringify({source_and_aligned_checks:checked,source_hash_scope_refusals:refusals,accessor_refused_without_invocation:true},null,2));console.log(`Preparation source retention: ${checked} source/aligned checks, ${refusals} raw-source/CSV/hash/scope/nesting refusals and non-invoked accessor refusal passed.`);
