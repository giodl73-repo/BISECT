import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/certified-exact-wasm';await fs.mkdir(root,{recursive:true});
async function native(request){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
let parity=0,rejections=0,oracles=0;
async function exact(request){const output=engine.execute(request);assert.deepEqual(output,await native(request));parity++;return output;}
async function reject(request){assert.throws(()=>engine.execute(request));await assert.rejects(()=>native(request));rejections++;}
function instance(n,k,kind){const ids=Array.from({length:n},(_,i)=>'u'+i);const edges=[];for(let a=0;a<n;a++)for(let b=a+1;b<n;b++)if(kind==='complete'||b===a+1||(kind==='ring'&&a===0&&b===n-1))edges.push({left:a,right:b,weight:1+(a+b)%3});return {schema_version:'certified-recursive-bisection-split-instance-v1',model_id:'certified-standard-bisect-split-v1',node_path:'',parent_certificate_id:null,unit_universe_hash:'sha256:'+createHash('sha256').update(JSON.stringify({unit_ids:ids})).digest('hex'),unit_ids:ids,populations:ids.map((_,i)=>10+i*3),edges,k_parent:k,k_left:Math.floor(k/2),k_right:Math.ceil(k/2),orientation_rule:k%2?'seat-ordered-floor-left-ceil-right':'equal-seats-unit-zero-left'};}
function oracle(input,pops=input.populations.map(BigInt)){
 const n=pops.length;let best;
 const connected=(a,label)=>{const units=a.map((d,i)=>d===label?i:-1).filter(i=>i>=0);const seen=new Set([units[0]]),todo=[units[0]];for(const u of todo)for(const e of input.edges){const v=e.left===u?e.right:e.right===u?e.left:-1;if(v>=0&&a[v]===label&&!seen.has(v)){seen.add(v);todo.push(v);}}return seen.size===units.length;};
 for(let mask=0;mask<2**n;mask++){
  const a=Array.from({length:n},(_,i)=>(mask>>i)&1);if(input.k_left===input.k_right&&a[0])continue;
  if(a.filter(d=>d===0).length<input.k_left||a.filter(d=>d===1).length<input.k_right||!connected(a,0)||!connected(a,1))continue;
  const total=pops.reduce((s,p)=>s+p,0n),left=pops.reduce((s,p,i)=>s+(a[i]===0?p:0n),0n);const abs=x=>x<0n?-x:x;
  const dl=abs(BigInt(input.k_parent)*left-BigInt(input.k_left)*total),dr=abs(BigInt(input.k_parent)*(total-left)-BigInt(input.k_right)*total);
  const cut=input.edges.reduce((s,e)=>s+(a[e.left]!==a[e.right]?BigInt(e.weight):0n),0n);const score=[dl>dr?dl:dr,dl+dr,cut,...a.map(BigInt)];
  const cmp=(x,y)=>{for(let i=0;i<x.length;i++){if(x[i]<y[i])return -1;if(x[i]>y[i])return 1;}return 0;};if(!best||cmp(score,best.score)<0)best={a,score};
 }
 return best;
}
for(const n of [4,6,7])for(const k of [2,3])for(const kind of ['path','ring','complete']){
 const input=instance(n,k,kind),out=await exact({operation:'solve-certified-split',instance_json:JSON.stringify(input)}),best=oracle(input);oracles++;
 assert.equal(out.status,best?'optimal':'infeasible');if(best){assert.deepEqual(out.assignment,best.a);assert.deepEqual(['max_population_deviation_scaled','total_population_deviation_scaled','weighted_boundary_cut'].map(key=>out.objective[key]),best.score.slice(0,3).map(String));}
 const artifacts=JSON.parse(out.artifacts_json);assert.deepEqual(artifacts.instance,input);
 await exact({operation:'verify-certified-split',artifacts_json:out.artifacts_json});
 const bad=structuredClone(artifacts);bad.proof.candidate_count++;await reject({operation:'verify-certified-split',artifacts_json:JSON.stringify(bad)});
 if(kind==='path'){const tree=await exact({operation:'solve-certified-tree',instance_json:JSON.stringify(input)});await exact({operation:'verify-certified-tree',tree_json:tree.tree_json});const altered=JSON.parse(tree.tree_json);altered.leaves[0].unit_ids=[];await reject({operation:'verify-certified-tree',tree_json:JSON.stringify(altered)});await fs.writeFile(root+'/tree.json',tree.tree_json);}
}
const big=instance(4,2,'complete');const pops=[9007199254740993n,9007199254740991n,3n,5n];
const source=JSON.stringify(big).replace(/"populations":\[[^\]]*\]/,'"populations":['+pops.join(',')+']');
const bigOut=await exact({operation:'solve-certified-split',instance_json:source});assert.ok(bigOut.artifacts_json.includes('9007199254740993'));assert.deepEqual(bigOut.assignment,oracle(big,pops).a);await exact({operation:'verify-certified-split',artifacts_json:bigOut.artifacts_json});oracles++;
const disconnected=instance(4,2,'path');disconnected.edges=[];const impossible=await exact({operation:'solve-certified-split',instance_json:JSON.stringify(disconnected)});assert.equal(impossible.status,'infeasible');await exact({operation:'verify-certified-split',artifacts_json:impossible.artifacts_json});
for(const input of [instance(25,2,'path'),{...big,k_parent:100000000}, {...big,unit_universe_hash:'sha256:'+'0'.repeat(64)}, {...big,populations:[-1,2,3,4]}, {...big,edges:[{left:0,right:4,weight:1}]}])await reject({operation:'solve-certified-split',instance_json:JSON.stringify(input)});
await reject({operation:'solve-certified-split',instance_json:'x'.repeat(3*1024*1024+1)});
const tree=JSON.parse(await fs.readFile(root+'/tree.json','utf8'));tree.k=100000000;await reject({operation:'verify-certified-tree',tree_json:JSON.stringify(tree)});
await fs.writeFile(root+'/large-split-artifacts.json',bigOut.artifacts_json);
console.log(`Certified exact WASM: ${parity} exact native/WASM operations, ${oracles} independent exhaustive/BigInt oracles, ${rejections} matching input/proof/tree rejections.`);

let singleParity=0,singleRejected=0;
function single(n){const split=instance(n,2,'path');return {schema_version:'certified-single-district-instance-v1',unit_universe_hash:split.unit_universe_hash,unit_ids:split.unit_ids,populations:split.populations,edges:split.edges};}
// Lexical ID order is canonical; use padded IDs for larger graphs.
function orderedSingle(n){const i=single(n);i.unit_ids=Array.from({length:n},(_,j)=>'u'+String(j).padStart(6,'0'));i.unit_universe_hash='sha256:'+createHash('sha256').update(JSON.stringify({unit_ids:i.unit_ids})).digest('hex');return i;}
for(const n of [1,2,4,250,1000])for(const zero of [false,true]){
 const i=orderedSingle(n);if(zero)i.populations.fill(0);
 const source=JSON.stringify(i),out=await exact({operation:'solve-certified-single',instance_json:source});singleParity++;
 assert.equal(out.status,'trivial-single-district');assert.equal(out.districts,1);assert.equal(out.connected,true);assert.equal(out.population_total,i.populations.reduce((s,p)=>s+BigInt(p),0n).toString());
 const artifacts=JSON.parse(out.artifacts_json);assert.deepEqual(artifacts.instance,i);assert.deepEqual(artifacts.certificate.assignment,Array(n).fill(0));assert.equal(artifacts.certificate.weighted_boundary_cut,0);assert.equal(artifacts.certificate.proof_kind,'trivial-wall-to-wall-k1');
 assert.equal(countResultTables(out)[0].rows.find(row=>row[0]==='Total population')[1],out.population_total);
 const project=createProject({name:'Synthetic single-district certificate',files:{exactSource:{name:'single.json',base64:Buffer.from(source).toString('base64')}},operation:'solve-certified-single',constraints:[],result:out,lastOperation:'solve-certified-single'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await exact({operation:'verify-certified-single',artifacts_json:out.artifacts_json});singleParity++;
 const bad=structuredClone(artifacts);bad.certificate.assignment[0]=1;await reject({operation:'verify-certified-single',artifacts_json:JSON.stringify(bad)});singleRejected++;
 if(n===250&&!zero){await fs.writeFile(root+'/single-instance.json',source);await fs.writeFile(root+'/single-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/single-artifacts.json',out.artifacts_json);}
}
for(const total of [9007199254740993n,9223372036854775807n]){
 const i=orderedSingle(1),source=JSON.stringify(i).replace('"populations":[10]','"populations":['+total+']');
 const out=await exact({operation:'solve-certified-single',instance_json:source});assert.equal(out.population_total,String(total));assert.ok(out.artifacts_json.includes(String(total)));await exact({operation:'verify-certified-single',artifacts_json:out.artifacts_json});singleParity+=2;
}
const singleBad=orderedSingle(2);
for(const i of [{...singleBad,edges:[]},{...singleBad,populations:[-1,2]},{...singleBad,unit_ids:[]},{...singleBad,unit_universe_hash:'sha256:'+'0'.repeat(64)},{...singleBad,edges:[{left:0,right:2,weight:1}]},{...singleBad,unit_ids:Array(100001).fill('x'),populations:Array(100001).fill(0)}]){await reject({operation:'solve-certified-single',instance_json:JSON.stringify(i)});singleRejected++;}
await reject({operation:'solve-certified-single',instance_json:JSON.stringify(singleBad).replace('"populations":[10,13]','"populations":[9223372036854775807,1]')});singleRejected++;
console.log(`Single district: ${singleParity} exact native/WASM operations with independent full-assignment/BigInt population oracles and project/table roundtrips, ${singleRejected} matching rejections.`);

let canonicalParity=0,canonicalOracles=0,canonicalRejected=0;
function canonicalInput(split){return {schema_version:'exact-canonical-instance-v1',model_id:'exact-canonical-k2-exhaustive-v1',unit_ids:split.unit_ids,populations:split.populations,edges:split.edges,k:2};}
function canonicalHash(value){const sort=v=>Array.isArray(v)?v.map(sort):v&&typeof v==='object'?Object.fromEntries(Object.keys(v).sort().map(k=>[k,sort(v[k])])):v;return 'sha256:'+createHash('sha256').update(JSON.stringify(sort(value))).digest('hex');}
for(const n of [4,6,7])for(const kind of ['path','ring','complete'])for(const zero of [false,true]){
 const split=instance(n,2,kind);if(zero)split.populations.fill(0);const input=canonicalInput(split),source=JSON.stringify(input),out=await exact({operation:'solve-certified-canonical',instance_json:source});canonicalParity++;const best=oracle(split);canonicalOracles++;
 assert.equal(out.status,best?'optimal':'infeasible');assert.equal(out.districts,2);if(best){assert.deepEqual(out.assignment,best.a);assert.deepEqual(['max_population_deviation_scaled','total_population_deviation_scaled','weighted_boundary_cut'].map(key=>out.objective[key]),best.score.slice(0,3).map(String));}
 const artifacts=JSON.parse(out.artifacts_json);assert.deepEqual(artifacts.instance,input);await exact({operation:'verify-certified-canonical',artifacts_json:out.artifacts_json});canonicalParity++;
 const project=createProject({name:'Synthetic exact canonical benchmark',files:{exactSource:{name:'canonical.json',base64:Buffer.from(source).toString('base64')}},operation:'solve-certified-canonical',constraints:[],result:out,lastOperation:'solve-certified-canonical'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(countResultTables(out)[0].rows.find(row=>row[0]==='Districts')[1],'2');
 const bad=structuredClone(artifacts);bad.proof.search_commitment='sha256:'+'0'.repeat(64);const proofProjection={...bad.proof};delete proofProjection.transcript_id;bad.proof.transcript_id=canonicalHash(proofProjection);bad.certificate.proof.transcript_id=bad.proof.transcript_id;const projection={...bad.certificate};delete projection.certificate_id;bad.certificate.certificate_id=canonicalHash(projection);await reject({operation:'verify-certified-canonical',artifacts_json:JSON.stringify(bad)});canonicalRejected++;
 if(n===6&&kind==='ring'&&!zero){await fs.writeFile(root+'/canonical-instance.json',source);await fs.writeFile(root+'/canonical-artifacts.json',out.artifacts_json);await fs.writeFile(root+'/canonical-project.bisect',JSON.stringify(project));}
}
const canonSplit=instance(4,2,'complete');
const canonPops=[9007199254740993n,9007199254740991n,3n,5n];const canonSource=JSON.stringify(canonicalInput(canonSplit)).replace(/"populations":\[[^\]]*\]/,'"populations":['+canonPops.join(',')+']');
const canonBig=await exact({operation:'solve-certified-canonical',instance_json:canonSource});assert.deepEqual(canonBig.assignment,oracle(canonSplit,canonPops).a);assert.ok(canonBig.artifacts_json.includes('9007199254740993'));await exact({operation:'verify-certified-canonical',artifacts_json:canonBig.artifacts_json});canonicalParity+=2;canonicalOracles++;
const edgeSource=JSON.stringify(canonicalInput(canonSplit)).replace('"weight":2','"weight":9007199254740993');const bigEdges=structuredClone(canonSplit);bigEdges.edges[0].weight=9007199254740993n;const canonEdge=await exact({operation:'solve-certified-canonical',instance_json:edgeSource});assert.deepEqual(canonEdge.assignment,oracle(bigEdges).a);assert.ok(canonEdge.artifacts_json.includes('9007199254740993'));await exact({operation:'verify-certified-canonical',artifacts_json:canonEdge.artifacts_json});canonicalParity+=2;canonicalOracles++;
const canonEmpty=canonicalInput(instance(4,2,'path'));canonEmpty.edges=[];const canonNoPlan=await exact({operation:'solve-certified-canonical',instance_json:JSON.stringify(canonEmpty)});assert.equal(canonNoPlan.status,'infeasible');await exact({operation:'verify-certified-canonical',artifacts_json:canonNoPlan.artifacts_json});canonicalParity+=2;
for(const input of [{...canonicalInput(canonSplit),k:3},canonicalInput(instance(25,2,'path')),{...canonicalInput(canonSplit),unit_ids:['z','b','c','d']},{...canonicalInput(canonSplit),populations:[-1,2,3,4]},{...canonicalInput(canonSplit),edges:[{left:0,right:4,weight:1}]}]){await reject({operation:'solve-certified-canonical',instance_json:JSON.stringify(input)});canonicalRejected++;}
console.log(`Canonical benchmark: ${canonicalParity} exact native/WASM operations, ${canonicalOracles} independent exhaustive/BigInt oracles, ${canonicalRejected} matching rejections including rehashed proof tampering.`);
