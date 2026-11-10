import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/config-run-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const native=(exe,path)=>JSON.parse(execFileSync(exe,[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
const evidence=[];
async function check(graph,yaml_text,districts,oracle=true,chamber='congressional') {
 const input={graph,yaml_text,districts,chamber,source_name:'run.yml',profile:'open'};
 const request={operation:'run-algorithm-config',input};
 const path=`${root}/case-${evidence.length}.json`;await fs.writeFile(path,JSON.stringify(request));
 const result=engine.execute(request);
 assert.deepEqual(result,native('target/release/examples/execute_tool_request.exe',path));
 assert.deepEqual(result.result,engine.execute(result.engine_request));
 if(oracle){const ref=path+'.native.json';await fs.writeFile(ref,JSON.stringify({graph,yaml_text,options:{districts}}));
  assert.deepEqual(result.result.assignments,native('target/debug/examples/config_seed_reference.exe',ref).assignments);}
 assert.equal(result.configuration.yaml_text,yaml_text);
 assert.equal(result.configuration.seed,result.engine_request.options.seed==null?null:String(result.engine_request.options.seed));
 evidence.push({options:result.engine_request.options,interpretation:result.configuration.search_interpretation,oracle});
 return result;
}
// Actual native YAML dispatch oracle, including None versus zero and exact u64.
for(let i=0;i<56;i++) {
 const v=JSON.parse(await fs.readFile(`target/optional-seed-wasm/case-${i}.json`,'utf8'));
 await check(v.graph,v.yaml_text,v.options.districts);
}
const graph=JSON.parse(await fs.readFile('target/optional-seed-wasm/case-0.json','utf8')).graph;
const yaml=(structure,search,extra='')=>`name: configured\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  weights: geographic\n  search: ${search}\n  seed: 42\n  seeds: 4\n  convergence_threshold: 2\n  burst_length: 2\n  n_bursts: 3\n  percentile: 0.5\n  balance_tolerance: 25\n${extra}`;
for(const search of ['convergence','short-burst','short-burst-forest','short-burst-merge-split']){
 const r=await check(graph,yaml('standard-bisect',search),2);
 assert.equal(r.configuration.search_interpretation,'native-search-compositor');
 if(search==='convergence')assert.equal(r.engine_request.options.convergence_unlimited,true);
 for(const structure of ['ratio-optimal','ratio-optimal-area']){
  const q=await check(graph,yaml(structure,search),2);
  assert.equal(q.engine_request.options.search,'multi');assert.equal(q.engine_request.options.seeds,search==='convergence'?2:3);
  assert.equal(q.configuration.search_interpretation,'native-seed-count-per-split');
 }
}
for(const structure of ['nway','prime-factor','compact-polsby','moving-knife','spectral','regionalization','flow-construction']){
 const r=await check(graph,yaml(structure,'multi'),2,structure==='nway');
 assert.equal(r.engine_request.options.search,structure==='compact-polsby'?'multi':'single');
 if(structure==='compact-polsby')assert.equal(r.engine_request.options.compact_epsilon,.01);
}
const metric=await check(graph,yaml('moving-knife','single','  mka_metric: polsby-popper\n'),2,false);
assert.equal(metric.engine_request.options.mka_metric,'reock');
for(const chamber of ['congressional','house','senate']){
 const r=await check(graph,yaml('standard-bisect','single').replace('  balance_tolerance: 25\n',''),2,false,chamber);
 assert.equal(r.configuration.execution_tolerance_percent,chamber==='congressional'?.5:5);
 assert.match(r.configuration.embedded_policy_sha256,/^[0-9a-f]{64}$/);
}
const generous=await check(graph,yaml('standard-bisect','single').replace('balance_tolerance: 25','balance_tolerance: 50'),2);
assert.equal(generous.engine_request.options.balance_tolerance,50);
let rejected=0;
for(const change of [x=>x.yaml_text=x.yaml_text.replace('engine: metis-core','engine: c-ffi'),x=>x.yaml_text=x.yaml_text.replace('  engine: metis-core\n',''),x=>x.chamber='bad\nchamber',x=>x.source_name='../unsafe.yml',x=>x.yaml_text=x.yaml_text.replace('balance_tolerance: 25','balance_tolerance: 50.01'),x=>x.graph={...graph,population:[]},x=>x.unexpected=true]){
 const input={graph,yaml_text:yaml('standard-bisect','single'),districts:2,chamber:'congressional',source_name:'run.yml',profile:'open'};change(input);
 assert.throws(()=>engine.execute({operation:'run-algorithm-config',input}));rejected++;
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence,rejected},null,2));
console.log(`Configured execution: ${evidence.length} native/WASM full matches, ${evidence.filter(x=>x.oracle).length} actual CLI YAML assignment oracles, dispatch interpretation, policy defaults, exact seeds and ${rejected} rejections passed.`);
