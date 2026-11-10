import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/config-one-district-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),nativeExe=path.resolve('target/release/examples/config_seed_reference.exe');
const n=12,graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array.from({length:n},(_,i)=>100+i),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i*.001,41])};
const structures=['standard-bisect','nway','ratio-optimal','ratio-optimal-area','ratio-optimal-vra','prime-factor','compact-polsby','moving-knife','spectral','regionalization','flow-construction'],backends=[null,'c-ffi','gpmetis','metis-core'],cases=[];let refusals=0;
const yaml=(structure,backend,seed,weights)=>`name: one_district\nalgorithm:\n  structure: ${structure}\n${backend===null?'':`  engine: ${backend}\n`}  weights: ${weights}\n  search: single\n  balance_tolerance: 5\n${seed===null?'':`  seed: ${seed}\n`}`;
async function check(g,structure,backend,seed,weights='geographic',real=false){
 const yaml_text=yaml(structure,backend,seed,weights),input={graph:g,yaml_text,source_name:'one.yml',profile:'open',chamber:'congressional',districts:1};
 if(structure==='ratio-optimal-vra')input.demographics={schema_version:'bisect-demographic-fractions-v1',state:g.state,year:g.year,basis:'total-population',source_label:'SYNTHETIC regression fractions',minority_fractions:Object.fromEntries(g.geoids.map(id=>[id,.2]))};
 const actual=engine.execute({operation:'run-algorithm-config',input});
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,yaml_text,options:{districts:1},...(input.demographics?{demographics:input.demographics}:{})}));
 const native=spawnSync(nativeExe,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
 assert.deepEqual(actual.result.assignments,reference.assignments);assert(Object.values(reference.assignments).every(d=>d===1));assert.equal(actual.result.metrics.within_requested_tolerance,reference.balanced);
 assert.equal(actual.result.metrics.population,g.population.reduce((a,b)=>a+b,0));assert.equal(actual.result.metrics.max_deviation_percent,0);assert.equal(actual.result.metrics.weighted_boundary,0);assert.equal(actual.result.metrics.graph_boundary_m,0);assert.equal(actual.result.metrics.root_split,null);if(structure==='spectral')assert.deepEqual(actual.result.metrics.structure_evidence,{schema_version:'bisect-spectral-run-summary-v1',method:'spectral',max_iters:200,tolerance:.05,k:1,edge_cut:0,nodes:[]});else if(['capacity-clustering','regionalization','flow-construction'].includes(structure))assert(actual.result.metrics.structure_evidence);else assert.equal(actual.result.metrics.structure_evidence,null);
 const c=actual.configuration;assert.equal(c.partitioning_shortcut,!['capacity-clustering','regionalization','flow-construction'].includes(structure));assert.equal(c.metis_engine_used,false);assert.equal(c.metis_engine_required_by_method,!['moving-knife','spectral','regionalization','flow-construction'].includes(structure));assert.equal(c.browser_engine_available,true);assert.equal(c.browser_execution_backend,['capacity-clustering','regionalization','flow-construction'].includes(structure)?'shared-native-rust-no-metis':'single-district-no-partitioning');assert.equal(c.declared_engine,backend??'c-ffi');assert.equal(c.yaml_text,yaml_text);
 const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
 const files={configSource:source('one.yml',yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(g)),...(input.demographics?{demographics:source('demographics.json',JSON.stringify(input.demographics))}:{})},configRunSettings:{chamber:'congressional',districts:1}};
 const project=createProject({name:'One district '+g.state,files,operation:'run-algorithm-config',constraints:[],result:actual,lastOperation:'run-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)).result,actual);
 if(!real&&structure==='standard-bisect'&&backend===null&&seed==='18446744073709551615'&&weights==='geographic')await fs.writeFile(root+'/browser-fixture.bisect',JSON.stringify(project));
 cases.push({state:g.state,units:g.geoids.length,structure,backend,seed,weights,real,contiguous:actual.result.metrics.contiguous,original_declaration_preserved:true});
}
for(const structure of structures)for(const backend of backends)for(const seed of [null,'42','18446744073709551615'])for(const weights of ['geographic','unweighted'])await check(graph,structure,backend,seed,weights);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['VT','WY']){
 const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8'));
 for(const structure of ['standard-bisect','nway','ratio-optimal','compact-polsby'])for(const backend of backends)await check(g,structure,backend,'18446744073709551615','geographic',true);
}
const disconnected={...graph,adjacency:graph.adjacency.map((ns,u)=>ns.filter(v=>!(u===5&&v===6||u===6&&v===5))),edges:graph.edges.filter(([u,v])=>!(u===5&&v===6))};
await check(disconnected,'standard-bisect',null,'42');assert.equal(cases.at(-1).contiguous,false);
const baseInput={graph,yaml_text:yaml('standard-bisect',null,'42','geographic'),source_name:'one.yml',profile:'open',chamber:'congressional',districts:1};
async function refuse(input){
 const request={operation:'run-algorithm-config',input};let error;
 try{engine.execute(request);}catch(e){error=e.message;}assert(error);
 const file=path.resolve(root,`refused-${refusals}.json`);await fs.writeFile(file,JSON.stringify(request));
 const native=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:1024*1024});assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr+' / '+error);refusals++;
}
// Isolate invalid district bounds with the common Rust backend: availability
// is a separate preflight, and the native host has no browser C bridge.
for(const edit of [v=>v.graph.population=[],v=>v.graph.population[0]=-1,v=>v.graph.adjacency[0].push(999),v=>{v.districts=0;v.yaml_text=v.yaml_text.replace('algorithm:\n','algorithm:\n  engine: metis-core\n');},v=>v.yaml_text=v.yaml_text.replace('search: single','search: invalid'),v=>v.yaml_text+='  engine: invalid\n']){
 const input=structuredClone(baseInput);edit(input);await refuse(input);
}
let unloadedRuntimeRefusals=0;
for(const backend of [null,'c-ffi','gpmetis']){
 const input={...baseInput,districts:2,yaml_text:yaml('standard-bisect',backend,'42','geographic')};
 if(engine.supportsCMetis&&backend!=='gpmetis'){
  const inspected=engine.execute({operation:'validate-algorithm-config',yaml_text:input.yaml_text,source_name:input.source_name,profile:input.profile});
  assert.equal(inspected.browser_engine_available,true);
  assert.throws(()=>engine.execute({operation:'run-algorithm-config',input}));unloadedRuntimeRefusals++;
 }else await refuse(input);
} 
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_cli_reference_sha256:sha(await fs.readFile(nativeExe)),cases:cases.length,exact_cli_assignment_matches:cases.length,exact_project_roundtrips:cases.length,disconnected_graph_reported:true,invalid_input_backend_refusals:refusals,unloaded_runtime_refusals:unloadedRuntimeRefusals,scope:'One-district shortcut on identical supplied graphs, with existing graph/options/data requirements retained. Synthetic VRA fractions are explicit; full filesystem acquisition and legal certification remain unproven.',cases_evidence:cases},null,2));
console.log(`One-district configuration: ${cases.length} exact CLI assignments and inert projects, preserved declarations, disconnected-graph reporting and ${refusals} input/backend refusals.`);
