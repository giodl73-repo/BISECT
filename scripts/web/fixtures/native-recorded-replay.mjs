import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import assert from 'node:assert/strict';

export const replayCorpusDirectory=path.join(path.dirname(fileURLToPath(import.meta.url)),'native-recorded-replay');
export const sha256=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function loadNativeReplayCorpus(){
 const manifest=JSON.parse(await fs.readFile(path.join(replayCorpusDirectory,'manifest.json'),'utf8'));
 assert.equal(manifest.schema_version,'bisect-native-replay-regression-corpus-v1');
 const cases=[];
 for(const ref of manifest.cases){
  assert(/^[a-z0-9-]+\.json$/.test(ref.file));
  const bytes=await fs.readFile(path.join(replayCorpusDirectory,ref.file));assert.equal(sha256(bytes),ref.sha256);
  const fixture=JSON.parse(bytes);assert.equal(fixture.schema_version,'bisect-native-replay-regression-fixture-v1');assert.equal(fixture.id,ref.id);assert.equal(fixture.reference_kind,ref.reference_kind);
  fixture.request=JSON.parse(fixture.request_json);
  if(fixture.reference_kind==='actual-native-completed-plan'){
   const reference=fixture.native_reference;assert.equal(sha256(Buffer.from(reference.rplan_json)),reference.rplan_sha256);
   const input=fixture.request.input.sources.prechecks;assert.equal(sha256(Buffer.from(input.manifest_json)),reference.selected_manifest_sha256);
   const plan=JSON.parse(reference.rplan_json).plan;
   assert.deepEqual(Object.fromEntries(plan.units.unit_ids.map((id,i)=>[id,plan.assignment[i]+1])),reference.assignments);
  }
  cases.push(fixture);
 }
 const core=cases.find(c=>c.id==='core-nway-empty'),c=cases.find(c=>c.id==='c-nway-effective-c');
 assert.notDeepEqual(core.native_reference.assignments,c.native_reference.assignments);
 const recipe=f=>JSON.parse(f.request.input.sources.prechecks.manifest_json).native_replay_recipe;
 assert.equal(recipe(core).source_graph_sha256,recipe(c).source_graph_sha256);
 assert.equal(recipe(core).state.algorithm.metis.engine,recipe(c).state.algorithm.metis.engine);
 return cases;
}

export function nativeBinaryWidth(bytes){
 if(bytes.toString('ascii',0,2)==='MZ'){
  const at=bytes.readUInt32LE(0x3c);assert.equal(bytes.toString('ascii',at,at+4),'PE\0\0');const magic=bytes.readUInt16LE(at+24);assert([0x10b,0x20b].includes(magic));return magic===0x20b?64:32;
 }
 if(bytes.subarray(0,4).equals(Buffer.from([0x7f,0x45,0x4c,0x46]))){assert([1,2].includes(bytes[4]));return bytes[4]===2?64:32;}
 const magic=bytes.readUInt32BE(0);assert([0xfeedface,0xfeedfacf,0xcefaedfe,0xcffaedfe].includes(magic),'Unsupported native oracle executable header');return [0xfeedfacf,0xcffaedfe].includes(magic)?64:32;
}
