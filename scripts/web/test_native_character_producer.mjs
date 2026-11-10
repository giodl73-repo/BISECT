import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {nativeCharacterProducerSource,verifyNativeCharacterProducerSource} from '../../web/lab/native-character-producer.js';

let refusals=0;
for(const folder of ['lab-native-character-browser','lab-native-character-multiscale-browser']){
 const project=JSON.parse(await fs.readFile('target/'+folder+'/audit.bisect','utf8')),{plan,context}=project.files;
 const verified=await verifyNativeCharacterProducerSource(plan,context);
 assert.deepEqual(verified.input,nativeCharacterProducerSource(plan));
 assert.equal(Buffer.from(verified.bytes).toString('base64'),verified.input.source.base64);
 for(const mutate of [
  p=>p.provenance.source_hashes['bisect.native-character-source']='0'.repeat(64),
  p=>{const r=p.provenance.producer.native_character_source??p.provenance.producer.multiscale_input.request.native_character_source;r.source_base64=Buffer.from('changed').toString('base64');},
  p=>p.metadata.jurisdiction='AK',
  p=>p.plan.units.year=2010,
  p=>{const r=p.provenance.producer.native_character_source??p.provenance.producer.multiscale_input.request.native_character_source;r.unknown=true;},
 ]){const bad=structuredClone(plan);mutate(bad);await assert.rejects(verifyNativeCharacterProducerSource(bad,context));refusals++;}
 const badContext=structuredClone(context);badContext.source_hashes['bisect.native-character-source']='0'.repeat(64);await assert.rejects(verifyNativeCharacterProducerSource(plan,badContext));refusals++;
}
assert.equal(nativeCharacterProducerSource({}),null);
await fs.mkdir('target/native-character-producer',{recursive:true});
await fs.writeFile('target/native-character-producer/evidence.json',JSON.stringify({ordinary_and_multiscale_sources_verified:2,source_scope_and_hash_refusals:refusals,no_parser_or_engine_invoked:true},null,2));
console.log(`Original producer CSV: two physical exports, ${refusals} source/scope/hash refusals, exact bytes without CSV parsing or engine execution passed.`);
