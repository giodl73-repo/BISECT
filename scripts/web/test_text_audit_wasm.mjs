import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {textAuditRequest} from '../../web/lab/package-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/text-audit-wasm';await fs.mkdir(root,{recursive:true});
const document_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8');
const context_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const profile=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;
delete profile.legal_disclaimer;profile.contiguity_required=true;profile.nesting_rule={type:'not-evaluated'};
const profile_json=JSON.stringify(profile),generated_at_utc='2026-10-04T00:00:00Z';
const base={operation:'audit-rplan-text',document_json,context_json,profile_json,constraints:['population','contiguity'],generated_at_utc,lineage_json:null};
let parity=0,rejections=0;
async function native(request){const path=root+'/request.json';await fs.writeFile(path,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
async function exact(request){const result=engine.execute(request);assert.deepEqual(result,await native(request));parity++;return result;}
async function reject(request){assert.throws(()=>engine.execute(request));await assert.rejects(()=>native(request));rejections++;}
let audited;
for(const constraints of [[],['population'],['contiguity'],['splits'],['vra'],['geometry'],['plan-shape','population','contiguity','splits','vra','geometry']]){
 const req={...base,constraints},out=await exact(req),certificate=JSON.parse(out.certificate_json);
 const typed=await exact({operation:'audit-plan',plan:JSON.parse(document_json).plan,context:JSON.parse(context_json),profile,constraints,generated_at_utc,lineage:null});
 assert.deepEqual(certificate,typed);
 const verification=await exact({operation:'verify-audit-text',certificate_json:out.certificate_json,document_json,context_json});assert.equal(verification.verified,true);assert.equal(verification.result,out.result);audited=out;
}
const missing=await exact({...base,context_json:null});assert.equal(missing.total_population,null);assert.ok(JSON.parse(missing.certificate_json).checks.some(c=>c.status==='missing-input'));
await exact({operation:'verify-audit-text',certificate_json:missing.certificate_json,document_json:null,context_json:null});
const canonical=v=>typeof v==='bigint'?v.toString():Array.isArray(v)?'['+v.map(canonical).join(',')+']':v&&typeof v==='object'?'{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}':JSON.stringify(v);
const ctx=JSON.parse(context_json);ctx.populations=Array(9).fill(9007199254740993n);ctx.context_hash='sha256:'+createHash('sha256').update(canonical({units:ctx.units,graph:ctx.graph,populations:ctx.populations,source_hashes:ctx.source_hashes})).digest('hex');
const large=await exact({...base,context_json:canonical(ctx)});assert.equal(large.total_population,'81064793292668937');assert.equal(large.result,'pass');await exact({operation:'verify-audit-text',certificate_json:large.certificate_json,document_json,context_json:canonical(ctx)});
ctx.populations[0]*=2n;ctx.context_hash='sha256:'+createHash('sha256').update(canonical({units:ctx.units,graph:ctx.graph,populations:ctx.populations,source_hashes:ctx.source_hashes})).digest('hex');
const failed=await exact({...base,context_json:canonical(ctx)});assert.equal(failed.result,'fail');assert.equal(failed.total_population,'90071992547409930');assert.ok(failed.certificate_json.includes('36028797018963972'));
const checkedFailed=await exact({operation:'verify-audit-text',certificate_json:failed.certificate_json,document_json,context_json:canonical(ctx)});assert.equal(checkedFailed.verified,true);assert.equal(checkedFailed.result,'fail');
const column=await exact({operation:'solve-branch-price-context',context_json,settings:{k:3,tolerance:0,formulation_only:false,exact_fixture_limit:12,label:'Lineage fixture',chamber:'congressional',created_at:generated_at_utc}});
const withLineage=await exact({...base,document_json:column.document_json,lineage_json:column.algorithm_lineage_json});
assert.equal(JSON.parse(withLineage.certificate_json).algorithm_lineage.parameters_hash,JSON.parse(column.algorithm_lineage_json).parameters_hash);
await exact({operation:'verify-audit-text',certificate_json:withLineage.certificate_json,document_json:column.document_json,context_json});
// JavaScript number normalization changes native canonical hashes (6.0 -> 6).
await reject({...base,lineage_json:JSON.stringify(JSON.parse(column.algorithm_lineage_json))});
const verify={operation:'verify-audit-text',certificate_json:audited.certificate_json,document_json,context_json};
for(const changed of [c=>c.certificate_id='sha256:wrong',c=>c.result=c.result==='fail'?'pass':'fail',c=>c.checks[0].summary='Changed']){const c=JSON.parse(audited.certificate_json);changed(c);await reject({...verify,certificate_json:JSON.stringify(c)});}
const wrongPlan=JSON.parse(document_json);wrongPlan.plan.assignment.reverse();await reject({...verify,document_json:JSON.stringify(wrongPlan)});
for(const req of [{...base,document_json:'{'},{...base,profile_json:'{'},{...base,generated_at_utc:''},{...base,generated_at_utc:'x'.repeat(201)},{...base,constraints:Array(7).fill('population')},{...base,constraints:['unknown']},{...base,document_json:'x'.repeat(3*1024*1024+1)},{...verify,context_json:null},{...verify,certificate_json:'x'.repeat(3*1024*1024+1)}])await reject(req);
for(const mutation of [c=>c.context_hash='sha256:stale',c=>c.populations[0]++,c=>c.units.unit_ids.reverse()]){const c=JSON.parse(context_json);mutation(c);await reject({...base,context_json:JSON.stringify(c)});}
const huge=JSON.parse(document_json);huge.plan.k=1e9;await reject({...base,document_json:JSON.stringify(huge)});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const files={auditPlanText:source('grid.rplan',document_json),auditContextText:source('grid.rctx',context_json),auditProfileText:source('profile.json',profile_json),textAuditSettings:{generated_at_utc}};
const request=textAuditRequest('audit-rplan-text',files,['population','contiguity'],files.textAuditSettings),result=await exact(request);
const project=createProject({name:'Synthetic exact file audit',files,operation:'audit-rplan-text',constraints:['population','contiguity'],result,lastOperation:'audit-rplan-text'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/browser-certificate.json',result.certificate_json);await fs.writeFile(root+'/profile.json',profile_json);
console.log(`Text plan audit WASM: ${parity} native/WASM matches, ${rejections} matching rejections; shared typed audit parity, exact large count witnesses, certificate bindings and byte-preserving project checks passed.`);
