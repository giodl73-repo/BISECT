import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {manifestReplayRequest,engineReplayReceiptBytes} from '../../web/lab/engine-replay-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/manifest-replay-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let parity=0;const evidence=[];
async function check(name,packageName,replayName,status,modify=null){
 const pre=JSON.parse(await fs.readFile('target/manifest-prechecks-wasm/'+packageName+'.bisect','utf8')),run=JSON.parse(await fs.readFile('target/engine-replay-wasm/'+replayName+'.bisect','utf8'));
 const artifact=JSON.parse(Buffer.from(run.files.engineReplayInputs.run.base64,'base64').toString()),graph=artifact.request.graph,options=artifact.request.options,manifest=JSON.parse(Buffer.from(pre.files.packageVerificationInputs.manifest.base64,'base64').toString());
 Object.assign(manifest,{state_code:graph.state,year:graph.year,num_districts:options.districts,seed:null,engine_seed:options.seed==null?null:options.seed,balance_tolerance_pct:options.balance_tolerance,rplan_path:null,rctx_path:null,audit_certificate_path:null});
 // Construct matching selected plan/context; certificate prechecks have separate regression coverage.
 try{
  let assignments=JSON.parse(Buffer.from(run.files.engineReplayInputs.original.base64,'base64').toString());if(!Object.keys(assignments).length)assignments=engine.execute(artifact.request).assignments;
  const exported=engine.execute({operation:'export-engine-plan',request:artifact.request,assignments,label:'selected-run',chamber:manifest.chamber,created_at:'2026-10-08T00:00:00Z'});
  Object.assign(manifest,{rplan_path:'plan.rplan',rctx_path:'context.rctx',context_hash:exported.context_hash});
  pre.files.packageVerificationInputs.files['plan.rplan']={name:'plan.rplan',base64:Buffer.from(exported.document_json).toString('base64')};pre.files.packageVerificationInputs.files['context.rctx']={name:'context.rctx',base64:Buffer.from(exported.context_json).toString('base64')};
 }catch(error){assert.equal(replayName,'failed-execution',error.message);}
 // Typed native manifest requires numeric u64, so splice exact raw seed without JS rounding.
 let text=JSON.stringify(manifest);if(typeof options.seed==='string')text=text.replace('"engine_seed":"'+options.seed+'"','"engine_seed":'+options.seed);
 pre.files.packageVerificationInputs.manifest.base64=Buffer.from(text).toString('base64');
 const files={packageVerificationInputs:pre.files.packageVerificationInputs,engineReplayInputs:run.files.engineReplayInputs},request=manifestReplayRequest(files.packageVerificationInputs,files.engineReplayInputs);if(modify)modify(request);
 const result=engine.execute(request),path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024})),result);parity++;
 assert.equal(result.status,status);assert.equal(result.verification_exit_code,status==='match'?0:1);assert.ok(['verified','partial','unavailable','not-run','failed'].includes(result.manifest_to_run_binding));
 if(['precheck-failed','binding-failed'].includes(status)){assert.equal(result.replay,null);assert.equal(result.execution_performed,false);}
 else {assert.deepEqual(result.prechecks,engine.execute({operation:'verify-manifest-prechecks',input:request.input.prechecks}));if(status!=='replay-failed'||result.replay)assert.deepEqual(result.replay,engine.execute({operation:'replay-engine-run',input:request.input.replay}));}
 assert.ok(countResultTables(result).some(t=>t.rows.some(r=>r[0]==='Manifest-to-run correspondence'&&r[1]===result.manifest_to_run_binding)));
 assert.deepEqual(JSON.parse(new TextDecoder().decode(engineReplayReceiptBytes(result))),result);
 if(!modify){const project=createProject({name:'Manifest and replay: '+name,files,operation:'verify-manifest-replay',constraints:[],result,lastOperation:'verify-manifest-replay'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));}
 await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));evidence.push({name,status,execution_performed:result.execution_performed});return result;
}
await check('match','bound-lineage','engine-18446744073709551615','match');
await check('different','bound-lineage','different','different');
await check('empty','bound-lineage','empty-original','unavailable');
await check('unrecorded','not-recorded','engine-42','match');
await check('bad-digest','bad-digest','engine-42','precheck-failed');
await check('changed-model','changed-model','engine-42','precheck-failed');
await check('invalid-run-blocked','changed-model','engine-42','precheck-failed',r=>r.input.replay.run_json='{bad');
await check('invalid-original-blocked','bad-digest','engine-42','precheck-failed',r=>r.input.replay.original_assignments_json='[]');
await check('invalid-run','bound-lineage','engine-42','binding-failed',r=>r.input.replay.run_json='{bad');
await check('invalid-original','bound-lineage','engine-42','binding-failed',r=>r.input.replay.original_assignments_json='[]');
await check('generation-failed','bound-lineage','failed-execution','replay-failed');
const request=JSON.parse(await fs.readFile(root+'/request.json','utf8'));request.input.replay.run_json=' '.repeat(16*1024*1024);assert.throws(()=>engine.execute(request),/Combined verification sources/);
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:parity,cases:evidence,combined_capacity_rejection:true,manifest_run_binding:'checked-when-recorded'},null,2));
console.log(`Manifest/replay composition: ${parity} native/WASM cases; stage parity, precheck short-circuit, absent sidecars, failures, exact seed, project/receipt retention and capacity rejection passed. Recorded manifest/run correspondence is checked before replay; missing evidence remains partial.`);
