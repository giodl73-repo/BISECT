// Real retained state data: measure native/WASM discovery and concrete browser proofs.
// This is a capacity gate, not an assertion that every seed can be certified.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {gunzipSync} from 'node:zlib';
import {pathToFileURL} from 'node:url';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {sha256} from '../../web/lab/proof-solver-core.js';
import {prepareNativeCertifiedModel} from '../../web/lab/native-model-prepare.js';
import {generateNativeProofTree} from '../../web/lab/native-tree-generate.js';
const states=process.argv.slice(2);if(!states.length)states.push('RI');assert(states.every(s=>/^[A-Z]{2}$/.test(s)));
const secondsText=process.env.BISECT_PROOF_SECONDS??'1';assert.match(secondsText,/^[1-9]\d{0,2}$/);const seconds=Number(secondsText);assert(seconds<=120,'Proof budget must be 1–120 seconds per decision.');
const root='target/native-tree-state-capacity'+(seconds===1?'':'-'+seconds+'s');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const checkerBase='target/proof-checker-wasm-build/wasm32-unknown-unknown/release/';
const {default:factory}=await import(pathToFileURL(path.resolve('target/roundingsat-wasm/roundingsat.mjs')).href);
const options={engine,seed:'42',refinement:'full',seconds,solverWasm:await fs.readFile('target/roundingsat-wasm/roundingsat.wasm'),solverReceipt:JSON.parse(await fs.readFile('target/roundingsat-wasm/build-receipt.json','utf8')),checkerWasm:await fs.readFile(checkerBase+'bisect_proof_checker.wasm'),checkerReceipt:JSON.parse(await fs.readFile(checkerBase+'proof-checker-build.json','utf8'))};
for(const state of states)for(const weights of ['geographic','unweighted']){
 const template=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,weights,seed:'42',workers:1,years:['2020']}));
 const start=performance.now(),prefix='target/original-all-state-wasm/'+state,out=root+'/'+state+'/'+weights;await fs.mkdir(out,{recursive:true});
 const receipt=JSON.parse(await fs.readFile(prefix+'/receipt.json','utf8'));assert.equal(receipt.native_source_graph_parity,true);
 const graphBytes=gunzipSync(await fs.readFile(prefix+'/engine-graph.json.gz'));assert.equal(await sha256(graphBytes),receipt.graph_sha256);const graph=JSON.parse(graphBytes);
 assert.deepEqual(graph.geoids,[...graph.geoids].sort(),'Preserve native unit ordering; never silently reorder a source.');
 const parity=async request=>{const file=path.resolve(out+'/request.json');await fs.writeFile(file,JSON.stringify(request));const actual=engine.execute(request),native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,timeout:120000,maxBuffer:96*1024*1024}));assert.deepEqual(actual,native);return actual;};
 const exported=await parity({operation:'export-configured-plan',input:{graph,yaml_text:template.yaml_text,source_name:template.generated_filename,profile:'open',chamber:'congressional',districts:receipt.districts},assignments:JSON.parse(await fs.readFile(prefix+'/assignments.json','utf8')),label:'capacity_'+state.toLowerCase(),created_at:'2026-10-12T00:00:00Z',attempt:receipt.accepted_attempt});
 await fs.writeFile(out+'/original.rctx',exported.context_json);const context=JSON.parse(exported.context_json);assert.deepEqual(context.units.unit_ids,graph.geoids);assert.deepEqual(context.populations,graph.population);
 const discoveryRequest={operation:'solve-certified-discovery-context',context_json:exported.context_json,settings:{districts:receipt.districts,tolerance:0.5,generated_at:'2026-10-12T00:00:00Z',seed:'42',refinement:'full'}};
 if(weights==='geographic'){
  assert(context.graph.adjacency.some(row=>row.some(edge=>!Number.isInteger(edge.weight))));
  assert.throws(()=>engine.execute(discoveryRequest),/Certified split edges require positive integer weights/);
  const file=path.resolve(out+'/request.json');await fs.writeFile(file,JSON.stringify(discoveryRequest));const refused=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',windowsHide:true,timeout:120000});assert.notEqual(refused.status,0);assert.match(refused.stderr,/Certified split edges require positive integer weights/);
  await fs.writeFile(out+'/evidence.json',JSON.stringify({state,weights,units:graph.geoids.length,districts:receipt.districts,status:'native-model-refused',error:'Certified split edges require positive integer weights',source_graph_sha256:receipt.graph_sha256,context_source_sha256:await sha256(Buffer.from(exported.context_json)),original_order_and_population_preserved:true,native_wasm_export_match:true,native_wasm_refusal_match:true,complete_certificate_withheld:true,elapsed_ms:performance.now()-start,scope:'Original geographic weights retained exactly; no rounding or replacement was performed.'},null,2)+'\n');
  console.log(state,weights,'native-model-refused');continue;
 }
 const prepared=await prepareNativeCertifiedModel({contextJson:exported.context_json,districts:receipt.districts,weightPolicy:'unweighted-edge-count',wasmBinary:options.checkerWasm,receipt:options.checkerReceipt});
 const preparedRequest={operation:'prepare-native-certified-model',context_json:exported.context_json,districts:receipt.districts,weight_policy:'unweighted-edge-count'};
 const nativePrepared=JSON.parse(execFileSync('target/proof-checker-build/release/bisect-proof-checker.exe',[],{input:JSON.stringify(preparedRequest),encoding:'utf8',windowsHide:true,maxBuffer:32*1024*1024}));assert.equal(nativePrepared.ok,true);assert.deepEqual(nativePrepared.result,prepared);assert.equal(prepared.original_context_json,exported.context_json);
 await fs.writeFile(out+'/model.rctx',prepared.model_context_json);await fs.writeFile(out+'/preparation.json',JSON.stringify(prepared));
 assert(JSON.parse(prepared.model_context_json).graph.adjacency.every(row=>row.every(edge=>edge.weight===1)));
 const discovered=await parity({...discoveryRequest,context_json:prepared.model_context_json});
 const rootInstanceJson=prepared.root_instance_json,discoveryJson=discovered.files['certified-discovery.json'];await fs.writeFile(out+'/root-instance.json',rootInstanceJson);await fs.writeFile(out+'/discovery.json',discoveryJson);
 const land=prepared.land_adjacency;await fs.writeFile(out+'/land-adjacency.json',JSON.stringify(land));
 const direct=await parity({operation:'solve-certified-discovery-instance',instance_json:rootInstanceJson,settings:{seed:'42',refinement:'full',land_adjacency:land}});assert.equal(direct.discovery_json,discoveryJson);assert.equal(rootInstanceJson,discovered.files['certified-split-instance.json']);
 const compiled=await parity({operation:'compile-certified-proof-models',instance_json:rootInstanceJson,discovery_json:discoveryJson});
 const metrics={state,weights,weight_choice_scope:'Explicit native unweighted option; edge-count objective differs from geographic boundary weighting.',year:graph.year,units:graph.geoids.length,edges:JSON.parse(rootInstanceJson).edges.length,population:graph.population.reduce((s,p)=>s+BigInt(p),0n).toString(),districts:receipt.districts,source_graph_sha256:receipt.graph_sha256,context_source_sha256:await sha256(Buffer.from(exported.context_json)),engine_wasm_sha256:engine.artifactIdentity.sha256,checker_wasm_sha256:options.checkerReceipt.wasm_sha256,seed:'42',refinement:'full',solver_seconds_per_decision:seconds,native_wasm_model_preparation_match:true,native_wasm_export_discovery_and_compiler_match:true,original_unit_order_and_populations_preserved:true,land_bridge_semantics_preserved:true,model_bytes:Object.fromEntries(Object.entries(compiled.artifacts).map(([stem,a])=>[stem,a.bytes])),solver_peak_linear_memory_bytes:0,phases:[]};
 const solverFactory=async config=>{let memory;const module=await factory({...config,instantiateWasm(imports,receive){const instance=new WebAssembly.Instance(new WebAssembly.Module(options.solverWasm),imports);memory=Object.values(instance.exports).find(value=>value instanceof WebAssembly.Memory);assert.ok(memory,'Measure the actual exported solver linear memory.');receive(instance);return instance.exports;}}),callMain=module.callMain;module.callMain=(...args)=>{try{return callMain(...args);}finally{metrics.solver_peak_linear_memory_bytes=Math.max(metrics.solver_peak_linear_memory_bytes,memory.buffer.byteLength);}};return module;};
 try{
  const result=await generateNativeProofTree({...options,rootInstanceJson,landAdjacency:land,solverFactory,onProgress:p=>{const sample={...p,elapsed_ms:performance.now()-start,process_rss_bytes:process.memoryUsage().rss};metrics.phases.push(sample);console.log(state,p.phase,p.stem||'',p.node_path||'(root)');}});
  assert(['tree-generated-and-checked','tree-generation-incomplete'].includes(result.status));metrics.status=result.status;metrics.completed_nodes=result.completed_nodes;metrics.stopped_run_status=result.stopped_run?.status??null;metrics.stopped_solver_status=result.stopped_run?.decisions.at(-1)?.solver_status??null;if(metrics.stopped_run_status==='proof-run-incomplete')assert(['solver-reported-time-limit','solver-proof-output-limit'].includes(metrics.stopped_solver_status));
  if(result.status==='tree-generation-incomplete'){assert.equal(result.tree,undefined);assert.equal(result.bundle,undefined);assert(['checked-counterexample','proof-run-incomplete'].includes(result.stopped_run.status));metrics.complete_certificate_withheld=true;const decision=result.stopped_run.decisions.at(-1);metrics.stopped_proof_status=decision.proof_status;const solverReport=JSON.parse(result.stopped_run.files[decision.stem+'.solver.json']);metrics.stopped_proof_bytes=solverReport.proof_bytes;metrics.proof_byte_limit=solverReport.proof_byte_limit;metrics.proof_truncated=solverReport.proof_truncated;assert.equal(decision.proof_status,'unverified');assert(solverReport.proof_bytes<=solverReport.proof_byte_limit);}
  else{assert.equal(result.tree.leaves.length,receipt.districts);metrics.complete_certificate_withheld=false;}
  await fs.writeFile(out+'/run.json',JSON.stringify(result));
 }catch(error){
  // Only an explicit supported capacity refusal is a measured outcome. Bugs fail the gate.
  if(!/at most 8 MiB|exceeds.*MiB|exceed.*limit|working set/.test(error.message))throw error;
  metrics.status='capacity-refused';metrics.error=error.message;metrics.complete_certificate_withheld=true;
 }
 metrics.elapsed_ms=performance.now()-start;metrics.process_peak_resident_bytes=process.resourceUsage().maxRSS*1024;metrics.memory_scope='Node process RSS/high-water mark and solver WASM linear memory; not browser-tab memory or checker-only memory.';
 metrics.scope='Retained original native state graph, full native/WASM export/discovery/compiler parity and actual proof attempt. No synthetic aggregation, silent source sorting, bounded oracle, automatic retry or statewide optimality claim.';
 await fs.writeFile(out+'/evidence.json',JSON.stringify(metrics,null,2)+'\n');console.log(state,metrics.status,'in',Math.round(metrics.elapsed_ms),'ms');
 // Emscripten sets the Node exit code on its intentional solver timeout.
 process.exitCode=0;
}
