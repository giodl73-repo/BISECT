import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
export async function loadFreshNativeMultiscaleCorpus(){
 const base=new URL('./fresh-native-multiscale/',import.meta.url),raw=await fs.readFile(new URL('manifest.json',base));
 assert.equal(sha(raw),'dada14051bebaea8429f0abdae5c6487b26b531e6f4f4ed626fb7ca5bc1887cc','Native source/reference manifest drifted');const manifest=JSON.parse(raw);
 assert.equal(manifest.schema_version,'bisect-fresh-native-multiscale-corpus-v1');
 assert.deepEqual(manifest.cases.map(row=>row.id),['core-native64-bg','c-native64-bg','tract-county','tract-adaptive']);
 const cases=[];
 for(const ref of manifest.cases){
  assert(/^[a-z0-9-]+\.json$/.test(ref.file));const bytes=await fs.readFile(new URL(ref.file,base));assert.equal(sha(bytes),ref.sha256);
  const row=JSON.parse(bytes);assert.equal(row.schema_version,'bisect-fresh-native-multiscale-fixture-v1');assert.equal(row.id,ref.id);assert.equal(row.reference_kind,'actual-native-completed-plan');
  assert.equal(sha(Buffer.from(row.native_reference.document_json)),row.native_reference.document_sha256);assert.equal(sha(Buffer.from(row.native_reference.context_json)),row.native_reference.context_sha256);
  assert.deepEqual(row.workload,{steps:64,seed:'17',tolerance_percent:'7.125',percentile:'0.5',niter:100,native_sampler:'rand-0.8.6-smallrng-native64-wilson'});
  const input=row.input,flag=name=>input.arguments[input.arguments.indexOf(name)+1];
  assert.equal(flag('--seed'),'17');assert.equal(flag('--multiscale-steps'),'64');assert.equal(Number(flag('--balance-tolerance')),7.125);assert.equal(flag('--niter'),'100');assert.equal(input.multiscale.sampler,row.workload.native_sampler);
  for(const key of ['context_source','context_json','document_json','recipe','receipt','original'])assert(!Object.hasOwn(input,key));
  const plan=JSON.parse(row.native_reference.document_json).plan,context=JSON.parse(row.native_reference.context_json);
  assert.equal(plan.units.unit_ids.length,row.id.endsWith('native64-bg')?32:16);assert.equal(plan.k,2);assert.deepEqual(plan.units,context.units);
  const physical=input.multiscale.fine_source??input.source;
  assert.equal(context.source_hashes.adjacency,'sha256:'+sha(Buffer.from(physical.sources.adjacency.base64,'base64')));
  // Bind the original native build explicitly; the algorithm flag is a separate input.
  cases.push({...row,input:{...input,native_recursive_backend:row.c_profile_required?'c-ffi':'metis-core'},name:row.id,document_json:row.native_reference.document_json,context_json:row.native_reference.context_json});
 }
 return cases;
}

export function freshMultiscaleRefusals(base){
 const rows=[];const add=(name,edit,message)=>{const input=structuredClone(base);edit(input);rows.push({name,input,message});};
 add('native32',v=>v.multiscale.sampler='rand-0.8.6-smallrng-native32-wilson','native64');
 add('missing-fine',v=>v.multiscale.fine_source=null,'complete raw fine source');
 add('scope-mismatch',v=>v.multiscale.fine_source.year='2010','state/year claims');
 add('derived-context-injection',v=>v.multiscale.context_json='{}','unknown field');
 add('incomplete-fine-index',v=>v.multiscale.fine_source.sources.geoids=null,'block-group');
 return rows;
}
