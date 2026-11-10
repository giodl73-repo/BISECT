import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/ilp-current-scope-full-audit';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const p=JSON.parse(await fs.readFile('target/manifest-prechecks-wasm/current-scope-with-stale.bisect','utf8')),files=p.files.packageVerificationInputs.files,text=name=>Buffer.from(files[name].base64,'base64').toString('utf8');
for(const extra of [false,true]){
 const req={operation:'audit-ilp-artifacts',input:{reports:{'node.json':text('nodes/node.json'),...(extra?{'node_old.json':text('nodes/node_old.json')}:{})},models_base64:{'node_root.lp':files['nodes/node_root.lp'].base64},summary_json:text('nodes/audit-summary.json')}},out=engine.execute(req),file=root+'/request.json';await fs.writeFile(file,JSON.stringify(req));const n=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:8*1024*1024});assert.equal(n.status,0,n.stderr);assert.deepEqual(out,JSON.parse(n.stdout));assert.equal(out.status,extra?'failed':'verified');assert.equal(out.summary.checked,extra?2:1);if(extra)assert.match(out.supplied_summary_error,/selection differs/);
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:2,explicit_full_selection_keeps_extra_reports:true,scoped_summary_does_not_hide_explicit_audit_inputs:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log('ILP scope full-selection audit: two native/WASM cases; explicit extra reports remain audited and cannot be hidden by the summary scope.');
