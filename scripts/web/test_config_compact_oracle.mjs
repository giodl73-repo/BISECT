import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/config-compact-oracle';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const side=6,n=side*side,adjacency=Array.from({length:n},(_,i)=>[i%side?i-1:-1,i%side<side-1?i+1:-1,i>=side?i-side:-1,i<n-side?i+side:-1].filter(v=>v>=0));
const graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency,population:Array(n).fill(100),edges:adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])),areas:Array.from({length:n},(_,i)=>100+i*5),exterior_perimeters:Array.from({length:n},(_,i)=>10+i%4),centroids:Array.from({length:n},(_,i)=>[-71+(i%side)*.001,41+Math.floor(i/side)*.001])};
const searches={single:{yaml:'',count:1},multi:{yaml:'  seeds: 3\n',count:3},convergence:{yaml:'  convergence_threshold: 2\n',count:2},'short-burst':{yaml:'  burst_length: 3\n  n_bursts: 2\n  percentile: 0.5\n',count:2}};
const evidence=[];let rejected=0;
async function check(g,epsilon,search,seed,weights,k=2,real=false){
  const yaml_text=`name: compact_oracle\nalgorithm:\n  structure: compact-polsby\n  engine: metis-core\n  weights: ${weights}\n  search: ${search}\n${searches[search].yaml}  balance_tolerance: 25\n${epsilon===null?'':`  epsilon: ${epsilon}\n`}${seed===null?'':`  seed: ${seed}\n`}`;
  const input={graph:g,yaml_text,chamber:'congressional',districts:k,source_name:'compact.yml',profile:'open'};
  if(epsilon!==null){
    assert.throws(()=>engine.execute({operation:'run-algorithm-config',input}),/unknown field `epsilon`/);
    const file=path.resolve(root,`rejected-${rejected}.json`);await fs.writeFile(file,JSON.stringify({graph:g,yaml_text,options:{districts:k}}));
    const native=spawnSync(path.resolve('target/release/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:1024*1024,timeout:120000});
    assert.notEqual(native.status,0);assert.match(native.stderr,/unknown field.*epsilon/);rejected++;return;
  }
  const actual=engine.execute({operation:'run-algorithm-config',input});
  const file=path.resolve(root,`case-${evidence.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,yaml_text,options:{districts:k}}));
  const native=JSON.parse(execFileSync(path.resolve('target/release/examples/config_seed_reference.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000}));
  assert.deepEqual(actual.result.assignments,native.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,native.balanced);
  assert.equal(native.declared_seed,seed);
  assert.equal(actual.result.metrics.weighted_boundary,native.weighted_cut);assert.equal(actual.result.metrics.graph_boundary_m,native.physical_cut);
  const labels=g.geoids.map(id=>native.assignments[id]);let contiguous=true;
  for(let d=1;d<=k;d++){
    const members=labels.map((label,i)=>label===d?i:null).filter(i=>i!==null),seen=new Set(),stack=members.length?[members[0]]:[];
    while(stack.length){const u=stack.pop();if(seen.has(u))continue;seen.add(u);for(const v of g.adjacency[u])if(labels[v]===d&&!seen.has(v))stack.push(v);}
    if(!members.length||seen.size!==members.length)contiguous=false;
  }
  assert.equal(actual.result.metrics.contiguous,contiguous);
  // Native YAML rejects an epsilon field: its CompactBisect slack is fixed at .01.
  // The direct engine/lab API separately exposes compact_epsilon (default .05).
  assert.equal(actual.engine_request.options.compact_epsilon,0.01);
  assert.equal(actual.configuration.document.algorithm.epsilon,undefined);
  assert.equal(actual.engine_request.options.seeds,searches[search].count);
  assert.equal(actual.engine_request.options.search,searches[search].count===1?'single':'multi');
  assert.equal(actual.result.metrics.structure_evidence.seed_policy,'fixed-1-through-budget');
  assert.equal(actual.result.metrics.structure_evidence.seeds_per_split,searches[search].count);
  assert.equal(actual.configuration.yaml_text,yaml_text);assert.equal(actual.configuration.seed,seed);
  const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
  const project=createProject({name:'Configured compact oracle',files:{configSource:source('compact.yml',yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(g))},configRunSettings:{chamber:'congressional',districts:k}},operation:'run-algorithm-config',constraints:[],result:actual,lastOperation:'run-algorithm-config'});
  assert.deepEqual(parseProject(JSON.stringify(project)).result,actual);
  evidence.push({state:g.state,units:g.geoids.length,graph_sha256:createHash('sha256').update(JSON.stringify(g)).digest('hex'),unsupported_yaml_epsilon:epsilon,effective_epsilon:0.01,declared_search:search,effective_seed_budget:searches[search].count,declared_seed:seed,weights,districts:k,real_exported_graph:real,within_tolerance:native.balanced,contiguous});
  if(search==='short-burst'&&seed==='18446744073709551615'&&weights==='geographic'&&!real&&k===2)await fs.writeFile(root+'/browser-fixture.bisect',JSON.stringify(project));
}
for(const epsilon of [null,0,0.05,0.2,1])for(const search of Object.keys(searches))for(const seed of [null,'42','18446744073709551615'])for(const weights of ['geographic','unweighted'])await check(graph,epsilon,search,seed,weights);
for(const k of [3,4,5])await check(graph,null,'multi','42','geographic',k);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){
  const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8'));
  const k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
  for(const search of ['single','multi'])await check(g,null,search,'18446744073709551615','geographic',k,true);
}
const repeatFile=path.resolve(root,'case-6.json');let repeated;
for(let i=0;i<12;i++){
  const current=JSON.parse(execFileSync(path.resolve('target/release/examples/config_seed_reference.exe'),[repeatFile],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000}));
  if(repeated)assert.deepEqual(current,repeated);repeated=current;
}
const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),native_cli_reference_sha256:sha(await fs.readFile('target/release/examples/config_seed_reference.exe')),geometry_source_sha256:sha(await fs.readFile('crates/bisect-data/src/adjacency.rs')),cases:evidence.length,actual_cli_yaml_dispatch_assignment_matches:evidence.length,matching_unknown_epsilon_rejections:rejected,repeated_native_process_results:12,independent_connectivity_checks:evidence.length,exact_project_roundtrips:evidence.length,balance_misses:evidence.filter(e=>!e.within_tolerance).length,disconnected_plans:evidence.filter(e=>!e.contiguous).length,scope:'Native CLI YAML dispatch on identical supplied geometry/graphs; full filesystem geometry acquisition, arbitrary capacities, every CLI flag and legal acceptance remain unproven.',cases_evidence:evidence},null,2));
console.log(`Compact configuration: ${evidence.length} actual native CLI assignments/balance/cut comparisons, seed budgets and inert project roundtrips; ${rejected} matching epsilon-key refusals; ${evidence.filter(e=>!e.within_tolerance).length} balance misses, ${evidence.filter(e=>!e.contiguous).length} disconnected plans.`);
