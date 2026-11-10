// Retained real replies plus fault injection; no engine, network or pipeline outputs needed.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {gunzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {compareCommandReplies} from './native_command_comparison.mjs';
const compressed=readFileSync(new URL('./fixtures/native-command-comparison.json.gz',import.meta.url));
assert.equal(createHash('sha256').update(compressed).digest('hex'),'4a5f5b36bba8ff121d385df5eb9d1b4e02b93e3a127e1a90dd63e3f989e9787e');
const fixture=JSON.parse(gunzipSync(compressed,{maxOutputLength:8*1024*1024}));
assert.equal(fixture.schema_version,'bisect-command-comparison-fixtures-v1');
assert.equal(fixture.cases.length,12);
let controls=0;const selectors=new Set();
const descend=(value,keys)=>keys.reduce((node,key)=>node[key],value);
for(const row of fixture.cases){
 const {request,wasm,native}=row;assert.equal(request.operation,row.operation);assert(!selectors.has(row.operation));selectors.add(row.operation);
 const before=JSON.stringify([wasm,native]);compareCommandReplies(request,wasm,native,64);
 for(const invalid of [17,'32',null,64]){const bad=structuredClone(wasm);descend(bad,row.width_node).target_pointer_width=invalid;assert.throws(()=>compareCommandReplies(request,bad,native,64));controls++;}
 const wrongNative=structuredClone(native);descend(wrongNative,row.width_node).target_pointer_width=32;assert.throws(()=>compareCommandReplies(request,wasm,wrongNative,64));controls++;
 const missing=structuredClone(wasm);delete descend(missing,row.width_node).target_pointer_width;assert.throws(()=>compareCommandReplies(request,missing,native,64));controls++;
 const wrongSchema=structuredClone(wasm);descend(wrongSchema,row.width_node).schema_version='unexpected';assert.throws(()=>compareCommandReplies(request,wrongSchema,native,64));controls++;
 const bothSchemaA=structuredClone(wasm),bothSchemaN=structuredClone(native);descend(bothSchemaA,row.width_node).schema_version='unexpected';descend(bothSchemaN,row.width_node).schema_version='unexpected';assert.throws(()=>compareCommandReplies(request,bothSchemaA,bothSchemaN,64));controls++;
 const a=structuredClone(wasm),n=structuredClone(native);a.unlisted={target_pointer_width:32};n.unlisted={target_pointer_width:64};assert.throws(()=>compareCommandReplies(request,a,n,64));controls++;
 const changed=structuredClone(wasm);changed.unrelated_field='changed';assert.throws(()=>compareCommandReplies(request,changed,native,64));controls++;
 for(const keys of row.envelopes){const bad=structuredClone(wasm);descend(bad,keys).schema_version='unexpected';assert.throws(()=>compareCommandReplies(request,bad,native,64));controls++;const badN=structuredClone(native);descend(badN,keys).schema_version='unexpected';assert.throws(()=>compareCommandReplies(request,bad,badN,64));controls++;}
 assert.equal(JSON.stringify([wasm,native]),before,'Original replies must remain unchanged');
}
assert.throws(()=>compareCommandReplies({operation:'unlisted-operation'},{target_pointer_width:32},{target_pointer_width:64},64));controls++;
assert.equal(controls,127);
console.log(JSON.stringify({status:'passed',retained_selectors:selectors.size,rejected_corruptions:controls,pipeline_independent:true,scope:fixture.scope}));
