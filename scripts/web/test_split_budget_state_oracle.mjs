import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/split-budget-state-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),exe='target/release/examples/split_budget_state_reference.exe';
const sha=b=>createHash('sha256').update(b).digest('hex'),records=[];
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array.from({length:n},(_,i)=>100+i%7),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array.from({length:n},(_,i)=>100+i*5),exterior_perimeters:Array.from({length:n},(_,i)=>10+i%4),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
async function check(g,structure,weights,flags,count,seed,k=3,route='preset',real=false){
 const mode=structure==='compact-polsby'?'compact-bisect':'geosection';
 const state_args=['--state',g.state,'--year',g.year,'--districts',String(k),'--balance-tolerance','25','--niter','10','--metis-engine','metis-core',...(route==='preset'?['--partition-mode',mode]:['--partition-mode','edge-weighted','--structure',structure]),'--weights-override',weights,...(seed===null?[]:['--seed',seed]),...flags];
 const input={graph:g,state_args},file=root+'/case-'+records.length+'.json';await fs.writeFile(file,JSON.stringify(input));const native=spawnSync(exe,[file],{encoding:'utf8',timeout:120000,maxBuffer:16*1024*1024});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.equal(reference.seeds_per_split,count);
 const effectiveSeed=reference.compositor_seed;
 const options={structure,weights,search:count===1?'single':'multi',districts:k,seed:effectiveSeed,seeds:count,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10,...(structure==='compact-polsby'?{compact_epsilon:0.05}:{})};
 const actual=engine.execute({graph:g,options});assert.deepEqual(actual.assignments,reference.assignments,JSON.stringify({structure,weights,flags,k,route}));assert.equal(actual.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.declared_seed,seed);
 if(route==='preset')assert.equal(effectiveSeed,null);else assert.equal(effectiveSeed,seed);
 if(structure==='compact-polsby')assert.equal(actual.metrics.structure_evidence.seeds_per_split,count);
 records.push({state:g.state,units:g.geoids.length,real,graph_sha256:sha(JSON.stringify(g)),structure,weights,flags,seeds_per_split:count,seed,compositor_seed:effectiveSeed,k,route,balanced:reference.balanced});
}
for(const structure of ['compact-polsby','ratio-optimal'])for(const weights of ['geographic','unweighted']){
 const flag=structure==='compact-polsby'?'--compact-seeds':'--geosection-seeds';
 for(const budget of [null,0,1,3])await check(graph,structure,weights,budget===null?[]:[flag,String(budget)],budget===null?1:Math.max(1,budget),'18446744073709551615');
 await check(graph,structure,weights,['--seeds','7'],1,'42'); // ignored without --search
 await check(graph,structure,weights,['--search','multi'],50,'42');
 await check(graph,structure,weights,['--search','multi','--seeds','3',flag,'7'],3,'42');
 await check(graph,structure,weights,['--search','single',flag,'7'],1,'42');
 await check(graph,structure,weights,['--search','multi','--compact-seeds','51','--geosection-seeds','52'],52,'42');
 await check(graph,structure,weights,[],50,'42',3,'override');
 await check(graph,structure,weights,['--search','single'],1,'18446744073709551615',3,'override');
 await check(graph,structure,weights,['--search','multi','--seeds','3'],3,null,5,'override');
}
const catalog=JSON.parse(await fs.readFile('dist/wasm-lab-build/catalog.json','utf8'));
for(const state of ['RI','IA']){const g=JSON.parse(await fs.readFile(path.join('dist/wasm-lab-build',catalog.graphs[state+':2020'].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===state).years.find(y=>y.year==='2020').districts.congressional;
 for(const structure of ['compact-polsby','ratio-optimal'])for(const budget of [null,3])await check(g,structure,'geographic',budget===null?[]:[structure==='compact-polsby'?'--compact-seeds':'--geosection-seeds',String(budget)],budget??1,'18446744073709551615',k,'preset',true);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_reference_sha256:sha(await fs.readFile(exe)),actual_state_clap_config_dispatch_matches:records.length,balance_misses:records.filter(r=>!r.balanced).length,geometry_scope:'Identical supplied native graph geometry vectors; does not prove TIGER acquisition, all-state capacity, native retries or plan certification.',records},null,2)+'\n');console.log(`State split budgets: ${records.length} actual State clap/config/kernel assignment and balance matches; preset defaults/zero, override precedence, seed policies and RI/IA native exported graphs.`);
