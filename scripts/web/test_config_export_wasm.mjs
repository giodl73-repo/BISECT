import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {configuredRunExportRequest} from '../../web/lab/config-run.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const graph=JSON.parse(await fs.readFile('target/optional-seed-wasm/case-0.json','utf8')).graph;
const dir='target/config-export-wasm';await fs.mkdir(dir,{recursive:true});let cases=0,rejections=0;
async function parity(request){const path=dir+'/request.json';await fs.writeFile(path,JSON.stringify(request));const native=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));const result=engine.execute(request);assert.deepEqual(result,native);cases++;return result;}
async function reject(request){assert.throws(()=>engine.execute(request));const path=dir+'/bad.json';await fs.writeFile(path,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[path],{stdio:'pipe'}));rejections++;}
const fixture=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
let first;
for(const seed of [null,'0','42','18446744073709551615']){
 const yaml_text='name: exported\r\nalgorithm:\r\n  structure: standard-bisect\r\n  engine: metis-core\r\n  search: single\r\n'+(seed===null?'':`  seed: ${seed}\r\n`);
 const input={graph,yaml_text,source_name:'exported.yml',profile:'open',chamber:'house',districts:2};
 const run=await parity({operation:'run-algorithm-config',input});
 const request=configuredRunExportRequest(run,{label:'Configured plan',created_at:'2026-10-07T00:00:00Z'});
 assert.deepEqual(request.input,input);assert.deepEqual(request.assignments,run.result.assignments);
 const exported=await parity(request);first??={run,request,exported};
 assert.equal(exported.schema_version,'bisect-configured-plan-export-v1');
 assert.deepEqual(exported.document.plan.assignment,graph.geoids.map(id=>run.result.assignments[id]-1));
 assert.deepEqual(exported.context.populations,graph.population);
 assert.deepEqual(exported.document.plan.units.unit_ids,graph.geoids);
 const c=exported.document.provenance.producer.algorithm_configuration;
 assert.equal(c.yaml_text,yaml_text);assert.equal(c.seed,seed);assert.equal(c.execution_tolerance_percent,5);
 assert.equal(exported.context.source_hashes['bisect.algorithm-yaml'],run.configuration.source_sha256);
 assert.equal(exported.context.source_hashes['bisect.embedded-location-policy'],run.configuration.embedded_policy_sha256);
 assert.equal(exported.document.provenance.producer.engine_options.seed==null?null:String(exported.document.provenance.producer.engine_options.seed),seed);
 const valid=await parity({operation:'validate-rplan-text',document_json:exported.document_json});assert.equal(valid.plan_hash,exported.plan_hash);
 const profile={...fixture.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};
 const audited=await parity({operation:'audit-rplan-text',document_json:exported.document_json,context_json:exported.context_json,profile_json:JSON.stringify(profile),constraints:['plan-shape','population','contiguity'],generated_at_utc:'2026-10-07T00:00:00Z',lineage_json:null});
 assert.equal(audited.result,'pass');assert.equal(audited.context_hash,exported.context_hash);
 const verified=await parity({operation:'verify-audit-text',certificate_json:audited.certificate_json,document_json:exported.document_json,context_json:exported.context_json});assert.equal(verified.verified,true);
 const project=createProject({name:'Exported configured run',files:{},operation:'run-algorithm-config',constraints:[],result:{...run,practitioner_export:exported},lastOperation:'run-algorithm-config'});
 assert.deepEqual(parseProject(JSON.stringify(project)).result,project.result);
 await fs.writeFile(`${dir}/seed-${seed??'none'}.rplan`,exported.document_json);await fs.writeFile(`${dir}/seed-${seed??'none'}.rctx`,exported.context_json);
}
const modified=structuredClone(first.request);modified.input.yaml_text+='\r\n# retained source comment\r\n';const changed=await parity(modified);
assert.equal(changed.plan_hash,first.exported.plan_hash);assert.notEqual(changed.context_hash,first.exported.context_hash);
const explicit=structuredClone(first.request);explicit.input.yaml_text+='  balance_tolerance: 5\r\n';const explicitExport=await parity(explicit);assert.equal(explicitExport.context.source_hashes['bisect.embedded-location-policy'],undefined);
// Chamber names also support custom jurisdictions; the boundary is UTF-8 bytes.
const customChambers=['oops','Community council','Conseil régional','x'.repeat(200),'é'.repeat(100)];
for(const chamber of customChambers){const selected=structuredClone(first.request);selected.input.chamber=chamber;const exported=await parity(selected);assert.equal(exported.configuration.execution_chamber,chamber);assert.equal(exported.document.metadata.chamber,chamber);assert.deepEqual(exported.document.plan.assignment,first.exported.document.plan.assignment);}
for(const mutate of [r=>delete r.assignments[graph.geoids[0]],r=>r.assignments.extra=1,r=>r.assignments[graph.geoids[0]]=0,r=>r.label='',r=>r.input.unknown=true,r=>r.input.profile='oops',r=>r.input.chamber='',r=>r.input.chamber='bad\nname',r=>r.input.chamber='x'.repeat(201),r=>r.input.chamber='é'.repeat(101)]){const bad=structuredClone(first.request);mutate(bad);await reject(bad);}
await fs.writeFile(dir+'/evidence.json',JSON.stringify({cases,rejections,custom_chamber_exports:customChambers.length,custom_chamber_utf8_boundary:true,exact_native_wasm:true,optional_u64_seed:true,source_binding:true,native_text_audit_and_verify:true,export_is_unsigned:true},null,2));
console.log(`Configured native export: ${cases} native/WASM matches, ${rejections} matching rejections; YAML/optional-u64 seed, source-bound context, native text validation/audit/verification and project retention passed.`);
