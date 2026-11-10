import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/config-mka-oracle';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/config-run-wasm/case-75.json','utf8')).input.graph;
const evidence=[];
async function check(graph,orientations,metric,seed,k=2,real=false){
 const yaml_text=`name: mka_oracle\nalgorithm:\n  structure: moving-knife\n  engine: metis-core\n  weights: geographic\n  search: multi\n  seeds: 7\n  balance_tolerance: 25\n${orientations===null?'':`  mka_orientations: ${orientations}\n`}${metric===null?'':`  mka_metric: ${metric}\n`}${seed===null?'':`  seed: ${seed}\n`}`;
 const request={operation:'run-algorithm-config',input:{graph,yaml_text,chamber:'congressional',districts:k,source_name:'mka.yml',profile:'open'}},actual=engine.execute(request);
 const file=path.resolve(root,`case-${evidence.length}.json`);await fs.writeFile(file,JSON.stringify({graph,yaml_text,options:{districts:k}}));
 const native=JSON.parse(execFileSync(path.resolve('target/debug/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000}));
 assert.deepEqual(actual.result.assignments,native.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,native.balanced);
 const labels=graph.geoids.map(id=>native.assignments[id]);let connected=true;
 for(let district=1;district<=k;district++){
  const members=labels.map((label,i)=>label===district?i:null).filter(i=>i!==null),visited=new Set(),pending=members.length?[members[0]]:[];
  while(pending.length){const i=pending.pop();if(visited.has(i))continue;visited.add(i);for(const j of graph.adjacency[i])if(labels[j]===district&&!visited.has(j))pending.push(j);}
  if(!members.length||visited.size!==members.length)connected=false;
 }
 assert.equal(actual.result.metrics.contiguous,connected);
 assert.equal(actual.engine_request.options.mka_orientations,orientations??180);assert.equal(actual.engine_request.options.mka_metric,metric==='polsby'?'polsby':'reock');
 assert.equal(actual.engine_request.options.search,'single');assert.equal(actual.result.metrics.structure_evidence.effective_metric,'reock');
 assert.equal(actual.result.metrics.structure_evidence.orientations_per_split,Math.max(1,orientations??180));
 assert.equal(actual.configuration.yaml_text,yaml_text);assert.equal(actual.configuration.seed,seed);
 evidence.push({state:graph.state,units:graph.geoids.length,graph_sha256:createHash('sha256').update(JSON.stringify(graph)).digest('hex'),declared_orientations:orientations,declared_metric:metric,declared_seed:seed,districts:k,real_exported_graph:real,assignment_match:true,balance_match:true,within_tolerance:actual.result.metrics.within_requested_tolerance,contiguous:actual.result.metrics.contiguous});
}
for(const orientations of [null,0,1,36,180,360])for(const metric of [null,'reock','polsby','polsby-popper'])for(const seed of [null,'42','18446744073709551615'])await check(graph,orientations,metric,seed);
for(const k of [3,4,5])await check(graph,36,'polsby-popper','42',k);
await check(graph,36,'unknown-native-token','42');
const uneven={...graph,population:graph.population.map((_,i)=>i===0?5000:i%7===0?0:50+i*3),centroids:graph.centroids.map(([x,y],i)=>[x+0.004*Math.sin(i),y+0.003*Math.cos(i)])};
for(const metric of ['reock','polsby'])await check(uneven,36,metric,'42',3);
const catalogRoot=process.argv[2]||'dist/wasm-national',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const state of ['RI','IA']){
 const entry=catalog.graphs[state+':2020'];assert(entry,'Required real graph '+state);
 const real=JSON.parse(await fs.readFile(path.join(catalogRoot,entry.graph_ref),'utf8'));
 assert.equal(real.state,state);assert.equal(real.year,'2020');assert.equal(real.geoids.length,state==='RI'?250:896);
 const k=catalog.catalog.states.find(s=>s.code===state).years.find(y=>y.year==='2020').districts.congressional;
 await check(real,36,'reock','42',k,true);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence.length,actual_cli_yaml_dispatch_assignment_matches:evidence.length,synthetic_cases:evidence.filter(e=>!e.real_exported_graph).length,real_graph_cases:evidence.filter(e=>e.real_exported_graph).length,scope:'Native CLI YAML conversion and exact moving-knife dispatch over identical supplied graphs. Original filesystem source acquisition and arbitrary capacity are not established. Native balance/connectivity failures remain outcomes, not certification.',cases_evidence:evidence},null,2));
console.log(`Configured moving-knife: ${evidence.length} actual native CLI YAML-dispatch assignment/balance matches, including two exported real-state graphs; metric tokens, default/zero/nondefault orientations, omitted/u64 seeds and recursive district counts preserved.`);
