// Independent native State first-attempt oracles composed under its retry policy.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';

const root='target/geometric-state-retries';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const exe='target/release/examples/geometric_state_reference.exe',sha=b=>createHash('sha256').update(b).digest('hex');
const side=8,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(j=>j>=0));
const synthetic={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array.from({length:n},(_,i)=>50+i%9),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i%side*.001,41+Math.floor(i/side)*.001])};
const records=[];
async function check(graph,structure,seed,k,tolerance,fixture){
 const cvd=structure==='centroidal-voronoi';
 const options={structure,weights:'unweighted',search:'single',districts:k,seed,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:tolerance,area_swing:1.1,iterations:10,...(cvd?{cvd_iters:20,cvd_metric:'graph-distance'}:{mka_orientations:180,mka_metric:'reock'})};
 const request={operation:'run-engine-build',request:{graph,options}},caseRoot=`${root}/case-${records.length}`;await fs.mkdir(caseRoot,{recursive:true});await fs.writeFile(caseRoot+'/request.json',JSON.stringify(request));
 let result,error;try{result=engine.execute(request);}catch(e){error=e;}
 const attempts=[];let accepted=null,acceptedAssignments,exportStatus;
 for(let attempt=0;attempt<=50;attempt++){
  // Native policy: absent seed remains absent only on attempt zero; u64 wraps.
  const effective=attempt===0?seed:BigInt.asUintN(64,BigInt(seed??0)+BigInt(attempt)).toString();
  const state_args=['--state',graph.state,'--year',graph.year,'--districts',String(k),'--balance-tolerance',String(tolerance),'--metis-engine','metis-core','--partition-mode',structure,...(effective===null?[]:['--seed',effective])];
  const file=caseRoot+`/attempt-${attempt}.json`;await fs.writeFile(file,JSON.stringify({graph,state_args}));
  const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:16*1024*1024,timeout:120000});assert.equal(native.error,undefined);assert.equal(native.status,0,native.stderr);
  const reference=JSON.parse(native.stdout);assert.equal(reference.effective_seed,effective??'0');
  attempts.push({attempt,seed:effective,balanced:reference.balanced,assignment_sha256:sha(JSON.stringify(reference.assignments))});
  if(reference.balanced){accepted=attempt;acceptedAssignments=reference.assignments;break;}
 }
 if(accepted!==null){
  assert.equal(error,undefined,error?.message);assert.deepEqual(result.assignments,acceptedAssignments);assert.equal(result.build_report.accepted_attempt,accepted);assert.equal(result.build_report.max_balance_retries,50);
  assert.deepEqual(result.build_report.attempts.map(({attempt,seed,balanced})=>({attempt,seed,balanced})),attempts.map(({attempt,seed,balanced})=>({attempt,seed,balanced})));
  assert.equal(result.options.seed==null?null:String(result.options.seed),attempts.at(-1).seed);assert.equal(result.metrics.within_requested_tolerance,true);
  await fs.writeFile(caseRoot+'/result.json',JSON.stringify(result));
  if(accepted>0||fixture==='prepared-real-state'){
   const bundle=engine.execute({operation:'run-prepared-engine',input:{request_json:JSON.stringify(request),inspect_only:false,label:'Geometric native retry',chamber:'congressional',created_at:'2026-10-18T00:00:00Z'}});
   assert.deepEqual(bundle.result,result);assert.equal(bundle.executed_request.options.seed==null?null:String(bundle.executed_request.options.seed),attempts.at(-1).seed);
   const replay=engine.execute(bundle.executed_request),withoutReport=structuredClone(result);delete withoutReport.build_report;assert.deepEqual(replay,withoutReport);
   const artifact=JSON.parse(bundle.replay_artifact_json);assert.deepEqual(artifact.request,bundle.executed_request);assert.equal(bundle.replay_artifact_sha256,sha(bundle.replay_artifact_json));
   if(bundle.export){assert.equal(bundle.export.document.provenance.producer.engine_options.seed==null?null:String(bundle.export.document.provenance.producer.engine_options.seed),attempts.at(-1).seed);exportStatus='available';}
   else{assert.equal(typeof bundle.export_error,'string');exportStatus=bundle.export_error;}
   await fs.writeFile(caseRoot+'/prepared.json',JSON.stringify(bundle));
  }
 }else{
  assert(error,'Exhaustion must return an error');assert.match(error.message,/exhausted after 51 attempts/);assert.equal(result,undefined);assert.equal(attempts.length,51);assert(attempts.every(a=>!a.balanced));
  await fs.writeFile(caseRoot+'/failure.txt',error.message);
 }
 records.push({fixture,state:graph.state,units:graph.geoids.length,structure,seed,k,tolerance,accepted_attempt:accepted,exhausted:accepted===null,export_status:exportStatus,graph_sha256:sha(JSON.stringify(graph)),attempts});
}
for(const structure of ['centroidal-voronoi','moving-knife'])for(const seed of [null,'42','18446744073709551615'])for(const tolerance of [5,25])await check(synthetic,structure,seed,3,tolerance,'uneven-grid');
await check(synthetic,'centroidal-voronoi',null,2,20,'later-passing-grid');
const impossible={...synthetic,population:synthetic.population.map((_,i)=>i===0?1000000:1)};
for(const structure of ['centroidal-voronoi','moving-knife'])await check(impossible,structure,'18446744073709551615',2,5,'dominant-unit');
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(catalogRoot+'/catalog.json','utf8'));
for(const state of ['RI','IA']){
 const graph=JSON.parse(await fs.readFile(catalogRoot+'/'+catalog.graphs[state+':2020'].graph_ref,'utf8'));
 for(const structure of ['centroidal-voronoi','moving-knife'])await check(graph,structure,'42',2,25,'prepared-real-state');
}
assert(records.some(r=>r.accepted_attempt>0),'Require a later accepted attempt');assert(records.some(r=>r.exhausted),'Require exhaustion');
assert(records.some(r=>r.seed==='18446744073709551615'&&r.attempts.length>1&&r.attempts[1].seed==='0'),'Require observed u64 wraparound');
const evidence={wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),cases:records.length,native_attempt_comparisons:records.reduce((n,r)=>n+r.attempts.length,0),accepted_first:records.filter(r=>r.accepted_attempt===0).length,accepted_later:records.filter(r=>r.accepted_attempt>0).length,exhausted:records.filter(r=>r.exhausted).length,scope:'Native State argument parsing and actual dispatch at every attempted seed, composed under the source-defined 0..50 retry policy. Exact first-passing assignment, seed ledger, wraparound and exhaustion compared to WASM build. Same supplied graphs; full native State filesystem acquisition, output writes, browser UI and other structures remain outside this check.',records};
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2)+'\n');
console.log(`Geometric State retries: ${evidence.cases} builds / ${evidence.native_attempt_comparisons} native attempts; ${evidence.accepted_first} accepted first, ${evidence.accepted_later} accepted later, ${evidence.exhausted} exhausted. Exact assignments, seed order and u64 wraparound verified.`);
