import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const root='target/geometric-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile(process.argv.includes('--observe-before')?root+'/before.wasm':'target/wasm32-unknown-unknown/release/bisect_wasm.wasm');
const engine=await instantiateEngine(wasm),exe='target/release/examples/geometric_state_reference.exe';
const sha=b=>createHash('sha256').update(b).digest('hex');
if(process.argv.includes('--observe-before')){
 const file=root+'/case-66.json',input=JSON.parse(await fs.readFile(file,'utf8'));
 const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);
 const expected=JSON.parse(native.stdout),actual=engine.execute({graph:input.graph,options:{structure:'moving-knife',weights:'unweighted',search:'single',districts:2,seed:'42',seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,mka_orientations:180,mka_metric:'reock'}});
 const differences=Object.keys(expected.assignments).filter(id=>expected.assignments[id]!==actual.assignments[id]);assert(differences.length>0);
 await fs.writeFile(root+'/before.json',JSON.stringify({wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),state:'IA',differences: differences.map(id=>({id,native:expected.assignments[id],wasm:actual.assignments[id]})),native_assignments:expected.assignments,wasm_assignments:actual.assignments},null,2)+'\n');
 console.log(`Preserved pre-fix Iowa mismatch: ${differences.length} assignments differ.`);process.exit(0);
}
const side=8,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(j=>j>=0));
const synthetic={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array.from({length:n},(_,i)=>50+i%9),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i%side*.001,41+Math.floor(i/side)*.001])};
const records=[];
const retainedBefore=await fs.readFile(root+'/before.json','utf8').then(JSON.parse,error=>{if(error.code==='ENOENT')return null;throw error;});
async function check(graph,structure,count,metric,seed,k,route,real=false){
 const cvd=structure==='centroidal-voronoi',effectiveCount=count??(cvd?20:180),effectiveMetric=cvd?(['geographic','graph-distance'].includes(metric)?metric:'graph-distance'):(metric==='polsby'?'polsby':'reock');
 const state_args=['--state',graph.state,'--year',graph.year,'--districts',String(k),'--balance-tolerance','25','--metis-engine','metis-core',...(route==='partition-mode'?['--partition-mode',structure]:['--partition-mode','unweighted','--structure',structure]),...(count===null?[]:[cvd?'--cvd-iters':'--mka-orientations',String(count)]),...(metric===null?[]:[cvd?'--cvd-metric':'--mka-metric',metric]),...(seed===null?[]:['--seed',seed])];
 const file=`${root}/case-${records.length}.json`;await fs.writeFile(file,JSON.stringify({graph,state_args}));
 const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:16*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 const options={structure,weights:'unweighted',search:'single',districts:k,seed,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,...(cvd?{cvd_iters:effectiveCount,cvd_metric:effectiveMetric}:{mka_orientations:effectiveCount,mka_metric:effectiveMetric})};
 let actual,error;try{actual=engine.execute({graph,options});}catch(e){error=e;}
 assert.equal(Boolean(actual),native.status===0,`${structure}/${graph.state}/${k}: ${native.stderr}; ${error?.message}`);
 const record={state:graph.state,units:graph.geoids.length,real,structure,count,metric,seed,k,route,graph_sha256:sha(JSON.stringify(graph))};
 if(actual){
  const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments,`${structure}/${graph.state}/${route}/${k}`);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.effective_seed,seed??'0');assert.equal(reference.resolved.count,effectiveCount);
  if(retainedBefore&&real&&graph.state==='IA'&&!cvd&&route==='partition-mode'){
   assert.deepEqual(reference.assignments,retainedBefore.native_assignments);assert.deepEqual(actual.assignments,retainedBefore.native_assignments);
  }
  const e=actual.metrics.structure_evidence;
  // Single-district fast paths may have no iterative structure evidence.
  if(k>1){assert.equal(e[cvd?'iterations_per_split':'orientations_per_split'],Math.max(1,effectiveCount));assert.equal(e[cvd?'distance_metric':'effective_metric'],cvd?effectiveMetric:'reock');}
  Object.assign(record,{assignment_match:true,assignment_sha256:sha(JSON.stringify(actual.assignments)),balanced:reference.balanced,contiguous:actual.metrics.contiguous});
 }else{assert(native.stderr.includes(error.message));record.error=error.message;}
 records.push(record);
}
for(const structure of ['centroidal-voronoi','moving-knife'])for(const route of ['partition-mode','structure'])for(const seed of [null,'42','18446744073709551615'])for(const k of [1,2,3])await check(synthetic,structure,null,null,seed,k,route);
for(const structure of ['centroidal-voronoi','moving-knife'])for(const count of [0,1,36,50])for(const metric of (structure==='centroidal-voronoi'?['geographic','graph-distance','unknown']:['reock','polsby','polsby-popper']))await check(synthetic,structure,count,metric,'42',3,'structure');
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(catalogRoot+'/catalog.json','utf8'));
for(const code of ['RI','IA']){
 const graph=JSON.parse(await fs.readFile(catalogRoot+'/'+catalog.graphs[code+':2020'].graph_ref,'utf8'));
 for(const structure of ['centroidal-voronoi','moving-knife'])for(const route of ['partition-mode','structure'])await check(graph,structure,null,null,'42',2,route,true);
}
const evidence={wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),cases:records.length,assignment_matches:records.filter(r=>r.assignment_match).length,matching_failures:records.filter(r=>r.error).length,balance_misses:records.filter(r=>r.balanced===false).length,disconnected:records.filter(r=>r.contiguous===false).length,scope:'Independent native State clap conversion and first-attempt engine dispatch on identical supplied graphs. Native defaults are explicit in WASM requests; browser legacy defaults are intentionally different. Filesystem acquisition, State population retries, UI automation and arbitrary capacities are not established.',records};
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2)+'\n');
console.log(`CVD/MKA State dispatch: ${evidence.cases} comparisons, ${evidence.assignment_matches} exact assignment/balance matches, ${evidence.matching_failures} matching failures; ${evidence.balance_misses} balance misses and ${evidence.disconnected} disconnected outcomes retained.`);
