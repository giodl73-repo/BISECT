import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const engine=await instantiateEngine(await fs.readFile(process.argv[2]||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/vra-audit-fixtures';await fs.mkdir(root,{recursive:true});let sequence=0,exact=0,rejections=0;
const document=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8')),context=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8')),original=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8'));
const baseProfile={...original.legal_profile,contiguity_required:true,nesting_rule:{type:'not-evaluated'}};delete baseProfile.legal_disclaimer;
async function native(request){const file=path.join(root,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));return spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:32*1024*1024});}
async function compare(request){const ref=await native(request);assert.equal(ref.status,0,ref.stderr);const result=engine.execute(request);assert.deepEqual(result,JSON.parse(ref.stdout));exact++;return result;}
async function reject(request){assert.throws(()=>engine.execute(request));assert.notEqual((await native(request)).status,0);rejections++;}
const patterns={weighted:[[100,50],[100,50],[100,50],[100,90],[100,90],[1000,100],[100,70],[100,70],[100,70]],zero:Array(9).fill([0,0]),boundary:Array(9).fill([100,50])};
let retained;
for(const basis of ['voting-age-population','citizen-voting-age-population'])for(const [pattern,counts]of Object.entries(patterns))for(const reverse of [false,true]){
 const rows=document.plan.units.unit_ids.map((id,i)=>`${id},${counts[i][0]},${counts[i][1]},"quoted, note"`);if(reverse)rows.reverse();
 const text=`GEOID,${basis==='voting-age-population'?'total_vap':'cvap'},minority_vap,note\r\n${rows.join('\r\n')}\r\n`,source_base64=Buffer.from(text).toString('base64'),source_label=`SYNTHETIC ${basis} ${pattern}`;
 const request={operation:'attach-demographic-csv',document,context,source_base64,basis,source_label},prepared=await compare(request);
 assert.deepEqual(prepared.context.demographics.total_vap,counts.map(c=>c[0]));assert.deepEqual(prepared.context.demographics.minority_vap,counts.map(c=>c[1]));assert.deepEqual(prepared.context.graph,context.graph);assert.deepEqual(prepared.context.populations,context.populations);assert.notEqual(prepared.context_hash,context.context_hash);assert.equal(prepared.context.source_hashes[`bisect.demographics.csv.${basis}`],`sha256:${createHash('sha256').update(Buffer.from(source_base64,'base64')).digest('hex')}`);
 const repeated=await compare({...request,context:prepared.context});assert.deepEqual(repeated.context,prepared.context,'Attachment is idempotent on the same source and unit order');
 for(const threshold of [0,0.5,1]){
  const profile={...baseProfile,vra_policy:{type:'report-opportunity-districts',minority_group:'Explicit synthetic group',vap_threshold:threshold}},auditRequest={operation:'audit-plan',plan:document.plan,context:prepared.context,profile,constraints:['plan-shape','population','contiguity','vra'],generated_at_utc:'2026-10-04T00:00:00Z',lineage:null};
  const certificate=await compare(auditRequest),vra=certificate.checks.find(c=>c.name==='vra');assert.equal(vra.status,'pass');assert.equal(vra.witnesses.length,3);
  for(const w of vra.witnesses){const group=counts.filter((_,i)=>document.plan.assignment[i]===w.district_id),total=group.reduce((s,c)=>s+c[0],0),minority=group.reduce((s,c)=>s+c[1],0);assert.equal(w.total_vap,total);assert.equal(w.minority_vap,minority);assert.equal(w.minority_vap_percent,total?minority/total*100:0);assert.equal(w.is_opportunity_district,total>0&&minority/total>threshold);}
  if(pattern==='weighted'&&threshold===0.5){assert.equal(vra.witnesses[0].is_opportunity_district,false,'Exactly threshold is not above threshold');assert.equal(vra.witnesses[1].is_opportunity_district,false,'Weighted count ratio, not unweighted tract mean');assert.equal(vra.witnesses[2].is_opportunity_district,true);}
  const verified=await compare({operation:'verify-certificate',certificate,plan:document.plan,context:prepared.context});assert.equal(verified.content_hash,certificate.content_hash);
  const snapshot=createProject({name:'Synthetic count-based VRA audit',files:{plan:document,context:prepared.context,profile,demographicSource:{name:'Synthetic counts.csv',base64:source_base64},demographicSettings:{basis,source_label}},operation:'audit-plan',constraints:['vra'],result:certificate,lastOperation:'audit-plan'}),reopened=parseProject(JSON.stringify(snapshot));assert.deepEqual(reopened.files.demographicSource.base64,source_base64);assert.deepEqual(reopened.files.context,prepared.context);assert.equal(countResultTables(certificate).find(t=>t.title==='VRA count witnesses').rows.length,3);
  await reject({operation:'verify-certificate',certificate,plan:document.plan,context});
  if(basis==='voting-age-population'&&pattern==='weighted'&&!reverse&&threshold===0.5)retained={prepared,certificate,request,auditRequest,project:reopened};
 }
}
const missing=await compare({...retained.auditRequest,context});assert.equal(missing.checks.find(c=>c.name==='vra').status,'missing-input');
const none=await compare({...retained.auditRequest,profile:baseProfile});assert.equal(none.checks.find(c=>c.name==='vra').status,'not-evaluated');
for(const change of [
 {basis:'total-population'},{source_base64:''},{source_base64:'not base64'},{source_label:'é'.repeat(101)},
 {source_base64:Buffer.from('GEOID,total_vap,minority_vap\n53001000100,100,50\n').toString('base64')},
 {source_base64:Buffer.from('GEOID,total_vap,minority_vap\n53001000100,100,50\n53001000100,100,50\n').toString('base64')},
 {source_base64:Buffer.from([0xff]).toString('base64')},
 {context:{...context,context_hash:'sha256:'+'0'.repeat(64)}},
])await reject({...retained.request,...change});
const short=structuredClone(retained.prepared.context);short.demographics.total_vap.pop();await reject({...retained.auditRequest,context:short});
const changed=structuredClone(retained.prepared.context);changed.demographics.minority_vap[0]++;await reject({...retained.auditRequest,context:changed});
const shortPlan=structuredClone(document.plan);shortPlan.assignment.pop();await reject({...retained.auditRequest,plan:shortPlan});
const zeroDistrictPlan=structuredClone(document.plan);zeroDistrictPlan.k=0;await reject({...retained.auditRequest,plan:zeroDistrictPlan});
const invalidDistrictPlan=structuredClone(document.plan);invalidDistrictPlan.assignment[0]=99;await reject({...retained.auditRequest,plan:invalidDistrictPlan});
const changedPlan=structuredClone(document.plan);changedPlan.units.state='RI';await reject({...retained.auditRequest,plan:changedPlan});
const altered=structuredClone(retained.certificate);altered.checks.find(c=>c.name==='vra').witnesses[0].minority_vap++;await reject({operation:'verify-certificate',certificate:altered,plan:document.plan,context:retained.prepared.context});
await fs.writeFile(path.join(root,'Synthetic-WA-VAP-counts.csv'),Buffer.from(retained.request.source_base64,'base64'));
await fs.writeFile(path.join(root,'prepared-context.rctx'),JSON.stringify(retained.prepared.context,null,2));await fs.writeFile(path.join(root,'profile.json'),JSON.stringify(retained.auditRequest.profile,null,2));await fs.writeFile(path.join(root,'audit-project.bisect'),JSON.stringify(retained.project));
await fs.writeFile('target/vra-audit-verification.json',JSON.stringify({exact_native_wasm_cases:exact,rejections,zero_and_strict_threshold_cases:true,weighted_count_not_tract_mean:true,raw_source_and_context_project_retention:true,certificate_replay:true,synthetic_demographics:true},null,2));
console.log(`VRA count auditing: ${exact} exact native/WASM preparation/audit/verification cases, ${rejections} malformed/stale/tampered input or certificate rejections; VAP/CVAP, CSV order, weighted ratios, strict threshold, zero counts, missing inputs and not-evaluated policy, retained original bytes and project/certificate roundtrips passed.`);
