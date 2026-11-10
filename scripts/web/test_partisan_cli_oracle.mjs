import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/partisan-cli-oracle';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/config-run-wasm/case-0.json','utf8')).input.graph,evidence=[];
for(const profile of ['uniform','mixed','stripe','zero','all'])for(const [dem_threshold,rep_threshold]of [[.55,.45],[.5,.5],[1,0]])for(const structure of ['standard-bisect'])for(const search of ['single','multi'])for(const seed of [42,'18446744073709551615']){
 const shares=graph.geoids.map((_,i)=>profile==='zero'?0:profile==='all'?1:profile==='uniform'?.5:profile==='mixed'?[.8,.5,.2,.5][i%4]:i%6<3?.8:.2),partisan={schema_version:'bisect-partisan-shares-v1',state:graph.state,year:graph.year,source_label:'Synthetic shares for native dispatch parity',dem_shares:Object.fromEntries(graph.geoids.map((id,i)=>[id,shares[i]]))};
 const options={structure,weights:'partisan',search,districts:4,seed,seeds:3,steps:20,percentile:.5,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:100,dem_threshold,rep_threshold};
 const request={graph,options,partisan},actual=engine.execute(request),file=path.resolve(root,`case-${evidence.length}.json`);
 const yaml_text=`name: partisan_reference\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  weights: geographic\n  search: ${search}\n  seeds: 3\n  seed: ${seed}\n  balance_tolerance: 25\n`;
 await fs.writeFile(file,JSON.stringify({graph,partisan,yaml_text,options:{districts:4,iterations:100,dem_threshold,rep_threshold}}));
 const native=JSON.parse(execFileSync(path.resolve('target/debug/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
 assert.deepEqual(actual.assignments,native.assignments);assert.equal(actual.metrics.within_requested_tolerance,native.balanced);
 const near=(a,b)=>assert(Math.abs(a-b)<=Math.max(1,Math.abs(b))*1e-12,`${a} != ${b}`);near(actual.metrics.weighted_boundary,native.weighted_cut);near(actual.metrics.graph_boundary_m,native.physical_cut);
 const count=shares.filter(v=>v>=dem_threshold||v<=rep_threshold).length,alpha=Math.max(3,10*(1-.7*count/shares.length));let expectedCut=0;
 for(const [u,v]of graph.edges)if(actual.assignments[graph.geoids[u]]!==actual.assignments[graph.geoids[v]])expectedCut+=(shares[u]>=dem_threshold&&shares[v]>=dem_threshold)||(shares[u]<=rep_threshold&&shares[v]<=rep_threshold)?alpha:1;
 near(actual.metrics.weighted_boundary,expectedCut);assert.equal(actual.metrics.weighting_evidence.dispatch_representation,'native-sparse-boost-map');
 evidence.push({profile,structure,search,seed,dem_threshold,rep_threshold,assignment_match:true,balance_match:true});
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({actual_native_cli_assignment_and_cut_cases:evidence.length,shares_are_synthetic:true,cases:evidence},null,2));console.log(`Partisan override: ${evidence.length} actual portable native CLI assignment, balance and cut matches.`);
