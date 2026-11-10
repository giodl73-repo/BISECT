import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/branch-price-wasm';await fs.mkdir(root,{recursive:true});
let parity=0, rejected=0, oracles=0;
const request=input=>({operation:'solve-branch-price',instance_json:typeof input==='string'?input:JSON.stringify(input)});
async function native(req){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
async function exact(input){const req=request(input),result=engine.execute(req);assert.deepEqual(result,await native(req));assert.equal(result.input_bytes_sha256,'sha256:'+createHash('sha256').update(req.instance_json).digest('hex'));parity++;return result;}
async function reject(input){const req=request(input);assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const path=n=>Array.from({length:n},(_,i)=>[i-1,i+1].filter(j=>j>=0&&j<n));
const fixture=(n,k,tolerance=0)=>({adjacency:path(n),populations:Array(n).fill(10),k,tolerance,formulation_only:false,exact_fixture_limit:12});
// Independent restricted-growth partition enumeration, rather than the native column master.
function oracle(input){
 const {adjacency,populations,k,tolerance}=input,n=populations.length,total=populations.reduce((a,b)=>a+b,0),ideal=total/k;
 const labels=Array(n).fill(0);let best=Infinity;
 function visit(i,max){
  if(i<n){for(let label=0;label<=Math.min(max+1,k-1);label++){labels[i]=label;visit(i+1,Math.max(max,label));}return;}
  if(max!==k-1)return;
  for(let label=0;label<k;label++){
   const units=labels.flatMap((v,i)=>v===label?[i]:[]),pop=units.reduce((s,i)=>s+populations[i],0);
   if(pop<ideal*(1-tolerance)||pop>ideal*(1+tolerance))return;
   const seen=new Set([units[0]]),queue=[units[0]];
   for(let j=0;j<queue.length;j++)for(const v of adjacency[queue[j]])if(labels[v]===label&&!seen.has(v)){seen.add(v);queue.push(v);}
   if(seen.size!==units.length)return;
  }
  let cut=0;for(let u=0;u<n;u++)for(const v of adjacency[u])if(u<v&&labels[u]!==labels[v])cut++;
  best=Math.min(best,cut);
 }
 visit(1,0);return best;
}
for(const n of [4,6,8])for(const k of [1,2,3,4].filter(k=>k<=n))for(const tolerance of [0,0.25]){
 const input=fixture(n,k,tolerance),output=await exact(input),expected=oracle(input);oracles++;
 assert.equal(output.report.status,expected===Infinity?'infeasible':'exact-fixture-optimal');
 assert.equal(output.report.solution?.objective??Infinity,expected);
 if(output.report.solution){const a=output.report.solution.assignment;assert.equal(a.length,n);assert.equal(new Set(a).size,k);assert.equal(output.report.gap,0);}
}
for(const input of [fixture(6,3,1),{...fixture(6,3),adjacency:Array.from({length:6},(_,i)=>Array.from({length:6},(_,j)=>j).filter(j=>j!==i))}]){
 const output=await exact(input);assert.equal(output.report.solution?.objective??Infinity,oracle(input));oracles++;
}
for(const input of [{...fixture(8,4),formulation_only:true},{...fixture(13,1),exact_fixture_limit:12}]){
 const output=await exact(input);assert.equal(output.report.status,'formulation-only');for(const key of ['solution','lower_bound','upper_bound','gap'])assert.equal(output.report[key],null);
}
const big='{"adjacency":[[1],[0]],"populations":[9007199254740993,9007199254740993],"k":2,"tolerance":0,"formulation_only":false,"exact_fixture_limit":12}';
assert.equal((await exact(big)).total_population,'18014398509481986');
for(const input of [fixture(21,1),{...fixture(4,2),k:0},{...fixture(4,2),k:5},{...fixture(4,2),tolerance:-1},{...fixture(4,2),tolerance:1.01},{...fixture(4,2),exact_fixture_limit:13},{...fixture(4,2),populations:[0,1,1,1]},{...fixture(4,2),populations:[1,1]},'{"adjacency":[[1],[0]],"populations":[9223372036854775807,1],"k":1,"tolerance":0,"formulation_only":false,"exact_fixture_limit":12}','{','x'.repeat(3*1024*1024+1)])await reject(input);
for(const adjacency of [[[4],[0],[3],[2]],[[0],[],[],[]],[[1,1],[0],[],[]],[[1],[],[],[]]])await reject({...fixture(4,2),adjacency});
// Summary printed after all artifact checks.

const browserInput=fixture(8,4),browserOutput=await exact(browserInput);
const project=createProject({name:'Synthetic native column benchmark',files:{exactSource:{name:'column-instance.json',base64:Buffer.from(JSON.stringify(browserInput)).toString('base64')}},operation:'solve-branch-price',constraints:[],result:browserOutput,lastOperation:'solve-branch-price'});
assert.deepEqual(parseProject(JSON.stringify(project)),project);
assert.equal(countResultTables(browserOutput)[0].rows.find(row=>row[0]==='Boundary objective')[1],'3');
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));
await fs.writeFile(root+'/browser-result.json',JSON.stringify(browserOutput));
console.log(`Branch-price WASM: ${parity} exact native/WASM matches, ${oracles} independent k-way partition oracles, ${rejected} matching rejections.`);
