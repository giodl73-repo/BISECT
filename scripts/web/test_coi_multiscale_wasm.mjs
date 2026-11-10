import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {manifestReplayRequest,engineReplayReceiptBytes} from '../../web/lab/engine-replay-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/coi-multiscale-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const base=JSON.parse(await fs.readFile('target/multiscale-fixtures/request-0-0.json','utf8')).input;
base.request.options.iterations=100; // Native YAML compositor's fixed niter default.
const g=base.request.graph,bg={schema_version:'bisect-block-group-graph-v1',state:g.state,year:g.year,geoids:g.geoids.flatMap(id=>[id+'1',id+'2']),population:g.population.flatMap(p=>[p/2,p/2]),adjacency:Array.from({length:g.geoids.length*2},(_,i)=>[i^1,...g.adjacency[Math.floor(i/2)].map(j=>j*2+i%2)].sort((a,b)=>a-b))};
const weights={[g.geoids[0]]:4,[g.geoids[1]]:9,[g.geoids.at(-1)]:16,[bg.geoids[0]]:100,unknown:25};
const text=JSON.stringify(weights,null,2)+'\n',coi={source_name:'selected-coi.json',weights_json:text},hash=createHash('sha256').update(text).digest('hex');
const adjusted=g.edges.map(([u,v,w])=>[u,v,w*Math.sqrt((weights[g.geoids[u]]??1)*(weights[g.geoids[v]]??1))]);
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const change=(source,fn)=>{const v=JSON.parse(Buffer.from(source.base64,'base64').toString());fn(v);source.base64=Buffer.from(JSON.stringify(v)).toString('base64');};
let native=0,bound=0,multiscale=0;
async function host(name,request,result){const file=root+'/'+name+'.json';await fs.writeFile(file,JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:64*1024*1024})));native++;}
async function check(name,files,status='match',aux='verified'){
 const req=manifestReplayRequest(files.packageVerificationInputs,files.engineReplayInputs),out=engine.execute(req);await host(name,req,out);bound++;assert.equal(out.status,status,out.error);
 if(status==='binding-failed'){assert.equal(out.execution_performed,false);assert.equal(out.replay,null);}else{assert.equal(out.binding.auxiliary_inputs,aux);assert.equal(out.binding.auxiliary_bindings.coi,aux==='verified'?'verified':'not-recorded');assert.equal(out.replay.agreement.similarity,1);}
 const project=createProject({name,operation:'verify-manifest-replay',lastOperation:'verify-manifest-replay',constraints:[],files,result:out});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(JSON.parse(new TextDecoder().decode(engineReplayReceiptBytes(out))),out);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));return out;
}
async function bindings(name,artifact,generated,exported,missing=true){
 const manifest=engine.execute({operation:'prepare-plan-report-manifest',document_json:exported.document_json,context_json:exported.context_json});
 const files={packageVerificationInputs:{manifest:source('manifest.json',manifest.manifest_json),files:{'plan.rplan':source('plan.rplan',exported.document_json),'context.rctx':source('context.rctx',exported.context_json)}},engineReplayInputs:{run:source('run.json',JSON.stringify(artifact)),original:source('original.json',JSON.stringify(generated.assignments)),minSimilarity:1}};
 const out=await check(name,files);assert.deepEqual(out.replay.replayed_result,generated);
 for(const [suffix,mutate]of [['relabelled',c=>c.source_name='other.json'],['same-values-new-bytes',c=>c.weights_json+=' '],['changed-weight',c=>c.weights_json=JSON.stringify({...weights,[g.geoids[0]]:5})]]){
  const altered=structuredClone(files);change(altered.engineReplayInputs.run,a=>mutate(a.request?.coi||a.input.request.coi));await check(name+'-'+suffix,altered,'binding-failed');
 }
 const absent=structuredClone(files);change(absent.engineReplayInputs.run,a=>delete (a.request||a.input.request).coi);await check(name+'-absent',absent,'binding-failed');
 if(missing){const altered=structuredClone(files);change(altered.packageVerificationInputs.files['plan.rplan'],p=>delete p.provenance.producer.coi_input);await check(name+'-missing-provenance',altered,'match','partial');}
 return files;
}
// Independent native CLI filesystem post-weighter and initial assignment dispatch.
await fs.writeFile(root+'/selected-coi.json',text);await fs.writeFile(root+'/native-edges.json',JSON.stringify({geoids:g.geoids,edges:g.edges,coi_path:path.resolve(root+'/selected-coi.json')}));
assert.deepEqual(JSON.parse(execFileSync('target/debug/examples/coi_weights_reference.exe',[root+'/native-edges.json'],{encoding:'utf8'})),adjusted);
const yaml='name: coi_multiscale_seed\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  weights: geographic\n  search: single\n  seed: 42\n  balance_tolerance: 5\n';
await fs.writeFile(root+'/native-seed.json',JSON.stringify({graph:{...g,edges:adjusted},yaml_text:yaml,options:{districts:3}}));const initial=JSON.parse(execFileSync('target/debug/examples/config_seed_reference.exe',[root+'/native-seed.json'],{encoding:'utf8'})).assignments;
for(const [fine,coarse]of [['tract','county'],['bg','tract'],['bg','county']])for(const adaptive of [false,true])for(const steps of [0,10]){
 const input=structuredClone(base);Object.assign(input,{fine_level:fine,coarse_level:coarse,total_steps:steps,alpha:.3,percentile:0,block_groups:fine==='bg'?bg:null,adaptive:adaptive?{target_accept:.3,adapt_interval:5,gamma_0:.1,coarse_tol_factor:3}:null});input.request.coi=coi;input.request.options.seed=42;
 const name=fine+'-'+coarse+'-'+adaptive+'-'+steps,req={operation:'run-multiscale',input},out=engine.execute(req);await host(name,req,out);multiscale++;
 const e=out.evidence.coi_weighting;assert.equal(e.source_sha256,hash);assert.equal(e.matched_units,3);assert.equal(e.ignored_units,2);assert.equal(e.defaulted_units,g.geoids.length-3);assert.match(e.algorithm_scope,/tract-seed-only/);
 const manual=structuredClone(input);delete manual.request.coi;manual.request.graph.edges=adjusted;const expected=engine.execute({operation:'run-multiscale',input:manual}),actual=structuredClone(out);delete actual.evidence.coi_weighting;delete actual.evidence.graph_sha256;delete expected.evidence.graph_sha256;assert.deepEqual(actual,expected);
 if(steps===0)for(const [id,d]of Object.entries(out.assignments))assert.equal(d,initial[id.slice(0,11)]);
 const plain=structuredClone(input);delete plain.request.coi;const plainOut=engine.execute({operation:'run-multiscale',input:plain});assert.equal(out.evidence.graph_sha256,plainOut.evidence.graph_sha256);
 const onlyBg=structuredClone(input);onlyBg.request.coi={source_name:'bg-only.json',weights_json:JSON.stringify({[bg.geoids[0]]:1e100})};const ignored=engine.execute({operation:'run-multiscale',input:onlyBg});assert.equal(ignored.evidence.coi_weighting.changed_edges,0);delete ignored.evidence.coi_weighting;assert.deepEqual(ignored,plainOut);
 const fine_edges=fine==='bg'?bg.adjacency.flatMap((ns,u)=>ns.filter(v=>v>u).map(v=>[u,v,100])):null;
 const exportReq={operation:'export-multiscale-plan',input,fine_edges,assignments:out.assignments,label:'coi_multiscale',chamber:'congressional',created_at:'2026-10-09T00:00:00Z'},exported=engine.execute(exportReq);await host(name+'-export',exportReq,exported);assert.equal(exported.context.source_hashes['bisect.coi-weights-json'],hash);assert.equal(exported.document.provenance.producer.multiscale_input.request.coi.weights_json,text);assert.deepEqual(exported.context.populations,fine==='bg'?bg.population:g.population);
 const physical=fine_edges||g.edges;exported.context.graph.adjacency.forEach((ns,u)=>ns.forEach(e=>assert.equal(e.weight,physical.find(([a,b])=>a===Math.min(u,e.to)&&b===Math.max(u,e.to))[2])));
 if(steps===0)await bindings(name,{kind:'multiscale',input,fine_edges},out,exported,false);
 if(steps===0&&fine==='bg'&&coarse==='tract'&&adaptive){const built=engine.execute({operation:'run-multiscale-build',input});assert.equal(built.build_report.accepted_attempt,0);await host(name+'-build-run',{operation:'run-multiscale-build',input},built);await bindings(name+'-build',{kind:'multiscale-build',input,fine_edges},built,exported,false);}
}
for(const mode of ['geographic','unweighted']){
 const request={graph:g,options:{...base.request.options,weights:mode,seed:42},coi},generated=engine.execute(request),exported=engine.execute({operation:'export-engine-plan',request,assignments:generated.assignments,label:'coi_binding',chamber:'congressional',created_at:'2026-10-09T00:00:00Z'});await bindings('engine-'+mode,{kind:'engine',request},generated,exported);
}
for(const bad of ['{"x":-1}','{"x":1,"\\u0078":2}','{"x":1e400}',JSON.stringify({[g.geoids[0]]:1e308,[g.geoids[1]]:1e308})]){const input=structuredClone(base);input.request.coi={source_name:'invalid.json',weights_json:bad};assert.throws(()=>engine.execute({operation:'run-multiscale',input}));}
console.log(`COI multiscale: ${multiscale} fixed/adaptive native-host/WASM runs across all resolution pairs; independent native file/initial assignment oracles; tract-only weighting, ignored BG IDs, original physical exports, ${bound} replay correspondence checks and ${native} full native-host comparisons passed.`);
