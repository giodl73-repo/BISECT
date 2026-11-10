import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,validateLabConfig} from '../../web/lab/laboratory-project.js';
const root='target/smc-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/smc_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[],assignmentsBySampler=new Map();let failures=0;
async function check(g,particles,threshold,seed,percentile,final,k=4,real=false){
 const options={structure:'standard-bisect',weights:'geographic',search:'smc-percentile',districts:k,seed,seeds:8,steps:20,percentile,alpha_county:0,balance_tolerance:final,area_swing:1.1,iterations:10,smc_particles:particles,smc_resample_threshold:threshold};
 validateLabConfig({...options,name:'SMC final balance',mode:'state',states:[g.state],year:g.year,chamber:'congressional',timeout_seconds:60});
 const state_args=['--state',g.state,'--year',g.year,'--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override','geographic','--search','smc-percentile','--particles',String(particles),'--smc-resample-threshold',String(threshold),'--balance-tolerance',String(final),'--percentile',String(percentile),'--seed',String(seed)];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k}));const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error));failures++;cases.push({state:g.state,particles,threshold,seed,percentile,final,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.final_tolerance,final/100);assert.equal(actual.metrics.structure_evidence.sampler_tolerance,.005);assert.equal(reference.sampler_tolerance,.005);
 const samplerKey=JSON.stringify([g.state,g.population,particles,threshold,seed,percentile,k]);if(assignmentsBySampler.has(samplerKey))assert.deepEqual(actual.assignments,assignmentsBySampler.get(samplerKey));else assignmentsBySampler.set(samplerKey,actual.assignments);
 const config={...options,seed:typeof seed==='string'&&BigInt(seed)<=9007199254740991n?Number(seed):seed};const state={metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(g,state,actual.assignments,config);
 cases.push({state:g.state,units:g.geoids.length,particles,threshold,seed,percentile,final,k,real,balanced:reference.balanced,contiguous:actual.metrics.contiguous});
}
for(const particles of [8,24])for(const threshold of [0,.5,1])for(const seed of [42,'18446744073709551615'])for(const percentile of [0,.5,1])for(const final of [.5,5,25])await check(graph,particles,threshold,seed,percentile,final);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const final of [.5,5,25])await check(g,16,.5,42,.5,final,k,true);}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.length-failures,matching_failures:failures,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,assignment_invariant_under_final_tolerance:true,scope:'Actual State argument conversion/native SMC selection on identical supplied graphs. Fixed sampler tolerance is distinct from final checks; no calibrated-distribution, full acquisition or capacity claim.',cases_evidence:cases},null,2));
console.log(`SMC State: ${cases.length-failures} exact native/WASM matches, ${failures} matching failures, ${cases.filter(c=>c.balanced===false).length} final-balance misses; final tolerance does not alter sampling.`);
