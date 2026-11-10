import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/convergence-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/convergence_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[];
async function check(g,weights,threshold,cap,seed,k,real=false){
 const options={structure:'standard-bisect',weights,search:'convergence',districts:k,seed,seeds:threshold,steps:cap??1,percentile:.5,alpha_county:weights==='county'?2:0,balance_tolerance:10,area_swing:1.1,iterations:10,...(cap===null?{convergence_unlimited:true}:{})};
 // Deliberately disagree with --seeds: actual State conversion uses the named
 // --convergence-threshold, regardless of the generic seed budget declaration.
 const state_args=['--state',g.state,'--year',g.year,'--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override',weights,'--alpha-county',String(options.alpha_county),'--metis-engine','metis-core','--search','convergence','--seeds','7','--convergence-threshold',String(threshold),'--niter','10','--balance-tolerance','10','--seed',String(seed)];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k,...(cap===null?{}:{cap_override:cap})}));const native=spawnSync(exe,[file],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr);cases.push({state:g.state,weights,threshold,cap,seed,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const ref=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,ref.assignments);assert.deepEqual(actual.metrics.structure_evidence,ref.summary);assert.equal(ref.threshold,threshold);assert.equal(ref.cli_seed_budget,7);
 const e=ref.summary;assert.equal(e.seed_limit,cap);assert.equal(e.selected_seed,((BigInt(seed)+BigInt(e.selected_index))%(1n<<64n)).toString());if(cap===null){assert.equal(e.halt,'threshold');assert.equal(e.consecutive_non_improving,threshold);}else assert(e.attempted<=cap);
 assert(actual.metrics.within_requested_tolerance);assert(actual.metrics.contiguous);
 cases.push({state:g.state,units:g.geoids.length,weights,threshold,cap,seed,k,real,attempted:e.attempted,rejected:e.rejected,halt:e.halt,selected_seed:e.selected_seed});
}
for(const weights of ['geographic','unweighted','county'])for(const threshold of [1,3])for(const cap of [null,1,8])for(const seed of [42,'18446744073709551615'])for(const k of [2,4])await check(graph,weights,threshold,cap,seed,k);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const weights of ['geographic','unweighted'])for(const cap of [null,1])await check(g,weights,2,cap,'18446744073709551615',k,true);}
const sha=b=>createHash('sha256').update(b).digest('hex'),summary={wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_and_summary_matches:cases.filter(c=>!c.error).length,matching_failures:cases.filter(c=>c.error).length,native_state_threshold_not_generic_seeds:true,scope:'Actual State conversion/native uncapped dispatch; capped runs use explicitly marked native API extension. Selected prepared graphs/parameters only; full orchestration/capacity and optimality unproven.',cases_evidence:cases};
await fs.writeFile(root+'/evidence.json',JSON.stringify(summary,null,2));console.log(`Convergence State/API: ${summary.exact_assignment_and_summary_matches} exact assignments/full stopping summaries, ${summary.matching_failures} matching failures; actual State threshold and cap extensions distinguished.`);
