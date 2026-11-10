import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {configRunRequest,configuredRunExportRequest} from '../../web/lab/config-run.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {parseProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/config-build-wasm';await fs.mkdir(root,{recursive:true});const evidence=[];
const base=JSON.parse(await fs.readFile('target/optional-seed-wasm/case-0.json','utf8')).graph;
const graph={...base,population:base.population.map((_,i)=>100+(i*37)%67)};
const native=(exe,path)=>JSON.parse(execFileSync(exe,[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
async function parity(request){const path=root+'/tool.json';await fs.writeFile(path,JSON.stringify(request));const result=engine.execute(request);assert.deepEqual(result,native('target/release/examples/execute_tool_request.exe',path));return result;}
for(const [structure,search] of [['standard-bisect','single'],['standard-bisect','multi'],['nway','single'],['ratio-optimal','single'],['ratio-optimal-area','single']])for(const seed of [null,'0','42','18446744073709551615']){
 const yaml_text=`name: retry\nalgorithm:\n  structure: ${structure}\n  engine: metis-core\n  search: ${search}\n  seeds: 3\n  balance_tolerance: 2\n`+(seed===null?'':`  seed: ${seed}\n`);
 const source={name:'retry.yml',base64:Buffer.from(yaml_text).toString('base64')};
 const request=configRunRequest(source,{profile:'open'},{graph:{name:'graph.json',base64:Buffer.from(JSON.stringify(graph)).toString('base64')}},{chamber:'congressional',districts:4,native_build:true});assert.equal(request.operation,'run-algorithm-build');
 let result;try{result=await parity(request);}catch(error){if(error instanceof assert.AssertionError)throw error;const path=root+'/failed.json';await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>native('target/release/examples/execute_tool_request.exe',path));if(!/exhausted after 51 attempts/.test(error.message)){assert.match(error.message,/multi search found no balanced contiguous cut|no sampled feasible root split/);const oracle=root+'/failed-oracle.json';await fs.writeFile(oracle,JSON.stringify({graph,yaml_text,options:{districts:4},attempt:0}));assert.throws(()=>native('target/debug/examples/config_seed_reference.exe',oracle));}evidence.push({structure,search,seed,exhausted:/exhausted after 51 attempts/.test(error.message),generation_error:!/exhausted after 51 attempts/.test(error.message)});continue;}
 const report=result.build_report;assert.equal(report.max_balance_retries,50);assert.equal(report.attempts.length,report.accepted_attempt+1);assert.equal(result.result.metrics.within_requested_tolerance,true);
 for(let attempt=0;attempt<=report.accepted_attempt;attempt++){
  const path=root+'/oracle.json';await fs.writeFile(path,JSON.stringify({graph,yaml_text,options:{districts:4},attempt}));
  const oracle=native('target/debug/examples/config_seed_reference.exe',path);
  assert.equal(oracle.declared_seed,report.attempts[attempt].seed);assert.equal(oracle.balanced,attempt===report.accepted_attempt);
  if(oracle.balanced)assert.deepEqual(result.result.assignments,oracle.assignments);
 }
 const exportRequest=configuredRunExportRequest(result,{label:'Native build',created_at:'2026-10-07T00:00:00Z'});assert.equal(exportRequest.attempt,report.accepted_attempt);
 const exported=await parity(exportRequest);assert.equal(exported.configuration.seed,seed);assert.equal(exported.configuration.execution_seed,report.attempts.at(-1).seed);assert.equal(exported.document.provenance.producer.engine_options.seed==null?null:String(exported.document.provenance.producer.engine_options.seed),report.attempts.at(-1).seed);
 assert.ok(countResultTables(result)[0].rows.some(([k,v])=>k==='Accepted attempt'&&v===String(report.accepted_attempt)));
 evidence.push({structure,search,seed,accepted_attempt:report.accepted_attempt,effective_seed:report.attempts.at(-1).seed});
}
assert.ok(evidence.some(e=>e.accepted_attempt>0));assert.ok(evidence.some(e=>e.seed==='18446744073709551615'&&e.accepted_attempt>0));
const impossible={...base,population:base.population.map((_,i)=>i===0?100000:1)};
const failed={operation:'run-algorithm-build',input:{graph:impossible,yaml_text:'name: impossible\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  seed: 42\n',source_name:'fail.yml',profile:'open',chamber:'congressional',districts:2}};
assert.throws(()=>engine.execute(failed),/exhausted after 51 attempts/);await fs.writeFile(root+'/exhausted.json',JSON.stringify(failed));assert.throws(()=>native('target/release/examples/execute_tool_request.exe',root+'/exhausted.json'));
const badExport={operation:'export-configured-plan',input:failed.input,assignments:Object.fromEntries(base.geoids.map(id=>[id,1])),label:'Bad',created_at:'now',attempt:51};assert.throws(()=>engine.execute(badExport),/exceeds/);
await fs.writeFile(root+'/evidence.json',JSON.stringify({cases:evidence,exhaustion_returns_error:true,exact_native_wasm:true,native_cli_assignment_and_balance_oracles:true,accepted_seed_export:true},null,2));
console.log(`Configured native build: ${evidence.length} cases, actual CLI attempt/seed/population oracles, exact native/WASM export, None/u64 wraparound and 51-attempt exhaustion passed.`);

// Restore the original input-only shared fixture, including its exact selected
// raw JSON/YAML bytes. This source fixture is never a completed-run authority.
const fixture=parseProject(await fs.readFile(new URL('./fixtures/config-build-retry-inputs.json',import.meta.url),'utf8'));
assert.equal(fixture.result,null);assert.equal(fixture.lastOperation,null);
assert.deepEqual(JSON.parse(Buffer.from(fixture.files.configRunInputs.graph.base64,'base64').toString('utf8')),graph);
assert.deepEqual(fixture.files.configRunSettings,{chamber:'congressional',districts:4,native_build:true});
assert.equal(Buffer.from(fixture.files.configSource.base64,'base64').toString('utf8'),'name: retry\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  search: single\n  seed: 42\n  balance_tolerance: 2\n');
const fixtureRun=await parity(configRunRequest(fixture.files.configSource,fixture.files.configSettings,fixture.files.configRunInputs,fixture.files.configRunSettings));
assert(fixtureRun.build_report.accepted_attempt>0,'The original shared fixture must exercise native population retries');
await fs.writeFile(root+'/browser-retry.bisect',JSON.stringify(fixture));
assert.deepEqual(parseProject(await fs.readFile(root+'/browser-retry.bisect','utf8')),fixture);
console.log('Original configured retry input fixture regenerated with exact selected-source bytes and native/WASM retry verification; saved project remains unexecuted.');
