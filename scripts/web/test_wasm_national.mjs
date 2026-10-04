import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2]||'dist/wasm-national';
const search=process.argv[3]||'single';
const seedBudget=Number(process.argv[4]||(search==='multi'?8:1));
assert.ok(['single','multi'].includes(search),'Choose single or multi search');
assert.ok(Number.isSafeInteger(seedBudget)&&seedBudget>=1&&seedBudget<=200,'Seed budget must be between 1 and 200');
assert.ok(search!=='single'||seedBudget===1,'Single search must use one seed');
const reportPath=search==='single'?'target/wasm-national-allocated-validation.json':'target/wasm-national-multi-validation.json';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const moduleBytes=await fs.readFile(path.join(root,'bisect_wasm.wasm'));
const moduleHash=createHash('sha256').update(moduleBytes).digest('hex');
assert.equal(moduleHash,manifest.wasm_sha256,'Catalog must identify the tested module');
const engine=await instantiateEngine(moduleBytes),states=[];
for(const state of manifest.catalog.states){
 const entry=manifest.graphs[state.code+':2020'];
 const graph=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8'));
 const k=state.years.find(y=>y.year==='2020').districts.congressional;
 const started=performance.now();
 try{
  const result=engine.execute({graph,options:{structure:'standard-bisect',weights:'geographic',search,districts:k,seed:42,seeds:seedBudget,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10}});
  assert.equal(result.options.search,search);assert.equal(result.options.seeds,seedBudget);
  const assignment=graph.geoids.map(id=>result.assignments[id]);
  assert.equal(Object.keys(result.assignments).length,graph.geoids.length);
  assert.ok(assignment.every(d=>Number.isInteger(d)&&d>=1&&d<=k));
  const populations=Array(k).fill(0),members=Array.from({length:k},()=>new Set());
  for(let i=0;i<assignment.length;i++){populations[assignment[i]-1]+=graph.population[i];members[assignment[i]-1].add(i);}
  assert.ok(members.every(set=>set.size));
  const componentCounts=members.map(set=>{let count=0;while(set.size){count++;const queue=[set.values().next().value];set.delete(queue[0]);for(let j=0;j<queue.length;j++)for(const v of graph.adjacency[queue[j]])if(set.delete(v))queue.push(v);}return count;});
  const total=populations.reduce((a,b)=>a+b,0),ideal=total/k;
  const deviation=Math.max(...populations.map(p=>Math.abs(p-ideal)/ideal*100));
  assert.equal(result.metrics.population,total);assert.ok(Math.abs(result.metrics.max_deviation_percent-deviation)<1e-9);
  assert.equal(result.metrics.contiguous,componentCounts.every(count=>count===1));
  assert.equal(result.metrics.within_requested_tolerance,deviation<=5);
  assert.ok(engine.memoryBytes()<=128*1024*1024);
  states.push({state:state.code,status:'completed',districts:k,units:assignment.length,deviation,contiguous:result.metrics.contiguous,balance_passed:result.metrics.within_requested_tolerance,memory_bytes:engine.memoryBytes(),seconds:(performance.now()-started)/1000});
 }catch(error){states.push({state:state.code,status:'failed',error:error.message,seconds:(performance.now()-started)/1000});}
 console.log(JSON.stringify(states.at(-1)));
}
assert.equal(states.length,50);
const report={scope:`Allocated 2020 congressional districts; standard bisection, ${search} search, seed 42, ${seedBudget} seeds, 5% tolerance, 10 iterations. Failures and invalid plans are recorded, not treated as passing.`,search,seeds:seedBudget,wasm_sha256:moduleHash,completed:states.filter(s=>s.status==='completed').length,valid:states.filter(s=>s.contiguous&&s.balance_passed).length,states};
await fs.writeFile(reportPath,JSON.stringify(report,null,2));
console.log(JSON.stringify({completed:report.completed,valid:report.valid,states:states.length,failures:states.filter(s=>s.status==='failed'),constraint_misses:states.filter(s=>s.status==='completed'&&(!s.contiguous||!s.balance_passed))}));
// Diagnostics are always saved, but this validation gate passes only when
// every state actually meets both constraints. Completion alone is insufficient.
assert.equal(report.valid,50,`All 50 allocated state plans must be contiguous and within the requested tolerance; inspect ${reportPath}.`);
