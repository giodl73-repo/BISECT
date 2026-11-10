// Test-only profile: these checks do not certify any jurisdiction's legal rules.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {preparedEngineAuditInputs} from '../../web/lab/prepared-engine-results.js';
import {textAuditRequest} from '../../web/lab/package-files.js';
const root='target/prepared-census-real-audit';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const profileTemplate=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
let matches=0;
async function parity(request){const actual=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));const expected=JSON.parse(execFileSync(path.resolve('target/release/examples/execute_tool_request.exe'),[path.resolve(root+'/request.json')],{encoding:'utf8',maxBuffer:128*1024*1024,stdio:['ignore','pipe','pipe']}));assert.deepEqual(actual,expected);matches++;return actual;}
for(const row of [{unit_kind:'county',units:5,tolerance_percent:50},{unit_kind:'block-group',units:792,tolerance_percent:0.5},{unit_kind:'block',units:25649,tolerance_percent:0.5}]){
 const output=JSON.parse(await fs.readFile('target/prepared-census-real/'+row.unit_kind+'-output.json','utf8'));
 assert.equal(output.source_preparation.units,row.units);
 const request={operation:'export-engine-plan',request:output.result.prepared.executed_request,assignments:output.result.prepared.result.assignments,...output.result.request_source.settings};delete request.inspect_only;
 const exported=await parity(request);assert.deepEqual(exported.context,output.result.prepared.export.context);assert.equal(exported.context_hash,output.result.prepared.export.context_hash);
 const result={...output.result.prepared,export:exported};
 const profile=structuredClone(profileTemplate);delete profile.legal_disclaimer;profile.profile_id='TEST_ONLY_ORIGINAL_RI';profile.jurisdiction='RI';profile.population_tolerance.max_deviation_ppm=Math.round(row.tolerance_percent*10000);profile.contiguity_required=true;profile.nesting_rule={type:'not-evaluated'};
 const settings={generated_at_utc:'2026-10-17T00:00:00Z'},constraints=['plan-shape','population','contiguity'];
 const files=preparedEngineAuditInputs(result,{auditProfileText:source('test-only-profile.json',JSON.stringify(profile))});
 const audit=await parity(textAuditRequest('audit-rplan-text',files,constraints,settings));
 const verified=await parity(textAuditRequest('verify-audit-text',{...files,auditCertificateText:source('test-only-certificate.json',audit.certificate_json)},constraints,settings));assert.equal(verified.verified,true);
 await fs.writeFile(root+'/'+row.unit_kind+'-certificate.json',audit.certificate_json);
 console.log(`Original RI ${row.unit_kind}: ${row.units} units, test-profile audit/certificate verification matches Native.`);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_host_comparisons:matches,resolutions:3,largest_units:25649,current_native_reexport_of_captured_runs:true,exact_native_plan_context_handoff:true,test_profile_only:true,does_not_certify_jurisdiction_rules:true},null,2));
