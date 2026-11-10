import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';

const root='target/config-no-metis-oracle';await fs.mkdir(root,{recursive:true});
const wasm=await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),executable=path.resolve('target/release/examples/config_seed_reference.exe');
const n=12,graph={schema_version:1,state:'RI',year:'2020',geoids:Array.from({length:n},(_,i)=>`44001${String(i).padStart(6,'0')}`),adjacency:Array.from({length:n},(_,i)=>[i-1,i+1].filter(v=>v>=0&&v<n)),population:Array(n).fill(100),edges:Array.from({length:n-1},(_,i)=>[i,i+1,100+i]),areas:Array(n).fill(100),exterior_perimeters:Array(n).fill(10),centroids:Array.from({length:n},(_,i)=>[-71+i*.001,41])};
const structures=['moving-knife','spectral','regionalization','flow-construction'],backends=[null,'c-ffi','gpmetis','metis-core'],cases=[];let successes=0,failures=0,guardRefusals=0,unloadedRuntimeRefusals=0;
const yaml=(structure,backend,seed)=>`name: no_metis_oracle\nalgorithm:\n  structure: ${structure}\n${backend===null?'':`  engine: ${backend}\n`}  weights: geographic\n  search: single\n  balance_tolerance: 25\n${seed===null?'':`  seed: ${seed}\n`}${structure==='moving-knife'?'  mka_orientations: 36\n':''}`;
async function check(g,structure,backend,seed,k=2,real=false){
 const yaml_text=yaml(structure,backend,seed),input={graph:g,yaml_text,chamber:'congressional',districts:k,source_name:'no-metis.yml',profile:'open'};
 const validated=engine.execute({operation:'validate-algorithm-config',yaml_text,source_name:input.source_name,profile:'open'});
 assert.equal(validated.metis_engine_used,false);assert.equal(validated.browser_engine_available,true);assert.equal(validated.browser_execution_backend,'shared-native-rust-no-metis');assert.equal(validated.declared_engine,backend??'c-ffi');assert.equal(validated.yaml_text,yaml_text);
 const file=path.resolve(root,`case-${cases.length}.json`);await fs.writeFile(file,JSON.stringify({graph:g,yaml_text,options:{districts:k}}));const native=spawnSync(executable,[file],{encoding:'utf8',maxBuffer:32*1024*1024,timeout:120000});assert.equal(native.error,undefined);
 let actual,error;try{actual=engine.execute({operation:'run-algorithm-config',input});}catch(e){error=e.message;}
 if(error){assert.notEqual(native.status,0);assert(native.stderr.includes(error),native.stderr+' / '+error);failures++;cases.push({structure,backend,seed,state:g.state,units:g.geoids.length,k,real,error});return;}
 assert.equal(native.status,0,native.stderr);const reference=JSON.parse(native.stdout);assert.deepEqual(actual.result.assignments,reference.assignments);assert.equal(actual.result.metrics.within_requested_tolerance,reference.balanced);assert.equal(reference.declared_seed,seed);
 const cut=value=>value===0?0:value;assert.equal(cut(actual.result.metrics.weighted_boundary),cut(reference.weighted_cut));assert.equal(cut(actual.result.metrics.graph_boundary_m),cut(reference.physical_cut));
 assert.equal(actual.configuration.metis_engine_used,false);assert.equal(actual.configuration.browser_execution_backend,'shared-native-rust-no-metis');assert.equal(actual.configuration.declared_engine,backend??'c-ffi');assert.equal(actual.configuration.resolution.metis.engine,backend==='metis-core'?'redist-metis':backend??'c-ffi');assert.equal(actual.configuration.yaml_text,yaml_text);
 const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
 const project=createProject({name:'Unused backend '+structure,files:{configSource:source('no-metis.yml',yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:source('graph.json',JSON.stringify(g))},configRunSettings:{chamber:'congressional',districts:k}},operation:'run-algorithm-config',constraints:[],result:actual,lastOperation:'run-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(project)).result,actual);
 if(!real&&seed==='18446744073709551615'&&backend===null&&structure==='flow-construction')await fs.writeFile(root+'/browser-fixture.bisect',JSON.stringify(project));
 successes++;cases.push({structure,backend,seed,state:g.state,units:g.geoids.length,k,real,balanced:reference.balanced,contiguous:actual.result.metrics.contiguous,original_backend_preserved:true});
}
for(const structure of structures)for(const backend of backends)for(const seed of [null,'42','18446744073709551615'])await check(graph,structure,backend,seed);
const catalogRoot=process.argv[2]||'dist/wasm-lab-build',catalog=JSON.parse(await fs.readFile(path.join(catalogRoot,'catalog.json'),'utf8'));
for(const code of ['RI','IA']){
 const g=JSON.parse(await fs.readFile(path.join(catalogRoot,catalog.graphs[`${code}:2020`].graph_ref),'utf8')),k=catalog.catalog.states.find(s=>s.code===code).years.find(y=>y.year==='2020').districts.congressional;
 for(const structure of structures)for(const backend of backends)await check(g,structure,backend,'18446744073709551615',k,true);
}
for(const structure of ['standard-bisect','nway','ratio-optimal','ratio-optimal-area','compact-polsby','prime-factor'])for(const backend of [null,'c-ffi','gpmetis']){
 const yaml_text=yaml(structure,backend,'42'),validated=engine.execute({operation:'validate-algorithm-config',yaml_text,source_name:'guard.yml',profile:'open'});assert.equal(validated.metis_engine_used,true);const compiled=engine.supportsCMetis&&backend!=='gpmetis';assert.equal(validated.browser_engine_available,compiled);assert.equal(validated.browser_execution_backend,compiled?'c-metis-msvcrt-v1':'unavailable-declared-metis-engine');
 assert.throws(()=>engine.execute({operation:'run-algorithm-config',input:{graph,yaml_text,chamber:'congressional',districts:2,source_name:'guard.yml',profile:'open'}}),undefined);if(compiled)unloadedRuntimeRefusals++;guardRefusals++;
}
for(const structure of structures){const yaml_text=yaml(structure,'unsupported-backend','42');assert.throws(()=>engine.execute({operation:'validate-algorithm-config',yaml_text,source_name:'bad.yml',profile:'open'}),/engine/);guardRefusals++;}
assert.equal(cases.length,80);const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:sha(wasm),native_cli_reference_sha256:sha(await fs.readFile(executable)),cases:cases.length,successes,matching_generation_failures:failures,guard_refusals:guardRefusals,unloaded_runtime_refusals:unloadedRuntimeRefusals,exact_project_roundtrips:successes,metis_called:false,backend_declaration_preserved:true,scope:'Four shared native Rust kernels on identical supplied graphs; no C-FFI/gpmetis execution, backend substitution, arbitrary-capacity or filesystem acquisition claim.',cases_evidence:cases},null,2));
console.log(`No-METIS configuration: ${successes} actual CLI/WASM matches, ${failures} matching failures, ${guardRefusals} operation refusals (${unloadedRuntimeRefusals} C-runtime not loaded); original declarations, execution metadata and inert projects retained.`);
