import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/annealing-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const exe='target/release/examples/geometric_state_reference.exe',sha=b=>createHash('sha256').update(b).digest('hex');
const graph=JSON.parse(await fs.readFile('target/geometric-state-oracle/case-0.json','utf8')).graph;
const records=[];let packetSaved=false;
async function check(g,change,seed,k,weights,niter=10,real=false){
 const options={structure:'simulated-annealing',weights,search:'single',districts:k,seed,seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:niter,...change};
 const state_args=['--state',g.state,'--year',g.year,'--partition-mode','simulated-annealing','--districts',String(k),'--balance-tolerance','25','--metis-engine','metis-core','--weights-override',weights,'--niter',String(niter),...(seed===null?[]:['--seed',seed]),...Object.entries(change).flatMap(([key,value])=>['--'+key.replaceAll('_','-'),String(value)])];
 const file=`${root}/case-${records.length}.json`;await fs.writeFile(file,JSON.stringify({graph:g,state_args}));
 const native=spawnSync(exe,[file],{encoding:'utf8',maxBuffer:16*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options});}catch(e){error=e;}
 assert.equal(Boolean(actual),native.status===0,`${g.state}/${k}: ${native.stderr}; ${error?.message}`);
 const record={state:g.state,units:g.geoids.length,real,k,seed,weights,niter,declared:change};
 if(actual){
  const reference=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,reference.assignments);assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);
  for(const [key,value]of Object.entries({sa_steps_per_tract:10,sa_t0_factor:.01,sa_t_final:.0001})){
   const resolved=change[key]??value;assert.equal(reference.resolved[key.slice(3)],resolved);
   assert.equal(Object.hasOwn(actual.options,key),Object.hasOwn(change,key));
   if(k>1)assert.equal(actual.metrics.structure_evidence[key.slice(3)],resolved);
  }
  const ignored=engine.execute({graph:g,options:{...options,iterations:niter===1?100:1}});assert.deepEqual(ignored.assignments,actual.assignments);
  Object.assign(record,{assignment_match:true,assignment_sha256:sha(JSON.stringify(actual.assignments)),balanced:reference.balanced,contiguous:actual.metrics.contiguous});
  if(!packetSaved&&k===2&&Object.keys(change).length===0){
   const settings={label:'Native annealing defaults',chamber:'congressional',created_at:'2026-10-18T00:00:00Z',inspect_only:false};
   const text=JSON.stringify({graph:g,options}),prepared=engine.execute({operation:'run-prepared-engine',input:{request_json:text,...settings}});
   assert(prepared.export,prepared.export_error);assert.deepEqual(prepared.result,actual);
   const built=engine.execute({operation:'run-engine-build',request:{graph:g,options}});
   assert.equal(built.build_report.accepted_attempt,0);assert.deepEqual(built.assignments,reference.assignments);
   const source={name:'annealing-defaults.json',text},project=createProject({name:'Native annealing defaults',operation:'run-prepared-engine',lastOperation:'run-prepared-engine',constraints:[],files:{preparedEngineInputs:{source,settings}},result:prepared});
   assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/default.bisect',JSON.stringify(project));packetSaved=true;
  }
 }else{assert(native.stderr.includes(error.message));record.error=error.message;}
 records.push(record);
}
for(const change of [{},{sa_steps_per_tract:0},{sa_steps_per_tract:2},{sa_t0_factor:0},{sa_t_final:.01},{sa_steps_per_tract:2,sa_t0_factor:.2,sa_t_final:.01}])for(const seed of [null,'42','18446744073709551615'])for(const k of [1,2,3])await check(graph,change,seed,k,'unweighted');
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(catalogRoot+'/catalog.json','utf8'));
for(const state of ['RI','IA']){
 const real=JSON.parse(await fs.readFile(catalogRoot+'/'+catalog.graphs[state+':2020'].graph_ref,'utf8'));
 for(const weights of ['unweighted','geographic'])for(const change of [{},{sa_steps_per_tract:2}])await check(real,change,'42',2,weights,50,true);
}
for(const change of [{sa_steps_per_tract:10001},{sa_t0_factor:-1},{sa_t_final:0},{sa_t_final:1e7},{structure:'standard-bisect',sa_steps_per_tract:2}]){
 assert.throws(()=>engine.execute({graph,options:{structure:'simulated-annealing',weights:'unweighted',search:'single',districts:2,seed:'42',seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,...change}}));
}
assert(packetSaved);
let nonfinite=0;
for(const key of ['sa_steps_per_tract','sa_t0_factor','sa_t_final'])for(const value of [NaN,Infinity,-Infinity])for(const build of [false,true]){
 const request={graph,options:{structure:'simulated-annealing',weights:'unweighted',search:'single',districts:2,seed:'42',seeds:1,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,[key]:value}};
 assert.throws(()=>engine.execute(build?{operation:'run-engine-build',request}:request),/annealing parameters must be finite/);nonfinite++;
}
const evidence={wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),cases:records.length,assignment_matches:records.filter(r=>r.assignment_match).length,matching_failures:records.filter(r=>r.error).length,invalid_browser_settings_rejected:5,nonfinite_javascript_parameters_rejected_before_json:nonfinite,scope:'Independent State clap parsing and simulated-annealing first-attempt dispatch on identical supplied graphs. Omitted and partial controls use native defaults; exact optional/u64 seeds, zero steps, fixed internal refinement despite niter, recursive counts and real RI/IA inputs checked. Browser work bounds and search applicability remain explicit restrictions. No native filesystem, full States command or arbitrary-capacity claim.',records};
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2)+'\n');
console.log(`Annealing State: ${evidence.cases} comparisons, ${evidence.assignment_matches} exact assignment/balance matches, ${evidence.matching_failures} matching failures; native omitted/partial defaults and five invalid-setting refusals verified.`);
