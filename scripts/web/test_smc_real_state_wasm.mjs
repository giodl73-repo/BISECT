import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM??'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/smc-real-state-wasm';await fs.mkdir(root,{recursive:true});
const nativeReference=process.env.BISECT_SMC_NATIVE_REFERENCE;
let site=path.resolve(process.argv[2]??'dist/wasm-merge-validation');
const catalogText=directory=>fs.readFile(path.join(directory,'catalog.json'),'utf8');
let text;
try{text=await catalogText(site);}catch(error){
 // The integrated builder passes its output toolkit; the graph catalog and
 // its relative assets live at the laboratory site root, one level above it.
 if(error.code!=='ENOENT'||path.basename(site)!=='toolkit')throw error;
 site=path.dirname(site);text=await catalogText(site);
}
const catalog=JSON.parse(text);
const sha=b=>createHash('sha256').update(b).digest('hex');
const evidence=[];
let parity=0,checks=0;
async function exact(req){const args=[root+'/request.json'],options={encoding:'utf8',stdio:['ignore','pipe','pipe'],timeout:120000,maxBuffer:32*1024*1024};await fs.writeFile(args[0],JSON.stringify(req));const out=engine.execute(req);const native=JSON.parse(nativeReference?execFileSync(nativeReference,args,options):execFileSync('target/release/examples/execute_tool_request.exe',args,options));assert.deepEqual(out,native);parity++;return out;}
for(const [state,k,particles] of [['RI',2,32],['IA',4,16],['NC',14,8]]){
 const key=state+':2020',entry=catalog.graphs[key];assert(entry?.graph_ref,'Missing original SMC graph: '+key);const bytes=await fs.readFile(path.join(site,entry.graph_ref));assert.equal(sha(bytes),catalog.assets[entry.graph_ref]);const graph=JSON.parse(bytes);
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:k,seed:'42',seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:2,area_swing:1.1,iterations:10};
 // Context preparation only: the placeholder assignment is never used as a sampled plan.
 const prepared=await exact({operation:'export-engine-plan',request:{graph,options},assignments:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%k+1])),label:'Context preparation only',chamber:'congressional',created_at:'2026-10-04T00:00:00Z'});
 assert.equal(typeof prepared.context_json,'string');
 const req={operation:'sample-smc-context',context_json:prepared.context_json,settings:{k,particles,population_tolerance:0.02,resample_threshold:0.5,base_seed:'42'}};
 const started=performance.now();const sampled=await exact(req);const elapsed=performance.now()-started;
 const records=sampled.ndjson.trimEnd().split('\n').slice(0,particles).map(JSON.parse);
 for(const [i,{plan}] of records.entries()){
  const pops=Array(k).fill(0n);for(const [v,d] of plan.entries())pops[d-1]+=BigInt(graph.population[v]);const total=pops.reduce((a,b)=>a+b,0n);
  const deviation=Math.max(...pops.map(p=>Number(p*BigInt(k)>total?p*BigInt(k)-total:total-p*BigInt(k))/Number(total)));
  let connected=true;for(let d=1;d<=k;d++){const members=plan.map((a,v)=>a===d?v:-1).filter(v=>v>=0);if(!members.length){connected=false;continue;}const seen=new Set([members[0]]),stack=[members[0]];while(stack.length){const v=stack.pop();for(const u of graph.adjacency[v])if(plan[u]===d&&!seen.has(u)){seen.add(u);stack.push(u);}}connected&&=seen.size===members.length;}
  assert.ok(Math.abs(deviation-sampled.diagnostics[i].maximum_population_deviation)<1e-12);assert.equal(connected,sampled.diagnostics[i].all_nonempty_and_connected);checks++;
 }
 const good=sampled.diagnostics.find(p=>p.positive_weight&&p.all_nonempty_and_connected);assert.ok(good);
 const selected=await exact({operation:'select-smc-plan',context_json:prepared.context_json,ensemble_ndjson:sampled.ndjson,settings:{particle_index:good.particle_index,label:state+' sampled plan',chamber:'congressional',created_at:'2026-10-04T00:00:00Z'}});
 assert.deepEqual(JSON.parse(selected.document_json).plan.units.unit_ids,graph.geoids);
 await fs.writeFile(root+'/'+state+'.rctx',prepared.context_json);await fs.writeFile(root+'/'+state+'-ensemble.ndjson',sampled.ndjson);await fs.writeFile(root+'/'+state+'-selected.rplan',selected.document_json);
 const files={auditContextText:{name:state+'.rctx',base64:Buffer.from(prepared.context_json).toString('base64')},smcSettings:req.settings};
 await fs.writeFile(root+'/'+state+'-ensemble.bisect',JSON.stringify(createProject({name:state+' real Census SMC',files,operation:'sample-smc-context',constraints:[],result:sampled,lastOperation:'sample-smc-context'})));
 evidence.push({state,year:2020,units:graph.geoids.length,k,particles,population:graph.population.reduce((a,b)=>a+b,0),seed:'42',resamplings:sampled.resample_count,positive_weight:sampled.diagnostics.filter(p=>p.positive_weight).length,connected:sampled.diagnostics.filter(p=>p.all_nonempty_and_connected).length,passes_population:sampled.diagnostics.filter(p=>p.passes_final_population_tolerance).length,native_and_wasm_ms:elapsed,selected_index:good.particle_index,selected_plan_hash:selected.plan_hash,context_hash:sampled.context_hash});
 console.log(JSON.stringify(evidence.at(-1)));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,checks,evidence},null,2));console.log(`Real-state SMC: ${parity} native/WASM matches, ${checks} independent final-plan checks across RI/IA/NC.`);
