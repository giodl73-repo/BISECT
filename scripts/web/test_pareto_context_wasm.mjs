import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {paretoRequest} from '../../web/lab/package-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const context_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'),context=JSON.parse(context_json);
const root='target/pareto-wasm';await fs.mkdir(root,{recursive:true});
const settings={k:3,population:12,generations:5,tolerance:0.2,base_seed:'42',democratic_shares:null,minority_shares:null};
const base={operation:'sample-pareto-context',context_json,settings};let matched=0,rejected=0,checked=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
let last;
for(const seed of ['0','42','18446744073709551615'])for(const objectives of [false,true]){
 const req={...base,settings:{...settings,base_seed:seed,...(objectives?{democratic_shares:[.1,.2,.3,.4,.5,.6,.7,.8,.9],minority_shares:[.9,.8,.7,.6,.5,.4,.3,.2,.1]}:{})}};
 const out=engine.execute(req);assert.deepEqual(out,await native(req));assert.deepEqual(engine.execute(req),out);matched++;
 assert.equal(out.protocol,'2.0-chacha12-u64-v1');assert.equal(out.base_seed,seed);assert.ok(out.ndjson.includes('\"base_seed\":'+seed));
 const lines=out.ndjson.trimEnd().split('\n'),meta=JSON.parse(lines.pop());assert.equal(meta.file_sha256,createHash('sha256').update(lines.join('\n')+'\n').digest('hex'));assert.equal(meta.runtime_secs,0);
 for(let i=0;i<lines.length;i++){
  const record=JSON.parse(lines[i]),diagnostic=out.diagnostics[i];assert.equal(record.plan.length,context.units.unit_ids.length);
  let cut=0;const pops=Array(settings.k).fill(0n);for(let u=0;u<record.plan.length;u++){const d=record.plan[u]-1;assert.ok(d>=0&&d<settings.k);pops[d]+=BigInt(context.populations[u]);for(const edge of context.graph.adjacency[u])if(edge.to>u&&record.plan[u]!==record.plan[edge.to])cut++;}
  assert.equal(cut,record.ec);const total=pops.reduce((a,b)=>a+b,0n),ideal=Number(total)/settings.k;const dev=Math.max(...pops.map(p=>Math.abs(Number(p)-ideal)/ideal));assert.equal(diagnostic.maximum_population_deviation,dev);assert.equal(diagnostic.passes_final_population_tolerance,dev<=settings.tolerance);
  let connected=true;for(let d=1;d<=settings.k;d++){const nodes=record.plan.map((v,i)=>v===d?i:-1).filter(i=>i>=0);const seen=new Set(nodes.slice(0,1)),stack=nodes.slice(0,1);while(stack.length){const u=stack.pop();for(const e of context.graph.adjacency[u])if(record.plan[e.to]===d&&!seen.has(e.to)){seen.add(e.to);stack.push(e.to);}}connected&&=nodes.length>0&&seen.size===nodes.length;}assert.equal(diagnostic.all_nonempty_and_connected,connected);
  if(!objectives){assert.equal(diagnostic.partisan_deviation,null);assert.equal(diagnostic.minority_deficit,null);}else{
   const dPop=Array(settings.k).fill(0),minority=Array(settings.k).fill(0);let statewideD=0;
   for(let u=0;u<record.plan.length;u++){const p=context.populations[u],d=record.plan[u]-1;dPop[d]+=req.settings.democratic_shares[u]*p;minority[d]+=req.settings.minority_shares[u]*p;statewideD+=req.settings.democratic_shares[u]*p;}
   const seats=pops.filter((p,d)=>p>0n&&dPop[d]/Number(p)>=.5).length;
   assert.ok(Math.abs(diagnostic.partisan_deviation-Math.abs(seats-settings.k*statewideD/Number(total)))<1e-12);
   assert.ok(Math.abs(diagnostic.minority_deficit-pops.reduce((sum,p,d)=>sum+(p>0n?Math.max(0,.5-minority[d]/Number(p)):0),0))<1e-12);
  }checked++;
 }
 assert.equal(countResultTables(out).length,2);last=out;
}
for(const mutate of [s=>s.population=1,s=>s.population=1001,s=>{s.population=1000;s.generations=1000;},s=>s.generations=1001,s=>s.tolerance=-.1,s=>s.tolerance=1.1,s=>s.k=10,s=>s.base_seed='01',s=>s.base_seed='18446744073709551616',s=>s.democratic_shares=[.5],s=>s.minority_shares=[...Array(9).fill(1.1)],s=>s.extra=1]){const req=structuredClone(base);mutate(req.settings);assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const files={auditContextText:{name:'grid.rctx',base64:Buffer.from(context_json).toString('base64')},paretoSettings:settings};assert.deepEqual(paretoRequest(files,settings),base);
const project=createProject({name:'Native Pareto frontier',files,operation:base.operation,constraints:[],result:last,lastOperation:base.operation});assert.equal(parseProject(JSON.stringify(project)).files.auditContextText.base64,files.auditContextText.base64);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/evidence.json',JSON.stringify({matched,rejected,checked}));
console.log(`Pareto: ${matched} native/WASM matches, ${rejected} rejected inputs, ${checked} independent boundary/population/connectivity checks; seed, NDJSON hash, table and project retention passed.`);

let real=0;const realEvidence=[];
for(const [state,k] of [['RI',2],['IA',4],['NC',14]]){
 let text;try{text=await fs.readFile(`target/smc-real-state-wasm/${state}.rctx`,'utf8');}catch(e){if(e.code==='ENOENT')continue;throw e;}
 const req={...base,context_json:text,settings:{...settings,k,tolerance:0.02}},out=engine.execute(req);assert.deepEqual(out,await native(req));
 const context=JSON.parse(text),records=out.ndjson.trimEnd().split('\n').slice(0,-1).map(JSON.parse);
 for(let i=0;i<records.length;i++){
  const record=records[i],pops=Array(k).fill(0n);let cut=0;
  for(let u=0;u<record.plan.length;u++){pops[record.plan[u]-1]+=BigInt(context.populations[u]);for(const e of context.graph.adjacency[u])if(e.to>u&&record.plan[u]!==record.plan[e.to])cut++;}
  assert.equal(out.diagnostics[i].edge_cut,cut);const ideal=Number(pops.reduce((a,b)=>a+b,0n))/k;const dev=Math.max(...pops.map(p=>Math.abs(Number(p)-ideal)/ideal));assert.equal(out.diagnostics[i].maximum_population_deviation,dev);assert.equal(out.diagnostics[i].passes_final_population_tolerance,dev<=req.settings.tolerance);
 }
 real++;realEvidence.push({state,units:context.units.unit_ids.length,frontier:records.length,passes_population:out.diagnostics.filter(d=>d.passes_final_population_tolerance).length});
}
await fs.writeFile(root+'/real-state-evidence.json',JSON.stringify({matches:real,states:realEvidence},null,2));console.log('Prepared-state Pareto parity:',JSON.stringify(realEvidence));
