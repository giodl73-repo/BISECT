import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateLabConfig,verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const n=64,w=8,adjacency=Array.from({length:n},(_,i)=>[i%w?i-1:-1,i%w<w-1?i+1:-1,i>=w?i-w:-1,i<n-w?i+w:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(0),centroids:Array.from({length:n},(_,i)=>[-71+i%w*.001,41+Math.floor(i/w)*.001])};
const partisan={schema_version:'bisect-partisan-shares-v1',state:'RI',year:'2020',source_label:'SYNTHETIC boundary fixture',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,i%2?.55:.45]))};
let cases=0;
for(const search of ['percentile','convergence','bisection-ensemble'])for(const districts of [1,3])for(const percentile of [0,1]){
 const options={structure:'standard-bisect',weights:'partisan',search,districts,seed:42,seeds:3,steps:search==='bisection-ensemble'?0:5,percentile,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,dem_threshold:.55,rep_threshold:.45};
 const request={graph,options,partisan};await fs.writeFile('target/partisan-search-edge-request.json',JSON.stringify(request));const native=spawnSync('target/release/examples/execute_request.exe',['target/partisan-search-edge-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);
 const result=engine.execute(request);assert.deepEqual(result,JSON.parse(native.stdout));
 const config={...options,name:'Partisan edge test',mode:'state',states:['RI'],year:'2020',chamber:'congressional',timeout_seconds:60,partisans:{RI:partisan}};validateLabConfig(config);
 const state={code:'RI',metrics:{...result.metrics,district_count:districts,districts:result.metrics.district_metrics,balance_passed:result.metrics.within_requested_tolerance}};verifyLabAssignments(graph,state,result.assignments,config);await verifyDemographicEvidence(graph,state,config);
 const ordinary={...options,weights:'geographic'};delete ordinary.dem_threshold;delete ordinary.rep_threshold;
 const weightedGraph={...graph,edges:graph.edges.map(([u,v])=>[u,v,partisan.dem_shares[graph.geoids[u]]===partisan.dem_shares[graph.geoids[v]]?3:1])};
 const replay=engine.execute({graph:weightedGraph,options:ordinary});assert.deepEqual(result.assignments,replay.assignments);assert.equal(result.metrics.weighted_boundary,replay.metrics.weighted_boundary);cases++;
}
console.log(`Partisan search boundaries: ${cases} exact native/WASM comparisons and weighted replays; default METIS options, one-district inputs, percentile endpoints and zero local proposals passed.`);
