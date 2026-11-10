import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {researchRequest,readResearchChains} from '../../web/lab/research-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/research-ensemble-wasm';await fs.mkdir(root,{recursive:true});let matches=0,rejected=0,cliMatches=0;
const make=(offset=0,n=16)=>Object.fromEntries(Array.from({length:4},(_,c)=>['chain-'+c+'.jsonl',Array.from({length:n},(_,i)=>JSON.stringify({step:i,metrics:{x:(i%2?1:-1)+c*offset,trend:i/10}})).join('\n')+'\n']));
const base={operation:'validate-research-ensemble',chains_jsonl:make(),enacted_json:'{ "x": 0, "trend": 0.7 }',rhat_threshold:1.05,ess_min:1};
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matches++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const out=await exact(base),metric=out.report.metrics.find(m=>m.metric==='x');assert.ok(Math.abs(metric.rhat.r_hat-Math.sqrt(15/16))<1e-12);assert.equal(metric.ess.ess,64);assert.equal(metric.enacted_percentile_rank,.5);assert.equal(out.report.all_pass,true);
const failed=await exact({...base,chains_jsonl:make(5)});assert.equal(failed.report.all_pass,false);
const higherEss=await exact({...base,ess_min:100});assert.equal(higherEss.report.all_pass,false);assert.equal((await exact({...base,rhat_threshold:.5})).report.all_pass,false);
await exact({...base,enacted_json:null});
const constantSources=Object.fromEntries(Array.from({length:4},(_,i)=>['chain-'+i+'.jsonl','{ "metrics": {"x": 3} }\n'.repeat(16)]));const constant=await exact({...base,chains_jsonl:constantSources});assert.equal(constant.report.metrics[0].rhat.r_hat,1);assert.equal(constant.report.metrics[0].ess.ess,64);
for(const req of [base,{...base,chains_jsonl:make(5)},{...base,chains_jsonl:constantSources}]){
 const dir=root+'/cli-'+cliMatches;await fs.mkdir(dir,{recursive:true});for(const [name,text] of Object.entries(req.chains_jsonl))await fs.writeFile(dir+'/'+name,text);await fs.writeFile(root+'/enacted.json',req.enacted_json);
 execFileSync('target/debug/bisect.exe',['research','validate-ensemble','--ensemble-dir',dir,'--output',dir+'/report.json','--enacted',root+'/enacted.json','--rhat-threshold',String(req.rhat_threshold),'--ess-min',String(req.ess_min)],{stdio:['ignore','pipe','pipe']});assert.equal(await fs.readFile(dir+'/report.json','utf8'),engine.execute(req).report_json);cliMatches++;
}
const few={...base.chains_jsonl};delete few['chain-3.jsonl'];await reject({...base,chains_jsonl:few});
const unequal={...base.chains_jsonl,'chain-0.jsonl':'{"metrics":{"x":0}}'};await reject({...base,chains_jsonl:unequal});
const missing={...base.chains_jsonl,'chain-0.jsonl':base.chains_jsonl['chain-0.jsonl'].replace('"trend":0,','')};
// Explicit missing metric in one step with otherwise matching lengths.
const missingLines=base.chains_jsonl['chain-0.jsonl'].trim().split('\n').map(JSON.parse);delete missingLines[0].metrics.trend;missing['chain-0.jsonl']=missingLines.map(JSON.stringify).join('\n');await reject({...base,chains_jsonl:missing});
const uniform=text=>Object.fromEntries(Array.from({length:4},(_,i)=>['chain-'+i+'.jsonl',text]));
await reject({...base,chains_jsonl:uniform('{"metrics":{}}\n'.repeat(16))});
await reject({...base,chains_jsonl:uniform('{"metrics":{"x":1,"x":2}}\n'.repeat(16))});
await reject({...base,enacted_json:'{"x":1,"x":2}'});
await reject({...base,chains_jsonl:Object.fromEntries(Array.from({length:4},(_,i)=>['chain-'+i+'.jsonl',`{"metrics":{"x":${i}}}\n`.repeat(16)]))});
await reject({...base,rhat_threshold:0});await reject({...base,ess_min:-1});
await reject({...base,chains_jsonl:{...base.chains_jsonl,'../x.jsonl':base.chains_jsonl['chain-0.jsonl']}});
await reject({...base,chains_jsonl:make(0,5000)});await reject({...base,chains_jsonl:make(0,5001)});
await reject({...base,enacted_json:' '.repeat(3*1024*1024+1)});
await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output:out}));await fs.writeFile(root+'/validate-ensemble.json',out.report_json);
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const inputs={chains:Object.fromEntries(Object.entries(base.chains_jsonl).map(([name,text])=>[name,source(name,text)])),enacted:source('enacted.json',base.enacted_json),rhat_threshold:base.rhat_threshold,ess_min:base.ess_min};
assert.deepEqual(researchRequest(inputs),{...base,strict:false});
const project=createProject({name:'Research ensemble fixture',files:{researchInputs:inputs},operation:base.operation,constraints:[],result:out,lastOperation:base.operation});
assert.deepEqual(parseProject(JSON.stringify(project)).files.researchInputs,inputs);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));
assert.equal(countResultTables(out)[1].rows.find(row=>row[0]==='x').at(-1),'0.5');
assert.equal(countResultTables(engine.execute({...base,enacted_json:null}))[1].rows[0].at(-1),'Not supplied');
let largeRead=false;await assert.rejects(()=>readResearchChains([{name:'huge.jsonl',size:3*1024*1024+1,arrayBuffer:async()=>{largeRead=true;}}]));assert.equal(largeRead,false);
await assert.rejects(()=>readResearchChains([{name:'same.jsonl',size:2},{name:'same.jsonl',size:2}]));
await assert.rejects(()=>readResearchChains([{name:'invalid.jsonl',size:2,arrayBuffer:async()=>new Uint8Array([0xff,0xff]).buffer}]));
console.log(`Research diagnostics: ${matches} native/WASM matches, ${cliMatches} native CLI report byte matches, ${rejected} rejections; independent R-hat/ESS/percentile fixtures, threshold failures, duplicate/missing metrics, constant-chain guard and work limits.`);
