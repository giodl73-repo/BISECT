import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {ilpAuditRequest,ilpAuditScopeSelection} from '../../web/lab/ilp-audit-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/ilp-explicit-scope-ui';await fs.mkdir(root,{recursive:true});
const prior=JSON.parse(await fs.readFile('target/ilp-audit-scope-retention/browser.bisect','utf8')),inputs=structuredClone(prior.files.ilpAuditInputs);inputs.reports['node_old.json']=structuredClone(inputs.reports['node.json']);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let cases=0,refusals=0;
for(const mode of ['all-selected','summary-declared']){
 const settings={mode},before=JSON.stringify(inputs),request=ilpAuditRequest(inputs,settings),result=engine.execute(request);assert.equal(JSON.stringify(inputs),before);assert.equal(result.status,mode==='all-selected'?'failed':'verified');assert.equal(result.summary.checked,mode==='all-selected'?2:1);
 const file=root+'/request.json';await fs.writeFile(file,JSON.stringify(request));const native=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:8*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(result,JSON.parse(native.stdout));
 const project=createProject({...prior,files:{ilpAuditInputs:inputs,ilpAuditScopeSettings:settings},result});assert.deepEqual(parseProject(JSON.stringify(project)).files.ilpAuditInputs,inputs);if(mode==='all-selected')await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));cases++;
}
assert.deepEqual(ilpAuditScopeSelection(inputs,{mode:'summary-declared'}).excluded,['node_old.json']);
for(const selection of [[],['../node.json'],['node.json','node.json'],['missing.json'],['node_old.json','node.json']]){
 const v=structuredClone(inputs),summary=JSON.parse(Buffer.from(v.summary.base64,'base64'));summary.report_selection=selection;summary.checked=selection.length;v.summary.base64=Buffer.from(JSON.stringify(summary)).toString('base64');assert.throws(()=>ilpAuditRequest(v,{mode:'summary-declared'}));refusals++;
}
const missing=structuredClone(inputs);missing.summary=null;assert.throws(()=>ilpAuditRequest(missing,{mode:'summary-declared'}));refusals++;
const invalid=createProject({...prior,files:{ilpAuditInputs:inputs},result:null});invalid.files.ilpAuditScopeSettings={mode:'unknown'};assert.throws(()=>parseProject(JSON.stringify(invalid)));refusals++;
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_scope_requests:cases,invalid_scope_refusals:refusals,full_inventory_preserved:true,default_audits_all_selected:true,project_roundtrips:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`Explicit ILP scope: ${cases} native/WASM requests, ${refusals} invalid scope refusals; original inventory preserved in both project modes.`);
