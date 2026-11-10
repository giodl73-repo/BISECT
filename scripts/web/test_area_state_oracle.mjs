import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {verifyLabAssignments} from '../../web/lab/laboratory-project.js';
import {parseSeed} from '../../web/lab/static.js';

const root='target/area-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/area_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`${i<n/2?'44001':'44003'}${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array.from({length:n},(_,i)=>100+i*3),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[];let failures=0;
async function check(g,init,search,seed,weights,swing,k=2,legacy=false,real=false){
 const state_args=['--state',g.state,'--year',g.year,'--metis-engine','metis-core','--partition-mode',legacy?'areasection':'edge-weighted',...(legacy?[]:['--structure','ratio-optimal-area']),'--area-section-init',init,'--area-swing',String(swing),'--weights-override',weights,'--alpha-county','2','--search',search,'--seeds','3','--niter','10','--balance-tolerance','25',...(seed===null?[]:['--seed',seed])];
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,state_args,districts:k}));
 const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 const effectiveSeed=legacy?null:seed;
 const options={structure:'ratio-optimal-area',weights,search,districts:k,seed:effectiveSeed,seeds:3,steps:20,percentile:0,alpha_county:weights==='county'?2:0,balance_tolerance:25,area_swing:swing,area_init:init,iterations:10};
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error));failures++;cases.push({state:g.state,init,search,seed,weights,swing,k,legacy,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.assignments,reference.assignments);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);
 assert.equal(reference.compositor_seed,effectiveSeed);assert.equal(reference.declared_seed,seed);assert.equal(reference.effective_base_seed,effectiveSeed??'1');assert.equal(reference.initializer,init);assert.equal(reference.seeds_per_ratio,search==='single'?1:3);
 assert.equal(actual.metrics.root_split.area_swing,reference.area_swing);assert.equal(reference.area_swing,swing);
 assert.equal(actual.metrics.root_split.left_districts,reference.left_districts);assert.equal(actual.metrics.root_split.right_districts,reference.right_districts);assert.equal(actual.metrics.structure_evidence.direction_radians,reference.direction_radians);
 const verificationConfig={...options,seed:parseSeed(effectiveSeed??'1')};
 const state={metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(g,state,actual.assignments,verificationConfig);
 for(const field of ['initialization','orientations','direction_radians','directional_lambda','scope']){const bad=structuredClone(state);bad.metrics.structure_evidence[field]='tampered';assert.throws(()=>verifyLabAssignments(g,bad,actual.assignments,verificationConfig),/AreaSection initialization evidence/);}
 cases.push({state:g.state,units:g.geoids.length,init,search,declared_seed:seed,compositor_seed:effectiveSeed,weights,swing,k,legacy,real,balanced:reference.balanced,contiguous:actual.metrics.contiguous,assignment_match:true});
}
for(const init of ['ratio-optimal','moving-knife'])for(const search of ['single','multi'])for(const seed of [null,'42','18446744073709551615'])for(const weights of ['geographic','unweighted','county'])for(const swing of [1.1,1.5])await check(graph,init,search,seed,weights,swing);
for(const init of ['ratio-optimal','moving-knife'])for(const search of ['single','multi'])for(const seed of ['42','18446744073709551615'])await check(graph,init,search,seed,'geographic',1.1,2,true);
for(const k of [3,4])for(const init of ['ratio-optimal','moving-knife'])await check(graph,init,'multi','42','geographic',1.5,k);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const init of ['ratio-optimal','moving-knife'])for(const search of ['single','multi'])await check(g,init,search,'42','geographic',1.1,k,false,true);}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.length-failures,matching_failures:failures,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,scope:'Actual State clap parsing and AlgorithmConfig::from_state_args, followed by native AreaSection dispatch on identical supplied graphs/geometry; original filesystem acquisition and full capacities remain unproven.',cases_evidence:cases},null,2));
console.log(`AreaSection State: ${cases.length-failures} actual CLI/WASM assignment/balance/direction matches, ${failures} matching failures; legacy seed reset, recursive counts, real RI/IA graphs and tamper checks.`);
