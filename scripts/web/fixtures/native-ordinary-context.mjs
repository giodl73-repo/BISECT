import fs from'node:fs/promises';import path from'node:path';import{fileURLToPath}from'node:url';import{createHash}from'node:crypto';import assert from'node:assert/strict';
export async function loadNativeOrdinaryContextCorpus(){
 const root=path.join(path.dirname(fileURLToPath(import.meta.url)),'native-ordinary-recorded-replay'),manifest=JSON.parse(await fs.readFile(path.join(root,'manifest.json'),'utf8'));
 assert.equal(manifest.schema_version,'bisect-native-ordinary-context-corpus-v1');assert.equal(manifest.cases.length,11);const ids=new Set(),result=[];
 for(const row of manifest.cases){
  assert(typeof row.file==='string'&&/^[a-z0-9_-]+\.json$/.test(row.file));assert(!ids.has(row.id));ids.add(row.id);const raw=await fs.readFile(path.join(root,row.file));assert.equal(createHash('sha256').update(raw).digest('hex'),row.sha256);const fixture=JSON.parse(raw);
  assert.equal(fixture.id,row.id);assert.equal(fixture.reference_kind,'actual-native-completed-plan');assert.equal(fixture.request.operation,'run-native-recorded-replay');assert.equal(typeof fixture.reference.document_json,'string');assert.equal(typeof fixture.reference.context_json,'string');assert(['match','preparation-failed'].includes(fixture.expected_core));assert.equal(fixture.expected_c,'match');result.push(fixture);
 }
 return result;
}
