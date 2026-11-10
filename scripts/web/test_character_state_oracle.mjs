import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {characterWeights} from '../../web/lab/character-input.js';
import {verifyLabAssignments,verifyDemographicEvidence} from '../../web/lab/laboratory-project.js';
const root='target/character-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe=path.resolve('target/release/examples/character_state_reference.exe');
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const cases=[];
async function check(g,kind,alpha,search,seed,k,real=false){
 const header=kind==='economic'?'geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11':'geoid,pct_single_family,pct_multifamily,pct_owner,housing_vintage';
 const rows=g.geoids.map((id,i)=>id+','+(kind==='economic'?(i%3===0?'0,0,0,0,0,0,0,0,0':i%2?'1000,10,20,30,400,40,100,50,50':'1000,10,20,30,100,200,200,100,100'):i%3===0?'0,0,0,0':i%2?'.8,.1,.6,.2':'.1,.7,.2,.8'));
 const character=engine.execute({operation:'import-character-csv',state:g.state,year:g.year,data_year:'2022',kind,source_label:'SYNTHETIC character observations; not LODES/ACS records',source_base64:Buffer.from(header+'\n'+rows.join('\n')+'\n').toString('base64')});
 const options={structure:'standard-bisect',weights:kind+'-character',search,districts:k,seed,seeds:3,steps:8,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10,character_alpha:alpha};
 const state_args=['--state',g.state,'--year',g.year,'--partition-mode','edge-weighted','--structure','standard-bisect','--weights-override',options.weights,'--metis-engine','metis-core','--search',search,'--seeds','3','--niter','10','--balance-tolerance','10','--seed',String(seed)];
 const request={graph:g,state_args,districts:k,...(alpha===.5?{}:{alpha_override:alpha}),...(kind==='economic'?{raw:g.geoids.map(id=>character.data.raw_counts[id])}:{characters:g.geoids.map(id=>character.data.characters[id])})};
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify(request));const native=spawnSync(exe,[file],{encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({graph:g,options,character});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr);cases.push({state:g.state,kind,alpha,search,seed,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const ref=JSON.parse(native.stdout);assert.deepEqual(actual.assignments,ref.assignments);assert.equal(actual.metrics.within_requested_tolerance,ref.balanced);assert.equal(ref.cli_alpha,.5);assert.equal(ref.alpha,alpha);assert(Math.abs(actual.metrics.weighted_boundary-ref.weighted_cut)<=1e-8*Math.max(1,Math.abs(ref.weighted_cut)));
 const config={...options,name:'Character oracle',mode:'state',states:[g.state],year:g.year,chamber:'congressional',characters:{[g.state]:character},timeout_seconds:60},weights=characterWeights(g,config);
 for(const [u,v,x]of ref.weighted_edges){const edge=g.edges.find(([a,b])=>a===u&&b===v);assert(edge);assert(Math.abs(weights.weight(u,v,edge[2])-x)<=1e-12*Math.max(1,Math.abs(x)));}
 const state={code:g.state,metrics:{...actual.metrics,district_count:k,districts:actual.metrics.district_metrics,balance_passed:actual.metrics.within_requested_tolerance}};verifyLabAssignments(g,state,actual.assignments,config);await verifyDemographicEvidence(g,state,config,actual.assignments);
 if(alpha===1){const geo={...options,weights:'geographic'};delete geo.character_alpha;assert.deepEqual(actual.assignments,engine.execute({graph:g,options:geo}).assignments);}
 cases.push({state:g.state,units:g.geoids.length,kind,alpha,search,seed,k,real,balanced:ref.balanced,contiguous:actual.metrics.contiguous,scope:alpha===.5?'State default blend':'Native weighter API override'});
}
for(const kind of ['economic','housing'])for(const alpha of [0,.5,1])for(const search of ['single','multi'])for(const seed of [42,'18446744073709551615'])for(const k of [2,4])await check(graph,kind,alpha,search,seed,k);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;for(const kind of ['economic','housing'])for(const alpha of [0,.5,1])await check(g,kind,alpha,'multi',42,k,true);}
const sha=b=>createHash('sha256').update(b).digest('hex'),summary={wasm_sha256:sha(wasm),native_state_reference_sha256:sha(await fs.readFile(exe)),cases:cases.length,exact_assignment_matches:cases.filter(c=>!c.error).length,matching_failures:cases.filter(c=>c.error).length,balance_misses:cases.filter(c=>c.balanced===false).length,disconnected_plans:cases.filter(c=>c.contiguous===false).length,scope:'State fixes alpha .5; other alphas independently test actual CLI weighter API. Synthetic complete observations only; no native file acquisition/missing-data fallback or full capacities.',cases_evidence:cases};
await fs.writeFile(root+'/evidence.json',JSON.stringify(summary,null,2));console.log(`Character State/native weighter: ${summary.exact_assignment_matches} exact assignments, ${summary.matching_failures} matching failures, ${summary.balance_misses} final misses, ${summary.disconnected_plans} disconnected plans; CLI alpha .5 and native API endpoints distinguished.`);
