import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';

const root='target/pt-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/pt_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[];let failures=0;
async function check(g,steps,replicas,swap,cold,hot,seed,weights,percentile,k=4,real=false){
 const options={structure:'standard-bisect',weights,search:'parallel-tempering',districts:k,seed,seeds:steps,steps:20,percentile,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10,pt_replicas:replicas,pt_swap_interval:swap,pt_cold_tol:cold,pt_hot_tol:hot};
 validateLabConfig({...options,name:'Independent cold tolerance',mode:'state',states:[g.state],year:g.year,chamber:'congressional',timeout_seconds:60});
 const state_args=['--state',g.state,'--year',g.year,'--metis-engine','metis-core','--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override',weights,'--search','parallel-tempering','--seeds',String(steps),'--niter','10','--balance-tolerance','5','--pt-replicas',String(replicas),'--pt-swap-interval',String(swap),'--pt-cold-tol',String(cold),'--pt-hot-tol',String(hot),'--percentile',String(percentile),'--seed',String(seed)];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k}));const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error));failures++;cases.push({state:g.state,steps,replicas,swap,cold,hot,seed,weights,percentile,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.cold_tolerance,cold);assert.equal(reference.final_tolerance,.05);
 const verification={...options,seed:typeof seed==='string'&&BigInt(seed)<=9007199254740991n?Number(seed):seed};
 const state={metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(g,state,actual.assignments,verification);
 assert.equal(actual.metrics.structure_evidence.cold_tolerance,cold);assert.equal(actual.metrics.structure_evidence.hot_tolerance,hot);
 if(!steps&&cold<=.25)assert.deepEqual(actual.assignments,engine.execute({graph:g,options:{...options,search:'single',seeds:1,balance_tolerance:cold*100,pt_replicas:undefined,pt_swap_interval:undefined,pt_cold_tol:undefined,pt_hot_tol:undefined}}).assignments);
 cases.push({state:g.state,units:g.geoids.length,steps,replicas,swap,cold,hot,final_tolerance:.05,seed,weights,percentile,k,real,balanced:reference.balanced,contiguous:actual.metrics.contiguous});
}
for(const steps of [0,10])for(const replicas of [1,3])for(const swap of [1,7])for(const cold of [.02,.1,.5])for(const seed of [42,'18446744073709551615'])for(const weights of ['geographic','unweighted'])for(const percentile of [0,1])await check(graph,steps,replicas,swap,cold,Math.max(cold,.2),seed,weights,percentile);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const cold of [.02,.1])await check(g,10,3,2,cold,.2,42,'geographic',.5,k,true);}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.length-failures,matching_failures:failures,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,scope:'Actual State argument conversion and native PT dispatch on identical supplied graphs; final balance is independent of the ladder. No calibrated sampling, full acquisition or arbitrary-capacity claim.',cases_evidence:cases},null,2));
console.log(`PT State: ${cases.length-failures} exact native CLI/WASM matches, ${failures} matching failures, ${cases.filter(c=>c.balanced===false).length} final-balance misses; independent cold/final tolerances and zero-work initialization.`);
