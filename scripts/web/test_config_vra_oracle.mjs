import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/config-vra-oracle';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/config-run-wasm/case-0.json','utf8')).input.graph,evidence=[];const cliOnly=process.argv.includes('--cli-only');let hostMatches=0;
const native=(file)=>JSON.parse(execFileSync(path.resolve('target/debug/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000,stdio:['ignore','pipe','pipe']}));
async function check(graph,profile,structure,weights,search,w_vra,seed,k=4,real=false){
 const fractions=graph.geoids.map((_,i)=>profile==='zero'?0:profile==='uniform'?.4:i%4===0?.4:i%4===1?.399999999999:i%4===2?.9:.1);
 const demographics={schema_version:'bisect-demographic-fractions-v1',state:graph.state,year:graph.year,basis:'total-population',source_label:'Explicit synthetic fractions on supplied graph',minority_fractions:Object.fromEntries(graph.geoids.map((id,i)=>[id,fractions[i]]))};
 const yaml_text=`name: vra_oracle\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  weights: ${weights}\n  search: ${search}\n  seeds: 3\n  convergence_threshold: 3\n  seed: ${seed}\n  balance_tolerance: 25\n  w_vra: ${w_vra}\n`;
 const request={operation:'run-algorithm-config',input:{graph,yaml_text,source_name:'vra.yml',profile:'open',chamber:'congressional',districts:k,demographics}},actual=engine.execute(request),file=path.resolve(root,`case-${evidence.length}.json`);
 await fs.writeFile(file,JSON.stringify({graph,yaml_text,demographics,options:{districts:k}}));const reference=native(file);assert.deepEqual(actual.result.assignments,reference.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,reference.balanced);
 const near=(a,b)=>assert(Math.abs(a-b)<=Math.max(1,Math.abs(b))*1e-12,`${a} != ${b}`);near(actual.result.metrics.weighted_boundary,reference.weighted_cut);near(actual.result.metrics.graph_boundary_m,reference.physical_cut);
 const high=fractions.filter(f=>f>=.4).length,alpha=Math.max(3,10*(1-.7*high/fractions.length));let cut=0,physical=0;
 for(const [u,v,length]of graph.edges)if(actual.result.assignments[graph.geoids[u]]!==actual.result.assignments[graph.geoids[v]]){physical+=length;cut+=weights==='vra-aligned'?(fractions[u]>=.4&&fractions[v]>=.4?alpha:1):weights==='unweighted'?1:length;}
 near(actual.result.metrics.weighted_boundary,cut);near(actual.result.metrics.graph_boundary_m,physical);
 assert.equal(actual.configuration.yaml_text,yaml_text);assert.equal(actual.configuration.document.algorithm.w_vra,w_vra);
 if(structure==='ratio-optimal-vra'){assert.equal(actual.engine_request.options.w_vra,w_vra);assert.equal(actual.result.metrics.structure_evidence.w_vra,w_vra);}
 await fs.writeFile(file+'.tool.json',JSON.stringify(request));if(!cliOnly){assert.deepEqual(actual,JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file+'.tool.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']})));hostMatches++;}
 evidence.push({state:graph.state,units:graph.geoids.length,profile,demographics_scope:'synthetic explicitly supplied total-population fractions',graph_sha256:createHash('sha256').update(JSON.stringify(graph)).digest('hex'),structure,weights,search,w_vra,seed,k,real_exported_graph:real,within_tolerance:actual.result.metrics.within_requested_tolerance,contiguous:actual.result.metrics.contiguous});
}
for(const profile of ['zero','uniform','mixed'])for(const [structure,search]of [['standard-bisect','single'],['standard-bisect','multi'],['nway','single']])for(const seed of ['42','18446744073709551615'])await check(base,profile,structure,'vra-aligned',search,.4,seed);
for(const weights of ['geographic','unweighted','vra-aligned'])for(const w_vra of [0,.4,1])for(const search of ['single','multi','convergence'])await check(base,'mixed','ratio-optimal-vra',weights,search,w_vra,'42');
const catalogRoot=process.argv.slice(2).find(arg=>!arg.startsWith('--'))||'dist/wasm-national',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const state of ['RI','IA']){
 const graph=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[state+':2020'].graph_ref),'utf8'));assert.equal(graph.geoids.length,state==='RI'?250:896);
 const k=catalog.catalog.states.find(s=>s.code===state).years.find(y=>y.year==='2020').districts.congressional;
 await check(graph,'mixed','standard-bisect','vra-aligned','single',.4,'42',k,true);
 await check(graph,'mixed','ratio-optimal-vra','vra-aligned','multi',.4,'42',k,true);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence.length,actual_cli_dispatch_assignment_matches:evidence.length,whole_host_wasm_matches:hostMatches,native_and_independent_cut_sums:true,demographics_are_synthetic_even_on_real_graphs:true,scope:'Native YAML conversion, actual MinorityOverrideWeighter and VRASection dispatch on identical supplied graphs/fractions. This verifies methods, not authentic demographic data, source acquisition, legal compliance or arbitrary capacity.',cases_evidence:evidence},null,2));
console.log(`Configured VRA: ${evidence.length} native CLI assignments and ${hostMatches} whole host/WASM matches, actual native minority weighting, alignment weights, native/independent cut sums and explicit synthetic demographic provenance passed.`);
