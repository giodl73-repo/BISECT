import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {manifestReplayRequest,engineReplayReceiptBytes} from '../../web/lab/engine-replay-files.js';
const root='target/replay-multiscale-bindings-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input;
const bgFor=g=>({schema_version:'bisect-block-group-graph-v1',state:g.state,year:g.year,geoids:g.geoids.flatMap(id=>[id+'1',id+'2']),population:g.population.flatMap(p=>[Math.floor(p/2),p-Math.floor(p/2)]),adjacency:Array.from({length:g.geoids.length*2},(_,i)=>[i^1,...g.adjacency[Math.floor(i/2)].map(j=>j*2+i%2)].sort((a,b)=>a-b))});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const change=(source,fn)=>{const v=JSON.parse(Buffer.from(source.base64,'base64').toString());fn(v);source.base64=Buffer.from(JSON.stringify(v)).toString('base64');};
let count=0;const evidence=[];
async function check(name,files,status='match',binding='verified'){
 const request=manifestReplayRequest(files.packageVerificationInputs,files.engineReplayInputs),result=engine.execute(request),path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024})),result);count++;
 assert.equal(result.status,status,result.error);assert.equal(result.manifest_to_run_binding,binding);if(status==='binding-failed'){assert.equal(result.execution_performed,false);assert.equal(result.replay,null);}else{assert.equal(result.replay.agreement.similarity,1);assert.equal(result.binding.multiscale_inputs,'verified');}
 const project=createProject({name:'Multiscale correspondence '+name,files,operation:'verify-manifest-replay',constraints:[],result,lastOperation:'verify-manifest-replay'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(JSON.parse(new TextDecoder().decode(engineReplayReceiptBytes(result))),result);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));evidence.push({name,status,binding});return result;
}
for(const [fine,coarse]of [['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of [false,true]){
 const input=structuredClone(base);Object.assign(input,{fine_level:fine,coarse_level:coarse,total_steps:6,alpha:.3,percentile:0,block_groups:fine==='bg'?bgFor(input.request.graph):null,adaptive:adaptive?{target_accept:.3,adapt_interval:2,gamma_0:.1,coarse_tol_factor:1}:null});Object.assign(input.request.options,{districts:2,seed:'18446744073709551615',balance_tolerance:50});
 const fine_edges=input.block_groups?input.block_groups.adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100+(u%7)/10])):null;
 const generated=engine.execute({operation:'run-multiscale',input}),exported=engine.execute({operation:'export-multiscale-plan',input,fine_edges,assignments:generated.assignments,label:'multiscale-bound',chamber:'congressional',created_at:'2026-10-08T00:00:00Z'}),manifest=engine.execute({operation:'prepare-plan-report-manifest',document_json:exported.document_json,context_json:exported.context_json});
 // The manifest must record the multiscale producer's effective options exactly.
 assert.match(manifest.manifest_json,/"engine_seed": 18446744073709551615/);assert.equal(JSON.parse(manifest.manifest_json).balance_tolerance_pct,50);
 const files={packageVerificationInputs:{manifest:source('manifest.json',manifest.manifest_json),files:{'plan.rplan':source('plan.rplan',exported.document_json),'context.rctx':source('context.rctx',exported.context_json)}},engineReplayInputs:{run:source('run.json',JSON.stringify({kind:'multiscale',input,fine_edges})),original:source('original.json',JSON.stringify(generated.assignments)),minSimilarity:1}},name=fine+'-'+coarse+'-'+(adaptive?'adaptive':'fixed');
 const good=await check(name,files);assert.equal(good.binding.fine_boundaries,'verified');assert.deepEqual(good.replay.replayed_result,generated);
 const reencoded=structuredClone(files);change(reencoded.packageVerificationInputs.files['plan.rplan'],()=>{});await check(name+'-reencoded',reencoded);
 for(const [suffix,mutate]of [['coarse-graph',a=>a.input.request.graph.population[0]++],['steps',a=>a.input.total_steps++],['alpha',a=>a.input.alpha=.4]]){const altered=structuredClone(files);change(altered.engineReplayInputs.run,mutate);await check(name+'-'+suffix,altered,'binding-failed','failed');}
 if(adaptive){const altered=structuredClone(files);change(altered.engineReplayInputs.run,a=>a.input.adaptive.gamma_0=.2);await check(name+'-adaptation',altered,'binding-failed','failed');}
 if(fine_edges){
  const absent=structuredClone(files);change(absent.engineReplayInputs.run,a=>delete a.fine_edges);const partial=await check(name+'-missing-boundaries',absent,'match','partial');assert.equal(partial.binding.fine_boundaries,'not-selected');
  for(const [suffix,mutate]of [['physical-weight',a=>a.fine_edges[0][2]++],['missing-edge',a=>a.fine_edges.pop()],['fine-graph',a=>a.input.block_groups.population[0]++],['boundary-order',a=>a.fine_edges.reverse()]]){const altered=structuredClone(files);change(altered.engineReplayInputs.run,mutate);await check(name+'-'+suffix,altered,'binding-failed','failed');}
 }else{const altered=structuredClone(files);change(altered.engineReplayInputs.run,a=>a.fine_edges=[]);await check(name+'-unused-boundaries',altered,'binding-failed','failed');}
 const build=structuredClone(files);change(build.engineReplayInputs.run,a=>a.kind='multiscale-build');const built=await check(name+'-build',build);assert.equal(built.replay.build_report.accepted_attempt,0);assert.deepEqual(built.replay.replayed_result,engine.execute({operation:'run-multiscale-build',input}));
}
for(const seed of [null,42]){
 const project=JSON.parse(await fs.readFile(root+'/tract-county-fixed.bisect','utf8')),artifact=JSON.parse(Buffer.from(project.files.engineReplayInputs.run.base64,'base64').toString()),input=artifact.input;input.request.options.seed=seed;input.total_steps=0;input.alpha=0;
 const generated=engine.execute({operation:'run-multiscale',input}),exported=engine.execute({operation:'export-multiscale-plan',input,fine_edges:null,assignments:generated.assignments,label:'seed-bound',chamber:'congressional',created_at:'2026-10-08T00:00:00Z'}),manifest=engine.execute({operation:'prepare-plan-report-manifest',document_json:exported.document_json,context_json:exported.context_json});
 const recorded=JSON.parse(manifest.manifest_json);assert.equal(recorded.seed,seed);assert.equal(recorded.engine_seed??null,null);
 const files={packageVerificationInputs:{manifest:source('manifest.json',manifest.manifest_json),files:{'plan.rplan':source('plan.rplan',exported.document_json),'context.rctx':source('context.rctx',exported.context_json)}},engineReplayInputs:{run:source('run.json',JSON.stringify(artifact)),original:source('original.json',JSON.stringify(generated.assignments)),minSimilarity:1}};
 const result=await check('tract-seed-'+seed,files);assert.deepEqual(result.replay.replayed_result,generated);
}
{
 const project=JSON.parse(await fs.readFile(root+'/tract-county-fixed.bisect','utf8')),input=JSON.parse(Buffer.from(project.files.engineReplayInputs.run.base64,'base64').toString()).input;input.request.options.seed=null;
 const request={operation:'run-multiscale',input},generated=engine.execute(request);await fs.writeFile(root+'/seedless-sampling-request.json',JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/seedless-sampling-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024})),generated);
 const exportRequest={operation:'export-multiscale-plan',input,fine_edges:null,assignments:generated.assignments,label:'observed-native-result',chamber:'congressional',created_at:'2026-10-08T00:00:00Z'};let rejected=false,error=null;try{engine.execute(exportRequest);}catch(e){rejected=true;error=e.message;}
 await fs.writeFile(root+'/seedless-sampling-export.json',JSON.stringify(exportRequest));let nativeRejected=false;try{execFileSync('target/release/examples/execute_tool_request.exe',[root+'/seedless-sampling-export.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe']});}catch(e){nativeRejected=true;assert.ok(e.stderr.includes(error));}assert.equal(nativeRejected,rejected);
 await fs.writeFile(root+'/seedless-sampling-observation.json',JSON.stringify({native_wasm_generation_exact:true,native_wasm_export_rejection_matches:true,represented_districts:new Set(Object.values(generated.assignments)).size,requested_districts:input.request.options.districts,export_rejected:rejected,error},null,2));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:count,cases:evidence,synthetic_graphs:true,physical_edges_used_by_sampler:false},null,2));console.log(`Multiscale correspondence: ${count} native/WASM cases; all fine/coarse pairs, fixed/adaptive/build paths, exact u64 manifest, original assignments, coarse/fine inputs, settings, physical boundary/order evidence and missing-boundary partial receipts passed.`);
