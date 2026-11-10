import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments,verifyDemographicEvidence,validateLabConfig} from '../../web/lab/laboratory-project.js';
const root='target/vra-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/vra_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[],plans=new Map();let failures=0;
async function check(g,profile,weights,steps,threshold,seed,final,k=4,real=false){
 const fractions=g.geoids.map((_,i)=>profile==='uniform'?.6:i<g.geoids.length/2?.9:.2),demographics={schema_version:'bisect-demographic-fractions-v1',state:g.state,year:g.year,basis:'total-population',source_label:'SYNTHETIC VRA regression fractions; not Census observations',minority_fractions:Object.fromEntries(g.geoids.map((id,i)=>[id,fractions[i]]))};
 const options={structure:'standard-bisect',weights,search:'vra-recom',districts:k,seed,seeds:8,steps,percentile:.5,alpha_county:0,balance_tolerance:final,area_swing:1.1,iterations:10,vra_threshold:threshold};
 const config={...options,name:'VRA final balance',mode:'state',states:[g.state],year:g.year,chamber:'congressional',demographics:{[g.state]:demographics},timeout_seconds:60};validateLabConfig(config);
 const state_args=['--state',g.state,'--year',g.year,'--metis-engine','metis-core','--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override',weights,'--search','vra-recom','--seeds',String(steps),'--niter','10','--balance-tolerance',String(final),'--vra-threshold',String(threshold),'--percentile','0.5','--seed',String(seed)];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k,fractions}));const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options,demographics});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error));failures++;cases.push({state:g.state,profile,weights,steps,threshold,seed,final,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);assert.deepEqual(actual.metrics.structure_evidence.run,reference.report);assert.equal(actual.metrics.structure_evidence.population_tolerance_percent,.5);
 const key=JSON.stringify([g.state,g.population,profile,weights,steps,threshold,seed,k]);if(plans.has(key))assert.deepEqual(actual.assignments,plans.get(key));else plans.set(key,actual.assignments);
 const state={code:g.state,metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(g,state,actual.assignments,config);await verifyDemographicEvidence(g,state,config);
 cases.push({state:g.state,units:g.geoids.length,profile,weights,steps,threshold,seed,final,k,real,balanced:reference.balanced,contiguous:actual.metrics.contiguous,protected:reference.report.protected_districts.length});
}
for(const profile of ['uniform','stripe'])for(const weights of ['geographic','unweighted'])for(const steps of [0,8])for(const threshold of [.5,.8])for(const seed of [42,'18446744073709551615'])for(const final of [.5,5])await check(graph,profile,weights,steps,threshold,seed,final);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const final of [.5,5,25])await check(g,'stripe','geographic',8,.5,42,final,k,true);}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.length-failures,matching_failures:failures,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,assignment_invariant_under_final_tolerance:true,demographics:'Explicit synthetic fractions, not Census observations',scope:'Actual State argument conversion/native dispatch with explicitly aligned fractions. No native CSV acquisition, full retries, capacities or legal certification claim.',cases_evidence:cases},null,2));
console.log(`VRA State: ${cases.length-failures} exact assignments/native preservation reports, ${failures} matching failures, ${cases.filter(c=>c.balanced===false).length} final misses; fixed proposals and independent final tolerance.`);
