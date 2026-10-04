import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const adjacency=[[1,4],[0,2,5],[1,3,6],[2,7],[0,5],[1,4,6],[2,5,7],[3,6]];
const edges=[];for(let u=0;u<8;u++)for(const v of adjacency[u])if(u<v)edges.push([u,v,(u===1&&v===2)||(u===5&&v===6)?1:v-u===4?100:1000]);
const graph={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:8},(_,i)=>`19001${String(i).padStart(6,'0')}`),adjacency,population:Array(8).fill(100),edges,areas:[9000000,9000000,1000000,1000000,9000000,9000000,1000000,1000000],exterior_perimeters:Array(8).fill(0),centroids:Array.from({length:8},(_,i)=>[-93+i%4*.01,42+Math.floor(i/4)*.01])};
const options={structure:'ratio-optimal-area',weights:'geographic',search:'multi',districts:2,seed:42,seeds:64,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:20};
const results=[];
for(const area_swing of [1.1,1.9]){
  const request={graph,options:{...options,area_swing}}, result=engine.execute(request);
  const root=result.metrics.root_split;
  assert.equal(root.area_constraint_active,true);assert.equal(root.area_swing,area_swing);assert.equal(root.area_within_requested_swing,true);
  assert.equal(root.population_multiplier,1.001);assert.equal(root.constraint_scope,'root-only');assert.equal(root.base_seed,42);assert.equal(root.seeds_per_ratio,64);
  assert.equal(result.metrics.contiguous,true);assert.equal(result.metrics.within_requested_tolerance,true);
  const area=graph.geoids.reduce((total,id,i)=>total+(result.assignments[id]<=root.left_districts?graph.areas[i]:0),0);
  assert.equal(root.area_left_m2,area);assert.equal(root.area_fraction_left,area/40000000);
  if(area_swing===1.1)assert.equal(root.area_fraction_left,0.5);else assert.ok(Math.abs(root.area_fraction_left-0.5)>0.3);
  results.push(result);
  await fs.writeFile(`target/area-fixture-${area_swing}.json`,JSON.stringify(request));
}
assert.ok(results[1].metrics.root_split.weighted_boundary<results[0].metrics.root_split.weighted_boundary);
assert.throws(()=>engine.execute({graph:{...graph,areas:[39000000,1,1,1,1,1,1,1]},options}),/no sampled feasible root split/);
assert.throws(()=>engine.execute({graph:{...graph,areas:Array(8).fill(0)},options}),/input dimensions/);
assert.throws(()=>engine.execute({graph:{...graph,areas:Array(8).fill(1e308)},options}),/geometry attributes/);
assert.throws(()=>engine.execute({graph:{...graph,edges:edges.map(([u,v])=>[u,v,1e308])},options}),/numeric range/);
console.log(JSON.stringify({scope:'Two-district dual constraints, tight versus loose area swing and infeasible atomic area; actual WASM.',cases:results.map(result=>result.metrics.root_split)}));
