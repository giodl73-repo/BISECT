import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
// Extract only selected raw bytes and user intent. Original outputs are test
// expectations; they are never part of a fresh engine request.
export function freshNativeStateInput(row){
 const selected=row.request.input.sources;
 const recipe=JSON.parse(selected.prechecks.manifest_json).native_replay_recipe;
 const blobs=new Map(selected.sources.map(source=>{
  const bytes=Buffer.from(source.base64,'base64');
  return [createHash('sha256').update(bytes).digest('hex'),source.base64];
 }));
 const role=(key,name)=>{
  const source=recipe.sources.find(source=>source.role===key);
  if(!source?.sha256)return null;
  const base64=blobs.get(source.sha256.replace(/^sha256:/,''));
  assert.equal(typeof base64,'string','Selected raw source must match its original digest');
  return {name,base64};
 };
 const arguments_=['--state','RI','--year','2020','--districts','2','--resolution',recipe.state.plan_resolution,'--seed','18446744073709551615','--metis-engine','metis-core','--adjacency',recipe.adjacency_path];
 const input={arguments:arguments_,source:{state:'RI',year:'2020',sources:{
  adjacency:role('adjacency-binary',path.basename(recipe.adjacency_path)),
  geoids:role('geoid-companion','selected-geoids.json'),
  centroids:role('centroid-companion','selected-centroids.json'),
 }},label:'fresh-native-state',chamber:'congressional',created_at:'2026-11-03T00:00:00Z'};
 const coi=role('explicit-coi','coi.json');
 if(coi){
  const name=recipe.state.coi_weights;arguments_.push('--coi-weights',name);
  input.auxiliary_json=JSON.stringify({native_auxiliary:{coi:{source_name:name,bytes_base64:coi.base64}}});
  input.auxiliary_source_name='selected-native-coi.json';
 }
 assert.deepEqual(Object.keys(input.source).sort(),['sources','state','year']);
 return input;
}

export function freshNativeStateRefusals(base){
 const cases=[];
 const add=(name,edit,message)=>{const input=structuredClone(base);edit(input);cases.push({name,input,message});};
 add('context-injection',v=>v.context_json='{}','unknown field');
 add('receipt-injection',v=>v.receipt={},'unknown field');
 add('plan-injection',v=>v.document_json='{}','unknown field');
 add('empty-label',v=>v.label='','before execution');
 add('oversized-label',v=>v.label='x'.repeat(201),'before execution');
 add('empty-time',v=>v.created_at=' ','before execution');
 add('scope-mismatch',v=>v.source.state='CA','must match');
 add('path-mismatch',v=>v.source.sources.adjacency.name='different.radj','exact adjacency basename');
 add('bad-base64',v=>v.source.sources.adjacency.base64='not-base64','base64');
 add('empty-source',v=>v.source.sources.adjacency.base64='','canonical native adjacency');
 add('supplemental-geometry',v=>v.source.sources.geometry={name:'geometry.json',base64:'e30='},'Supplemental geometry');
 add('custom-county',v=>v.source.sources.county_ids={name:'county.json',base64:'e30='},'county companions');
 add('derived-auxiliary',v=>v.auxiliary_json='{"population":[]}','Derived typed observations');
 add('conflicting-weights',v=>v.auxiliary_json='{"native_weighting":{}}','WeightSpec must match');
 add('unknown-source-field',v=>v.source.unit_kind='Imported','unknown field');
 return cases;
}
