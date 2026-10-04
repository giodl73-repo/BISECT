import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const wasm=await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/debug/bisect_wasm.wasm');
const engine=await instantiateEngine(wasm);
const n=64, adjacency=[], edges=[];
for(let i=0;i<n;i++) {
  const neighbors=[];
  if(i%8>0)neighbors.push(i-1);if(i%8<7)neighbors.push(i+1);if(i>=8)neighbors.push(i-8);if(i<56)neighbors.push(i+8);
  adjacency.push(neighbors);for(const j of neighbors)if(i<j)edges.push([i,j,100]);
}
export const fixture={graph:{schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:n},(_,i)=>`19001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges,areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-93+(i%8)*.01,42+Math.floor(i/8)*.01])},options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:4,seed:42,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10}};
let tested=0;
const diagnostics=[];
function components(graph, assignments, district) {
  const remaining=new Set(graph.geoids.map((id,i)=>assignments[id]===district?i:-1).filter(i=>i>=0));
  let count=0;
  while(remaining.size) {
    count++;const queue=[remaining.values().next().value];remaining.delete(queue[0]);
    for(let j=0;j<queue.length;j++)for(const v of graph.adjacency[queue[j]])if(remaining.delete(v))queue.push(v);
  }
  return count;
}
for(const structure of ['standard-bisect','nway','bfs-growth','centroidal-voronoi','moving-knife','compact-polsby','ratio-optimal','ratio-optimal-area']) {
  const result=engine.execute({...fixture,options:{...fixture.options,structure}});
  assert.equal(Object.keys(result.assignments).length,n);
  assert.equal(result.metrics.population,6400);
  assert.equal(result.metrics.district_metrics.length,4);
  for(const district of result.metrics.district_metrics)assert.equal(district.components,components(fixture.graph,result.assignments,district.district),structure);
  if(['prime-factor','standard-bisect','nway','ratio-optimal-area'].includes(structure)) {
    assert.equal(result.metrics.contiguous,true,structure);
    assert.equal(result.metrics.within_requested_tolerance,true,structure);
  }
  if(structure==='ratio-optimal-area')assert.equal(result.metrics.root_split.area_within_requested_swing,true);
  diagnostics.push({structure,contiguous:result.metrics.contiguous,balanced:result.metrics.within_requested_tolerance});
  tested++;
}
// Exercise the real prime-factor schedule on a fixture with feasible atomic populations.
const primeWidth=12,primeN=primeWidth*primeWidth;
const primeAdjacency=Array.from({length:primeN},(_,i)=>[...(i%primeWidth?[i-1]:[]),...(i%primeWidth<primeWidth-1?[i+1]:[]),...(i>=primeWidth?[i-primeWidth]:[]),...(i<primeN-primeWidth?[i+primeWidth]:[])]);
const primeGraph={...fixture.graph,geoids:Array.from({length:primeN},(_,i)=>`19001${String(i).padStart(6,'0')}`),adjacency:primeAdjacency,population:Array(primeN).fill(100),edges:primeAdjacency.flatMap((neighbors,u)=>neighbors.filter(v=>v>u).map(v=>[u,v,100])),areas:Array(primeN).fill(10000),exterior_perimeters:Array(primeN).fill(0),centroids:Array.from({length:primeN},(_,i)=>[-93+i/10000,42])};
const spectralResult=engine.execute({graph:primeGraph,options:{...fixture.options,structure:'spectral',weights:'unweighted',districts:6,balance_tolerance:5,steps:200}});
assert.equal(Object.keys(spectralResult.assignments).length,primeN);
assert.equal(spectralResult.metrics.structure_evidence.nodes.length,5);
assert.equal(spectralResult.metrics.structure_evidence.max_iters,200);
assert.deepEqual(engine.execute({graph:primeGraph,options:{...fixture.options,structure:'spectral',weights:'unweighted',districts:6,balance_tolerance:5,steps:200,seed:123}}).assignments,spectralResult.assignments);
assert.deepEqual(engine.execute({graph:primeGraph,options:{...fixture.options,structure:'spectral',weights:'county',alpha_county:2,districts:6,balance_tolerance:5,steps:200}}).assignments,spectralResult.assignments);
tested++;
const primeResult=engine.execute({graph:primeGraph,options:{...fixture.options,structure:'prime-factor',districts:6,balance_tolerance:5}});
assert.equal(Object.keys(primeResult.assignments).length,primeN);
assert.deepEqual(primeResult.metrics.structure_evidence.factor_sequence,[2,3]);
assert.equal(primeResult.metrics.contiguous,true);assert.equal(primeResult.metrics.within_requested_tolerance,true);
assert.throws(()=>engine.execute({...fixture,options:{...fixture.options,structure:'prime-factor'}}),/3% research limit/);
tested++;
assert.throws(()=>engine.execute({...fixture,options:{...fixture.options,structure:'nway',search:'multi'}}),/does not execute/);
assert.throws(()=>engine.execute({...fixture,graph:{...fixture.graph,geoids:Array(n).fill('19001000000')}}),/unique/);
const document=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
const context=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'));
const originalCertificate=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
const validated=engine.execute({operation:'validate-rplan',document});
assert.equal(validated.plan_hash,originalCertificate.plan_hash);
const originalVerified=engine.execute({operation:'verify-certificate',certificate:originalCertificate,plan:document.plan,context});
assert.equal(originalVerified.content_hash,originalCertificate.content_hash);
const profile={...originalCertificate.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
const certificate=engine.execute({operation:'audit-plan',plan:document.plan,context,profile,constraints:['plan-shape','population','contiguity'],generated_at_utc:'2026-10-03T00:00:00Z',lineage:null});
assert.equal(certificate.plan_hash,originalCertificate.plan_hash);
assert.equal(certificate.runtime_provenance?.binary_name || certificate.runtime?.binary_name,'bisect-wasm');
const verified=engine.execute({operation:'verify-certificate',certificate,plan:document.plan,context});
assert.equal(verified.content_hash,certificate.content_hash);
assert.throws(()=>engine.execute({operation:'verify-certificate',certificate:{...certificate,plan_hash:'sha256:tampered'},plan:document.plan,context}),/hash mismatch/);
for(const search of ['convergence','multi','percentile','bisection-ensemble','short-burst','short-burst-forest','short-burst-merge-split','flip','forest-recom','merge-split','parallel-tempering']) {
  const result=engine.execute({...fixture,options:{...fixture.options,search}});
  assert.equal(Object.keys(result.assignments).length,n,search);
  assert.equal(result.metrics.contiguous,true,search);
  assert.equal(result.metrics.within_requested_tolerance,true,search);
  tested++;
}
console.log(JSON.stringify({wasm_bytes:wasm.length,cases_executed:tested,memory_bytes:engine.memoryBytes(),diagnostics}));
