import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const ids=['53001000300','53001000100','53001000200','53001000400'];
const graph={schema_version:1,state:'WA',year:'2020',geoids:ids,adjacency:[[1],[0,2],[1,3],[2]],population:[100,100,100,100],edges:[[0,1,1.5],[1,2,2.5],[2,3,3.5]],areas:[10,10,10,10],exterior_perimeters:[1,1,1,1],centroids:[[-120,47],[-120.1,47],[-120.2,47],[-120.3,47]]};
const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:42,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
const request={graph,options};
const assignments=engine.execute(request).assignments;
const input={operation:'export-engine-plan',request,assignments,label:'Synthetic generated plan',chamber:'test',created_at:'2026-10-04T00:00:00Z'};
const nativePath='target/plan-export-request.json';
let cases=0;
async function parity(value){await fs.writeFile(nativePath,JSON.stringify(value));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[nativePath],{encoding:'utf8'}));const result=engine.execute(value);assert.deepEqual(result,native);cases++;return result;}
const result=await parity(input);
assert.deepEqual(result.document.plan.units.unit_ids,ids);
assert.deepEqual(result.document.plan.assignment,ids.map(id=>assignments[id]-1));
assert.deepEqual(result.context.populations,graph.population);
assert.deepEqual(result.context.subdivisions.county_ids,ids.map(id=>id.slice(0,5)));
for(let u=0;u<ids.length;u++){assert.deepEqual(result.context.graph.adjacency[u].map(e=>e.to),graph.adjacency[u]);for(const e of result.context.graph.adjacency[u]){assert.equal(e.kind,'custom');assert.equal(e.weight,graph.edges.find(([a,b])=>a===Math.min(u,e.to)&&b===Math.max(u,e.to))[2]);}}
assert.equal(result.document.geometry,null);assert.equal(result.context.geometry,undefined);
await parity({operation:'validate-rplan',document:result.document});
const original=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
const profile={...original.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
const cert=await parity({operation:'audit-plan',plan:result.document.plan,context:result.context,profile,constraints:['plan-shape','population','contiguity'],generated_at_utc:input.created_at,lineage:null});
assert.equal(cert.result,'pass');
await parity({operation:'verify-certificate',certificate:cert,plan:result.document.plan,context:result.context});
for(const basis of ['voting-age-population','citizen-voting-age-population','total-population']){
  const withCounts=structuredClone(input);
  withCounts.request.demographics={schema_version:'bisect-demographic-counts-v2',state:'WA',year:'2020',basis,source_label:'Explicit synthetic counts',minority_fractions:Object.fromEntries(ids.map(id=>[id,.5])),counts:Object.fromEntries(ids.map(id=>[id,{total:80,minority:40}]))};
  const exported=await parity(withCounts);
  assert.equal(exported.demographic_basis,basis);
  if(basis==='total-population')assert.equal(exported.context.demographics,undefined);
  else assert.deepEqual(exported.context.demographics,{total_vap:[80,80,80,80],minority_vap:[40,40,40,40]});
  assert.notEqual(exported.context_hash,result.context_hash);
}
let rejected=0;
async function reject(value){assert.throws(()=>engine.execute(value));await fs.writeFile(nativePath,JSON.stringify(value));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[nativePath],{stdio:'pipe'}));rejected++;}
for(const mutate of [v=>delete v.assignments[ids[0]],v=>v.assignments.extra=1,v=>v.assignments[ids[0]]=0,v=>v.assignments[ids[0]]=3,v=>v.request.graph.adjacency[0]=[],v=>v.request.graph.edges[0][2]=-1,v=>v.request.graph.geoids[0]=ids[1],v=>v.label='',v=>v.request.graph.year='oops']){const bad=structuredClone(input);mutate(bad);await reject(bad);}
const altered=structuredClone(result.context);altered.populations[0]++;await reject({operation:'verify-certificate',certificate:cert,plan:result.document.plan,context:altered});
await fs.writeFile('target/plan-export-verification.json',JSON.stringify({exact_native_wasm_cases:cases,rejections:rejected,generated_assignments:true,explicit_unit_order:true,physical_weights:true,native_audit_and_certificate:true},null,2));
console.log(`Plan export: ${cases} exact native/WASM cases and ${rejected} matching rejections; generated assignments, explicit unit order, physical weights, population, county IDs, native audit and certificate verification passed.`);
