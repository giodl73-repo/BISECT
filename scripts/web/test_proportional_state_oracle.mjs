import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyElectionResult} from '../../web/lab/election-input.js';
const root='target/proportional-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/proportional_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[],seedInvariant=new Map();
async function check(g,profile,eta,budget,weights,seed,k=4,real=false){
 const two=g.population.map(p=>p),dem=two.map((v,i)=>v*(profile==='uniform'?.5:i<two.length/2?.75:.25));
 const elections={schema_version:'bisect-election-counts-v1',state:g.state,year:g.year,election_year:'2016',source_label:'SYNTHETIC proportional regression counts; not election observations',counts:Object.fromEntries(g.geoids.map((id,i)=>[id,{democratic:dem[i],two_party:two[i]}]))};
 const search=budget===1?'single':'multi',options={structure:'proportional-section',weights,search,districts:k,seed,seeds:budget,steps:8,percentile:.5,alpha_county:weights==='county'?5:0,balance_tolerance:10,area_swing:1.1,iterations:10,proportional_eta:eta};
 const state_args=['--state',g.state,'--year',g.year,'--metis-engine','metis-core','--partition-mode','proportional-section','--weights-override',weights,'--alpha-county',String(options.alpha_county),'--geosection-seeds',String(budget),'--niter','10','--balance-tolerance','10','--eta',String(eta),'--seed',String(seed)];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k,dem,two_party:two}));
 const native=spawnSync(exe,[file],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options,elections});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr);cases.push({state:g.state,profile,eta,budget,weights,seed,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const ref=JSON.parse(native.stdout),e=actual.metrics.structure_evidence;
 assert.deepEqual(actual.assignments,ref.assignments);assert.equal(actual.metrics.within_requested_tolerance,ref.balanced);assert.equal(ref.eta,eta);assert.equal(ref.seeds,budget);assert.equal(ref.effective_seed,null);
 for(const key of ['left_seats','right_seats','root_cut'])assert.equal(e[key],ref[key]);assert.equal(e.statewide_democratic_share,ref.share);assert.equal(e.eta,eta);assert.equal(e.seeds_per_root,budget);
 await verifyElectionResult(g,options,elections,actual);
 const key=JSON.stringify([g.state,g.population,profile,eta,budget,weights,k]);if(seedInvariant.has(key))assert.deepEqual(actual.assignments,seedInvariant.get(key));else seedInvariant.set(key,actual.assignments);
 cases.push({state:g.state,units:g.geoids.length,profile,eta,budget,weights,seed,k,real,balanced:ref.balanced,contiguous:actual.metrics.contiguous,root_population_passed:e.root_population_within_requested_multiplier,root_votes_passed:e.root_democratic_within_requested_multiplier});
}
for(const profile of ['uniform','stripe'])for(const eta of [1,1.1,1.5,2])for(const budget of [1,3])for(const weights of ['geographic','unweighted','county'])for(const seed of [42,'18446744073709551615'])await check(graph,profile,eta,budget,weights,seed);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const profile of ['uniform','stripe'])for(const eta of [1,1.1,2])await check(g,profile,eta,3,'geographic',42,k,true);}
const sha=b=>createHash('sha256').update(b).digest('hex');const summary={wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.filter(c=>!c.error).length,matching_failures:cases.filter(c=>c.error).length,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,declared_seed_does_not_change_assignments:true,elections:'Explicit synthetic counts, not election observations',scope:'Actual State parsing/conversion and ProportionalSection dispatch. No native CSV acquisition, full retries or arbitrary capacities.',cases_evidence:cases};
await fs.writeFile(root+'/evidence.json',JSON.stringify(summary,null,2));console.log(`Proportional State: ${summary.exact_assignment_matches} exact assignments/root outputs, ${summary.matching_failures} matching failures, ${summary.balance_misses} final misses, ${summary.disconnected_plans} disconnected plans; eta and actual seed budgets checked.`);
