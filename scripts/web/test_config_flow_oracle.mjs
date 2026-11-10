import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/config-flow-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm);
const executable=path.resolve('target/release/examples/config_seed_reference.exe');
const n=12,line={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`${i<6?'44001':'44003'}${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i*.001,41])};
const searches={single:'',multi:'  seeds: 7\n',convergence:'  convergence_threshold: 3\n','short-burst':'  burst_length: 3\n  n_bursts: 2\n  percentile: 0.5\n'};
const cases=[];let successes=0,failures=0,keyRefusals=0;
async function check(graph,search,seed,weights,k,real=false){
  const yaml_text=`name: flow_oracle\nalgorithm:\n  structure: flow-construction\n  engine: metis-core\n  weights: ${weights}\n${weights==='county'?'  alpha_county: 2\n':''}  search: ${search}\n${searches[search]}  balance_tolerance: 5\n${seed===null?'':`  seed: ${seed}\n`}`;
  const input={graph,yaml_text,chamber:'congressional',districts:k,source_name:'flow.yml',profile:'open'};
  const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph,yaml_text,options:{districts:k}}));
  const native=spawnSync(executable,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
  let actual,error;try{actual=engine.execute({operation:'run-algorithm-config',input});}catch(e){error=e.message;}
  if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr+' / '+error);failures++;cases.push({state:graph.state,units:graph.geoids.length,search,seed,weights,k,real,error});return;}
  assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);
  assert.deepEqual(actual.result.assignments,reference.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.balanced,true);
  // An empty native Iterator<f64>::sum can serialize -0; zero cut has no direction.
  const cut=value=>value===0?0:value;
  assert.equal(cut(actual.result.metrics.weighted_boundary),cut(reference.weighted_cut));assert.equal(cut(actual.result.metrics.graph_boundary_m),cut(reference.physical_cut));assert.equal(reference.declared_seed,seed);
  assert.equal(actual.engine_request.options.search,'single');assert.equal(actual.configuration.search_interpretation,'native-structure-ignores-search');
  if(k>1)assert.equal(actual.result.metrics.structure_evidence.repair_method,'bfs');
  assert.equal(actual.configuration.seed,seed);assert.equal(actual.configuration.yaml_text,yaml_text);
  // An independent traversal verifies connectivity, not just the native flag.
  const labels=graph.geoids.map(id=>reference.assignments[id]);
  for(let d=1;d<=k;d++){
    const members=labels.map((label,i)=>label===d?i:null).filter(i=>i!==null),seen=new Set(),stack=members.length?[members[0]]:[];
    while(stack.length){const u=stack.pop();if(seen.has(u))continue;seen.add(u);for(const v of graph.adjacency[u])if(labels[v]===d&&!seen.has(v))stack.push(v);}
    assert.equal(seen.size,members.length);assert(members.length);
  }
  assert.equal(actual.result.metrics.contiguous,true);
  const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
  const project=createProject({name:'Configured flow oracle',files:{configSource:source('flow.yml',yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(graph))},configRunSettings:{chamber:'congressional',districts:k}},operation:'run-algorithm-config',constraints:[],result:actual,lastOperation:'run-algorithm-config'});
  assert.deepEqual(parseProject(JSON.stringify(project)).result,actual);
  if(!real&&search==='short-burst'&&seed==='18446744073709551615'&&weights==='county'&&k===2)await fs.writeFile(root+'/browser-fixture.bisect',JSON.stringify(project));
  successes++;cases.push({state:graph.state,units:graph.geoids.length,search,seed,weights,k,real,assignment_match:true,balanced:true,contiguous:true});
}
for(const graph of [line,{...line,population:[0,...Array(n-2).fill(100),0]},{...line,population:[100000,...Array(n-1).fill(100)]}])for(const search of Object.keys(searches))for(const seed of [null,'42','18446744073709551615'])for(const weights of ['unweighted','geographic','county'])await check(graph,search,seed,weights,2);
for(const k of [1,3,6])await check(line,'single','42','geographic',k);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){
  const graph=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8'));
  const k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
  for(const weights of ['unweighted','geographic','county'])await check(graph,'multi','18446744073709551615',weights,k,true);
}
for(const repair of ['bfs','none']){
  const yaml_text=`name: unsupported_flow_repair\nalgorithm:\n  structure: flow-construction\n  engine: metis-core\n  flow_repair: ${repair}\n`;
  assert.throws(()=>engine.execute({operation:'run-algorithm-config',input:{graph:line,yaml_text,chamber:'congressional',districts:2,source_name:'flow.yml',profile:'open'}}),/unknown field `flow_repair`/);
  const file=path.resolve(root,`refusal-${keyRefusals}.json`);await fs.writeFile(file,JSON.stringify({graph:line,yaml_text,options:{districts:2}}));const native=spawnSync(executable,[file],{encoding:'utf8'});assert.notEqual(native.status,0);assert.match(native.stderr,/unknown field.*flow_repair/);keyRefusals++;
}
assert(successes&&failures);const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_cli_reference_sha256:sha(await fs.readFile(executable)),cases:cases.length,successes,matching_failures:failures,matching_unknown_repair_key_refusals:keyRefusals,exact_project_roundtrips:successes,independent_connectivity_checks:successes,scope:'Actual native CLI FlowConfig::new dispatch on identical supplied populations/adjacency. Native YAML has fixed BFS repair and ignores seed/search/weight choices for construction. Complete filesystem acquisition, alternate population basis, arbitrary capacity and certification remain unproven.',cases_evidence:cases},null,2));
console.log(`Configured flow: ${successes} actual CLI assignment/balance/cut matches, ${failures} matching generation failures, ${keyRefusals} repair-key refusals; inert projects and independent connectivity checks passed.`);
