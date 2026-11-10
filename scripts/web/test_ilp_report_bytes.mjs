import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readIlpAuditFolder,ilpAuditRequest} from '../../web/lab/ilp-audit-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/ilp-report-bytes';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const p=JSON.parse(await fs.readFile('target/ilp-audit-scope-retention/browser.bisect','utf8')),f=p.files.ilpAuditInputs,bytes=s=>Buffer.from(s.base64,'base64'),report=bytes(f.reports['node.json']),model=bytes(f.models['node_root.lp']),summary=bytes(f.summary),bad=Buffer.from([255,192,0]);
const file=(name,b)=>({name,size:b.length,webkitRelativePath:'audit/'+name,arrayBuffer:async()=>b});
const inputs=await readIlpAuditFolder([file('node.json',report),file('node_old.json',bad),file('node_root.lp',model),file('audit-summary.json',summary)]);assert.equal(inputs.reports['node_old.json'].base64,bad.toString('base64'));
let cases=0,cli=0;
for(const mode of ['all-selected','summary-declared']){
 const request=ilpAuditRequest(inputs,{mode}),result=engine.execute(request),name=root+'/request.json';await fs.writeFile(name,JSON.stringify(request));const n=spawnSync('target/release/examples/execute_tool_request.exe',[name],{encoding:'utf8',maxBuffer:8*1024*1024});assert.equal(n.status,0,n.stderr);assert.deepEqual(result,JSON.parse(n.stdout));assert.equal(result.status,mode==='all-selected'?'failed':'verified');assert.equal(result.summary.checked,mode==='all-selected'?2:1);
 if(mode==='all-selected'){assert.equal(result.summary.failed,1);assert.equal(result.report_sha256['node_old.json'],createHash('sha256').update(bad).digest('hex'));assert(request.input.reports_base64);}else assert.equal(Object.hasOwn(request.input,'reports_base64'),false);
 const project=createProject({...p,files:{ilpAuditInputs:inputs,ilpAuditScopeSettings:{mode}},result});assert.deepEqual(parseProject(JSON.stringify(project)).files.ilpAuditInputs,JSON.parse(JSON.stringify(inputs)));if(mode==='all-selected')await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));cases++;
}
for(const [label,raw]of [['invalid-utf8',bad],['bom',Buffer.concat([Buffer.from([239,187,191]),report])],['empty',Buffer.alloc(0)]]){
 const request={operation:'audit-ilp-artifacts',input:{reports:{},reports_base64:{'node.json':raw.toString('base64')},models_base64:{'node_root.lp':model.toString('base64')},summary_json:null}},result=engine.execute(request),name=root+'/'+label+'.json';await fs.writeFile(name,JSON.stringify(request));const n=spawnSync('target/release/examples/execute_tool_request.exe',[name],{encoding:'utf8',maxBuffer:8*1024*1024});assert.equal(n.status,0,n.stderr);assert.deepEqual(result,JSON.parse(n.stdout));assert.equal(result.report_sha256['node.json'],createHash('sha256').update(raw).digest('hex'));
 const dir=await fs.mkdtemp(path.resolve(root,'cli-'+label+'-'));await fs.writeFile(dir+'/node.json',raw);await fs.writeFile(dir+'/node_root.lp',model);const actual=spawnSync('target/debug/bisect.exe',['ilp-audit',dir+'/node.json','--json'],{encoding:'utf8'});assert.equal(actual.status,result.native_exit_code,actual.stderr);assert.equal(JSON.parse(actual.stdout).checked,result.summary.checked);assert.equal(JSON.parse(actual.stdout).failed,result.summary.failed);cli++;cases++;
}
for(const input of [{reports:{'node.json':report.toString()},reports_base64:{'node.json':report.toString('base64')},models_base64:{},summary_json:null},{reports:{},reports_base64:{'node.json':'!!!'},models_base64:{},summary_json:null}])assert.throws(()=>engine.execute({operation:'audit-ilp-artifacts',input}));
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:cases,actual_cli_verdict_cases:cli,binary_reports_preserved_inertly:true,explicit_scope_excludes_invalid_stale_report:true,raw_report_hashes_exact:true,byte_encoding_refusals:2,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`ILP report bytes: ${cases} Native host/WASM cases, ${cli} actual CLI verdicts, two encoding refusals; raw hashes and scope/project preservation passed.`);
