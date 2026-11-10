import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/county-weights-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const size=6,n=size*size,adjacency=[],edges=[];
for(let i=0;i<n;i++){const neighbors=[];if(i%size)neighbors.push(i-1);if(i%size<size-1)neighbors.push(i+1);if(i>=size)neighbors.push(i-size);if(i<n-size)neighbors.push(i+size);adjacency.push(neighbors);for(const j of neighbors)if(i<j)edges.push([i,j,50+((i+j)%7)*3]);}
const graph={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:n},(_,i)=>(i%size<3?'19001':'19003')+String(i).padStart(6,'0')),adjacency,population:Array(n).fill(100),edges,areas:Array(n).fill(10000),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-93+(i%size)*.01,42+Math.floor(i/size)*.01])};
const options={structure:'standard-bisect',weights:'county',search:'single',districts:4,seed:42,seeds:3,steps:20,percentile:0,alpha_county:2,balance_tolerance:25,area_swing:1.1,iterations:20};
const native=(exe,args)=>JSON.parse(execFileSync(exe,args,{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
let cases=0,overflow=0;
for(const alpha of [0,1e-11,1e-10,1.1e-10,.5,1,2,100])for(const search of ['single','multi']){
 const request={graph,options:{...options,search,alpha_county:alpha}},path=root+`/request-${cases}.json`;await fs.writeFile(path,JSON.stringify(request));
 const weights=native('target/debug/examples/county_weights_reference.exe',[path]);
 const result=engine.execute(request);assert.deepEqual(result,native('target/release/examples/execute_request.exe',[path]));
 // Feed weights from the actual native county class into the geographic engine path.
 const control=engine.execute({graph:{...graph,edges:weights},options:{...request.options,weights:'geographic',alpha_county:0}});
 assert.deepEqual(result.assignments,control.assignments);
 const assignments=result.assignments;const weighted=weights.filter(([u,v])=>assignments[graph.geoids[u]]!==assignments[graph.geoids[v]]).reduce((sum,edge)=>sum+edge[2],0);
 const physical=edges.filter(([u,v])=>assignments[graph.geoids[u]]!==assignments[graph.geoids[v]]).reduce((sum,edge)=>sum+edge[2],0);
 assert.equal(result.metrics.weighted_boundary,weighted);assert.equal(result.metrics.graph_boundary_m,physical);
 assert.equal(result.metrics.weighting_evidence.method,'native-county-additive-v1');assert.equal(result.metrics.weighting_evidence.within_county_multiplier,alpha>1e-10?1+alpha:1);
 const same=edges.findIndex(([u,v])=>graph.geoids[u].slice(0,5)===graph.geoids[v].slice(0,5));const cross=edges.findIndex(([u,v])=>graph.geoids[u].slice(0,5)!==graph.geoids[v].slice(0,5));assert.equal(weights[same][2],edges[same][2]*(alpha>1e-10?1+alpha:1));assert.equal(weights[cross][2],edges[cross][2]);cases++;
}
// A positive finite total whose old max(alpha,1) rule stayed finite but native 1+alpha overflows.
const huge={...graph,edges:edges.map((e,i)=>[e[0],e[1],i===0?1.7e308:1])};
const request={graph:huge,options:{...options,alpha_county:.5}},path=root+'/overflow.json';await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>engine.execute(request),/finite numeric range/);assert.throws(()=>native('target/release/examples/execute_request.exe',[path]));overflow++;
assert.ok(engine.execute({graph,options}).metrics.within_requested_tolerance);
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases,native_county_class:'SubdivisionWeighter::county_only',protocol:'native-county-additive-v1',alphas:[0,1e-11,1e-10,1.1e-10,.5,1,2,100],overflow_rejections:overflow},null,2));
console.log(`County weights: ${cases} native/WASM and actual native-weighter assignment/cut comparisons; threshold, fractional alpha, same/cross-county edges, weighting evidence, overflow rejection and recovery passed.`);
