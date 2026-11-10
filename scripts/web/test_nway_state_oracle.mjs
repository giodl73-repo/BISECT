import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/nway-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const exe='target/release/examples/geometric_state_reference.exe',sha=b=>createHash('sha256').update(b).digest('hex');
const graph=JSON.parse(await fs.readFile('target/geometric-state-oracle/case-0.json','utf8')).graph,records=[];
async function check(g,route,factor,niter,seed,k,weights,objective='cut',real=false){
 const effectiveSeed=route==='vra-preset'?null:seed,ufactor=factor??5,iterations=niter??100;
 const options={structure:'nway',weights,search:'single',districts:k,seed:effectiveSeed,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations,metis_ufactor:ufactor};
 const state_args=['--state',g.state,'--year',g.year,'--districts',String(k),'--balance-tolerance','5','--metis-engine','metis-core',...(route==='vra-preset'?['--partition-mode','metis-vra']:['--partition-mode','edge-weighted','--structure','nway']),'--weights-override',weights,'--objtype',objective,...(factor===null?[]:['--ufactor',String(factor)]),...(niter===null?[]:['--niter',String(niter)]),...(seed===null?[]:['--seed',seed])];
 const file=`${root}/case-${records.length}.json`;await fs.writeFile(file,JSON.stringify({graph:g,state_args}));
 const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:16*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e;}
 assert.equal(Boolean(actual),native.status===0,`${route}/${g.state}: ${native.stderr}; ${error?.message}`);
 const record={state:g.state,units:g.geoids.length,real,route,factor,niter,declared_seed:seed,effective_seed:effectiveSeed,k,weights,declared_objtype:objective};
 if(actual){
  const ref=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,ref.assignments);assert.equal(actual.metrics.within_requested_tolerance,ref.balanced);
  assert.deepEqual(ref.resolved,{structure:'nway',ufactor,niter:iterations,seed:effectiveSeed});
  if(k>1)assert.equal(actual.metrics.structure_evidence.imbalance_factor.effective,Math.max(1,Math.min(1000,ufactor)));
  Object.assign(record,{assignment_match:true,assignment_sha256:sha(JSON.stringify(actual.assignments)),balanced:ref.balanced,contiguous:actual.metrics.contiguous});
 }else{assert(native.stderr.includes(error.message));record.error=error.message;}
 records.push(record);
}
for(const route of ['structure','vra-preset'])for(const seed of [null,'42','18446744073709551615'])for(const factor of [null,0,5,100,4294967295])for(const niter of [null,1,20])await check(graph,route,factor,niter,seed,3,'unweighted');
for(const route of ['structure','vra-preset'])for(const k of [1,4])await check(graph,route,null,null,'42',k,'geographic');
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(catalogRoot+'/catalog.json','utf8'));
for(const state of ['RI','IA']){
 const g=JSON.parse(await fs.readFile(catalogRoot+'/'+catalog.graphs[state+':2020'].graph_ref,'utf8'));
 for(const route of ['structure','vra-preset'])for(const weights of ['unweighted','geographic'])for(const objective of ['cut','vol'])await check(g,route,null,null,'42',2,weights,objective,true);
}
for(const r of records.filter(r=>r.declared_objtype==='vol')){
 const cut=records.find(c=>c.real&&c.state===r.state&&c.route===r.route&&c.weights===r.weights&&c.declared_objtype==='cut');
 assert.equal(r.assignment_sha256,cut.assignment_sha256,'Native objtype is currently unused by State dispatch');
}
const evidence={wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),cases:records.length,exact_assignment_matches:records.filter(r=>r.assignment_match).length,matching_failures:records.filter(r=>r.error).length,balance_misses:records.filter(r=>r.balanced===false).length,native_objtype_cut_vol_match_pairs:records.filter(r=>r.declared_objtype==='vol').length,scope:'Independent State clap/config conversion and actual n-way dispatch on identical supplied graphs; explicit weights avoid claiming native VRA source acquisition. Native default ufactor5/niter100, zero/max-u32 clamps, preset seed reset and structure-override seed retention verified. Parsed objtype is currently unused. Browser volume/internal trials are separate effective controls, not native State objtype equivalence. Full State filesystem/retry/batch orchestration and other backends unproven.',records};
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2)+'\n');
console.log(`N-way State: ${evidence.cases} native comparisons / ${evidence.exact_assignment_matches} exact assignment/balance matches; ${evidence.balance_misses} native balance misses retained, ${evidence.native_objtype_cut_vol_match_pairs} unchanged cut/vol pairs.`);
