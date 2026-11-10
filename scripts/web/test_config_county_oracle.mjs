import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/config-county-oracle';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/config-run-wasm/case-0.json','utf8')).input.graph;
const graph={...base,geoids:base.geoids.map((id,i)=>(i%6<3?'19001':'19003')+id.slice(5))};
const incompatible={graph,yaml_text:'name: wrong_backend\nalgorithm:\n  structure: standard-bisect\n  engine: c-ffi\n',options:{districts:4}};
await fs.writeFile(root+'/wrong-backend.json',JSON.stringify(incompatible));const refused=spawnSync(path.resolve('target/debug/examples/config_seed_reference.exe'),[path.resolve(root+'/wrong-backend.json')],{encoding:'utf8'});assert.notEqual(refused.status,0);assert(refused.stderr.includes('explicit metis-core declaration'));
const evidence=[];let hostMatches=0;const cliOnly=process.argv.includes('--cli-only');
async function check(graph,alpha,structure,search,seed,k=4,real=false,weights='county'){
 const yaml_text=`name: county_oracle\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  weights: ${weights}\n  alpha_county: ${alpha}\n  search: ${search}\n  seeds: 4\n  seed: ${seed}\n  balance_tolerance: 25\n`;
 const request={operation:'run-algorithm-config',input:{graph,yaml_text,source_name:'county.yml',profile:'open',chamber:'congressional',districts:k}},actual=engine.execute(request),file=path.resolve(root,`case-${evidence.length}.json`);
 await fs.writeFile(file,JSON.stringify({graph,yaml_text,options:{districts:k}}));const native=JSON.parse(execFileSync(path.resolve('target/debug/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000}));
 assert.deepEqual(actual.result.assignments,native.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,native.balanced);
 const effective=weights==='county'&&alpha>1e-10?alpha:0;
 assert.equal(actual.engine_request.options.alpha_county,effective);assert.equal(actual.engine_request.options.weights,effective?'county':weights==='unweighted'?'unweighted':'geographic');
 assert.equal(actual.configuration.document.algorithm.alpha_county,alpha);assert.equal(actual.configuration.yaml_text,yaml_text);
 const close=(a,b)=>assert(Math.abs(a-b)<=Math.max(1,Math.abs(b))*1e-12,`${a} != ${b}`);close(actual.result.metrics.weighted_boundary,native.weighted_cut);close(actual.result.metrics.graph_boundary_m,native.physical_cut);
 let weighted=0,physical=0;
 for(const [u,v,length]of graph.edges)if(actual.result.assignments[graph.geoids[u]]!==actual.result.assignments[graph.geoids[v]]){physical+=length;weighted+=weights==='unweighted'?1:length*(effective&&graph.geoids[u].slice(0,5)===graph.geoids[v].slice(0,5)?1+effective:1);}
 close(actual.result.metrics.weighted_boundary,weighted);close(actual.result.metrics.graph_boundary_m,physical);
 await fs.writeFile(file+'.tool.json',JSON.stringify(request));if(!cliOnly){assert.deepEqual(actual,JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file+'.tool.json'],{encoding:'utf8',maxBuffer:32*1024*1024})));hostMatches++;}
 evidence.push({state:graph.state,units:graph.geoids.length,alpha,weights,structure,search,seed,k,real_exported_graph:real,effective_alpha:effective,within_tolerance:actual.result.metrics.within_requested_tolerance,contiguous:actual.result.metrics.contiguous});
}
for(const alpha of [-2,0,1e-11,1e-10,1.1e-10,.5,2,100])for(const [structure,search]of [['standard-bisect','single'],['standard-bisect','multi'],['nway','single'],['ratio-optimal','multi']])for(const seed of ['42','18446744073709551615'])await check(graph,alpha,structure,search,seed);
for(const weights of ['geographic','unweighted'])await check(graph,2,'standard-bisect','single','42',4,false,weights);
const catalogRoot=process.argv.slice(2).find(arg=>!arg.startsWith('--'))||'dist/wasm-national',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const state of ['RI','IA']){
 const real=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[state+':2020'].graph_ref),'utf8'));
 assert.equal(real.geoids.length,state==='RI'?250:896);const k=catalog.catalog.states.find(s=>s.code===state).years.find(y=>y.year==='2020').districts.congressional;
 for(const alpha of [0,2])await check(real,alpha,'standard-bisect','single','42',k,true);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence.length,actual_cli_yaml_county_dispatch_assignment_matches:evidence.length,whole_host_wasm_matches:hostMatches,native_weighted_and_physical_cut_checks:true,independent_cut_sums:true,scope:'Native YAML conversion, native SubdivisionWeighter county-only composition and matching split dispatch on explicit prepared graph inputs. Original State acquisition, other subdivision levels and arbitrary capacity are not established.',cases_evidence:evidence},null,2));
console.log(`Configured county weighting: ${evidence.length} native CLI assignment and ${hostMatches} host/WASM matches, native/independent cut sums, inactive threshold/negative declarations, two exported real states and original source retention passed.`);
