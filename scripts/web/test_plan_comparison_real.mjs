import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2];assert.ok(root,'Specify a prepared laboratory catalog.');
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let exact=0;
async function parity(request){await fs.writeFile('target/comparison-fixtures/real-request.json',JSON.stringify(request));const actual=engine.execute(request);const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',['target/comparison-fixtures/real-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(actual,native);exact++;return actual;}
for(const code of ['RI','IA','NC']){
 const entry=manifest.graphs[`${code}:2020`];assert.ok(entry);
 const graph=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:42,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
 const documents=[];for(const seed of [42,'18446744073709551615']){
  const request={graph,options:{...options,seed}},generated=engine.execute(request);
  documents.push(await parity({operation:'export-engine-plan',request,assignments:generated.assignments,label:`${code} seed ${seed}`,chamber:'congressional',created_at:'2026-10-04T00:00:00Z'}));
 }
 for(const [a,b]of [[0,1],[1,0]]){
  const result=await parity({operation:'compare-plans',plan_a:documents[a].document,plan_b:documents[b].document,context:documents[a].context});
  assert.equal(result.plan_a.units,graph.geoids.length);assert.equal(result.plan_a.total_population,graph.population.reduce((s,p)=>s+BigInt(p),0n).toString());
  for(const [key,exported]of [['plan_a',documents[a]],['plan_b',documents[b]]])for(const d of result[key].district_measurements){let total=0n;for(let i=0;i<graph.geoids.length;i++)if(exported.document.plan.assignment[i]===d.district_id)total+=BigInt(graph.population[i]);assert.equal(d.population,total.toString());}
 }
}
console.log(`Real RI/IA/NC comparison: ${exact} exact native/WASM exports/comparisons, exact district population sums and max-u64 seeds passed.`);
