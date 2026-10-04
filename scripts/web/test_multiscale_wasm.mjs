import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
await fs.mkdir('target/multiscale-fixtures',{recursive:true});
const width=6,n=width*width,adj=Array.from({length:n},(_,i)=>[i-width,i-1,i+1,i+width].filter(j=>j>=0&&j<n&&Math.abs(j%width-i%width)+Math.abs(Math.floor(j/width)-Math.floor(i/width))===1));
const geoids=Array.from({length:n},(_,i)=>`01${String(1+2*Math.floor(i/12)).padStart(3,'0')}${String(i+1).padStart(6,'0')}`);
const graph={schema_version:1,state:'AL',year:'2020',geoids,adjacency:adj,population:Array(n).fill(100),edges:adj.flatMap((ns,i)=>ns.filter(j=>j>i).map(j=>[i,j,100])),areas:Array(n).fill(1000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-86+i%width/100,32+Math.floor(i/width)/100])};
const bg={schema_version:'bisect-block-group-graph-v1',state:'AL',year:'2020',geoids:geoids.flatMap(id=>[id+'1',id+'2']),population:Array(n*2).fill(50),adjacency:Array.from({length:n*2},(_,i)=>[i^1,...adj[Math.floor(i/2)].map(j=>j*2+i%2)].sort((a,b)=>a-b))};
const options={structure:'standard-bisect',search:'single',weights:'geographic',districts:3,seed:42,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10};
const request=(fine,coarse,adaptive,alpha,steps,percentile,seed)=>({operation:'run-multiscale',input:{request:{graph,options:{...options,seed}},fine_level:fine,coarse_level:coarse,total_steps:steps,alpha,percentile,block_groups:fine==='bg'?bg:null,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null}});
let exact=0,rejected=0,misses=0,disconnected=0;
async function native(req){const file=`target/multiscale-fixtures/request-${exact}-${rejected}.json`;await fs.writeFile(file,JSON.stringify(req));return spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024});}
async function success(req){const ref=await native(req);assert.equal(ref.status,0,ref.stderr);const result=engine.execute(req);assert.deepEqual(result,JSON.parse(ref.stdout));assert.deepEqual(engine.execute(req),result);exact++;
 const fine=req.input.fine_level==='bg'?bg:graph,assignment=result.assignments,populations=Array(options.districts).fill(0);assert.equal(Object.keys(assignment).length,fine.geoids.length);
 fine.geoids.forEach((id,i)=>{assert.ok(assignment[id]>=1&&assignment[id]<=options.districts);populations[assignment[id]-1]+=fine.population[i];});assert.deepEqual(result.populations,populations);
 const ideal=fine.population.reduce((a,b)=>a+b,0)/options.districts,maxDeviation=Math.max(...populations.map(p=>Math.abs((p-ideal)/ideal)*100));assert.equal(result.max_deviation_percent,maxDeviation);assert.equal(result.within_requested_tolerance,maxDeviation<=options.balance_tolerance);
 if(req.input.total_steps===0) {
  const initial=engine.execute(req.input.request).assignments;
  for(const id of fine.geoids)assert.equal(assignment[id],initial[id.slice(0,11)]);
 }
 const connected=populations.map((_,d)=>{const ids=fine.geoids.flatMap((id,i)=>assignment[id]===d+1?[i]:[]);if(!ids.length)return false;const seen=new Set([ids[0]]),queue=[ids[0]];while(queue.length){for(const j of fine.adjacency[queue.pop()])if(assignment[fine.geoids[j]]===d+1&&!seen.has(j)){seen.add(j);queue.push(j);}}return seen.size===ids.length;});assert.deepEqual(result.connected,connected);
 if(!result.within_requested_tolerance)misses++;if(connected.some(v=>!v))disconnected++;
 if(req.input.adaptive)assert.equal(result.evidence.diagnostics.alpha_trace.length,Math.floor(req.input.total_steps/req.input.adaptive.adapt_interval));
 assert.equal(result.evidence.seed,req.input.request.options.seed);assert.match(result.evidence.graph_sha256,/^[0-9a-f]{64}$/);return result;
}
async function failure(req){const ref=await native(req);assert.notEqual(ref.status,0);assert.throws(()=>engine.execute(req));rejected++;}
for(const [fine,coarse]of[['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of[false,true])for(const alpha of[0,.3,1])for(const seed of[42,'18446744073709551615'])for(const percentile of[0,1])await success(request(fine,coarse,adaptive,alpha,10,percentile,seed));
for(const [fine,coarse]of[['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of[false,true])await success(request(fine,coarse,adaptive,.3,0,0,42));
const valid=request('bg','tract',true,.3,10,0,42);
for(const mutate of[
 r=>r.input.fine_level='bad',r=>r.input.coarse_level='bad',r=>r.input.block_groups=null,r=>r.input.total_steps=100001,
 r=>r.input.alpha=-1,r=>r.input.percentile=1.1,r=>r.input.adaptive.adapt_interval=0,r=>r.input.adaptive.gamma_0=2,
 r=>r.input.adaptive.target_accept=-.1,r=>r.input.adaptive.coarse_tol_factor=.5,r=>r.input.unexpected=1,
 r=>r.input.request.options.weights='unweighted',r=>r.input.request.options.seed='18446744073709551616',
 r=>r.input.block_groups.state='RI',r=>r.input.block_groups.year='2010',r=>r.input.block_groups.schema_version='bad',
 r=>r.input.block_groups.population[0]=51,r=>r.input.block_groups.population[0]=-1,
 r=>r.input.block_groups.geoids[0]=r.input.block_groups.geoids[1],r=>r.input.block_groups.geoids[0]='010990000001',
 r=>r.input.block_groups.geoids[0]='é'.repeat(12),r=>r.input.block_groups.adjacency[0].push(10000),
 r=>r.input.block_groups.adjacency[0].push(r.input.block_groups.adjacency[0][0]),r=>r.input.block_groups.adjacency[0]=[],
 r=>r.input.block_groups.population.pop(),r=>r.input.block_groups.geoids.pop(),r=>r.input.block_groups.extra=true,
 r=>r.input.request.graph.geoids[0]='44001000001',r=>{r.input.total_steps=100000;},
 r=>{const b=r.input.block_groups;b.adjacency[0]=b.adjacency[0].filter(j=>j!==1);b.adjacency[1]=b.adjacency[1].filter(j=>j!==0);},
 r=>{const b=r.input.block_groups;for(const [i,j]of[[0,2],[1,3]]){b.adjacency[i]=b.adjacency[i].filter(v=>v!==j);b.adjacency[j]=b.adjacency[j].filter(v=>v!==i);}},
]){const r=structuredClone(valid);mutate(r);await failure(r);}
await failure({...valid,input:{...valid.input,fine_level:'tract'}});
const malformedSeed=structuredClone(valid);malformedSeed.input.request.options.seed=9007199254740992;assert.throws(()=>engine.execute(malformedSeed));
const negativeZero=structuredClone(valid);negativeZero.input.alpha=-0;assert.throws(()=>engine.execute(negativeZero));
await fs.writeFile('target/multiscale-verification.json',JSON.stringify({exact,rejected,population_tolerance_misses:misses,disconnected,scope:'synthetic tract/county and BG/tract/county; fixed and adaptive; no laboratory UI claim'},null,2)+'\n');
console.log(`Multiscale: ${exact} exact native/WASM deterministic cases, ${rejected} matching invalid requests, ${misses} balance misses, ${disconnected} disconnected plans.`);
