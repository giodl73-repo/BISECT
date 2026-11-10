import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createProject,parseProject} from '../../web/lab/project.js';
import {smcRequest} from '../../web/lab/package-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/smc-context-wasm';await fs.mkdir(root,{recursive:true});
const context_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');const context=JSON.parse(context_json);
const base={operation:'sample-smc-context',context_json,settings:{k:3,particles:32,population_tolerance:0.1,resample_threshold:0.5,base_seed:'42'}};
let parity=0,rejections=0,oracles=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));parity++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejections++;}
const sha=text=>createHash('sha256').update(text).digest('hex');
let result;
for(const [k,seed,threshold,tolerance] of [[1,'0',0,0],[2,'42',0,0.1],[3,'18446744073709551615',1,0.1],[3,'9007199254740993',0.5,0.2]]){
 const req={...base,settings:{...base.settings,k,base_seed:seed,resample_threshold:threshold,population_tolerance:tolerance}};
 const out=await exact(req);assert.equal(out.base_seed,seed);assert.equal(out.ndjson_sha256,sha(out.ndjson));assert.equal((await exact(req)).ndjson,out.ndjson);
 const lines=out.ndjson.trimEnd().split('\n'),metadata=JSON.parse(lines.at(-1));assert.equal(metadata.file_sha256,sha(lines.slice(0,-1).join('\n')+'\n'));assert.ok(lines.at(-1).includes('"base_seed":'+seed));
 const records=lines.slice(0,32).map(line=>JSON.parse(line));assert.equal(records.length,32);
 for(const [i,record] of records.entries()){
  const populations=Array(k).fill(0n);for(const [v,d] of record.plan.entries()){assert.ok(d>=1&&d<=k);populations[d-1]+=BigInt(context.populations[v]);}
  const total=populations.reduce((a,b)=>a+b,0n);const deviation=Math.max(...populations.map(p=>Number(p*BigInt(k)-total<0n?total-p*BigInt(k):p*BigInt(k)-total)/Number(total)));
  let connected=true;for(let d=1;d<=k;d++){const members=record.plan.map((label,v)=>label===d?v:-1).filter(v=>v>=0);if(!members.length){connected=false;continue;}const seen=new Set([members[0]]),stack=[members[0]];while(stack.length){const v=stack.pop();for(const e of context.graph.adjacency[v])if(record.plan[e.to]===d&&!seen.has(e.to)){seen.add(e.to);stack.push(e.to);}}if(seen.size!==members.length)connected=false;}
  assert.ok(Math.abs(out.diagnostics[i].maximum_population_deviation-deviation)<1e-12);assert.equal(out.diagnostics[i].all_nonempty_and_connected,connected);assert.equal(out.diagnostics[i].passes_final_population_tolerance,deviation<=tolerance);assert.equal(record.log_weight===null,!out.diagnostics[i].positive_weight);oracles++;
 }
 const sum=out.diagnostics.reduce((sum,p)=>sum+p.weight,0);assert.ok(Math.abs(sum-1)<1e-12);result=out;
}
for(const settings of [{...base.settings,base_seed:'01'},{...base.settings,base_seed:'18446744073709551616'},{...base.settings,base_seed:42},{...base.settings,particles:0},{...base.settings,particles:1001},{...base.settings,k:0},{...base.settings,k:501},{...base.settings,population_tolerance:-1},{...base.settings,resample_threshold:2},{...base.settings,unknown:1}])await reject({...base,settings});
await reject({...base,context_json:'{'});await reject({...base,context_json:'x'.repeat(3*1024*1024+1)});
await fs.writeFile(root+'/result.json',JSON.stringify(result));
console.log(`SMC context WASM: ${parity} native/WASM matches, ${rejections} matching rejections, ${oracles} independent final-plan population/contiguity checks; exact large seeds and NDJSON payload hashes passed.`);

const settings={...base.settings,base_seed:'18446744073709551615',resample_threshold:1};
const files={auditContextText:{name:'grid.rctx',base64:Buffer.from(context_json).toString('base64')},smcSettings:settings};
const final=await exact(smcRequest(files,settings));
assert.ok(countResultTables(final)[0].rows.some(row=>row[0]==='Base seed'&&row[1]===settings.base_seed));
const project=createProject({name:'Synthetic native SMC ensemble',files,operation:'sample-smc-context',constraints:[],result:final,lastOperation:'sample-smc-context'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/browser-result.json',JSON.stringify(final));
