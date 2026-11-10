import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject} from '../../web/lab/project.js';
import {ilpAuditSummaryBytes} from '../../web/lab/ilp-audit-files.js';
const root='target/ilp-audit-scope-retention';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const p=JSON.parse(await fs.readFile('target/manifest-prechecks-wasm/current-scope-with-stale.bisect','utf8')),files=p.files.packageVerificationInputs.files,text=name=>Buffer.from(files[name].base64,'base64').toString('utf8'),reports={'node.json':text('nodes/node.json')},models_base64={'node_root.lp':files['nodes/node_root.lp'].base64},scoped=text('nodes/audit-summary.json');
let cases=0,cli=0;
for(const kind of ['legacy','scoped','extra','unsafe']){
 const input={reports:{...reports,...(kind==='extra'?{'node_old.json':text('nodes/node_old.json')}:{})},models_base64,summary_json:kind==='legacy'?JSON.stringify({...JSON.parse(scoped),report_selection:undefined}):kind==='unsafe'?JSON.stringify({...JSON.parse(scoped),report_selection:['../node.json']}):scoped};
 const req={operation:'audit-ilp-artifacts',input},out=engine.execute(req),file=root+'/request.json';await fs.writeFile(file,JSON.stringify(req));const n=spawnSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:8*1024*1024});assert.equal(n.status,0,n.stderr);assert.deepEqual(out,JSON.parse(n.stdout));assert.deepEqual(JSON.parse(new TextDecoder().decode(ilpAuditSummaryBytes(out))),out.summary);
 const matched=['legacy','scoped'].includes(kind);assert.equal(out.status,matched?'verified':'failed');assert.deepEqual(out.summary.report_selection,kind==='scoped'?['node.json']:undefined);
 const dir=await fs.mkdtemp(path.resolve(root,'cli-'+kind+'-'));for(const [name,value]of Object.entries(input.reports))await fs.writeFile(dir+'/'+name,value);await fs.writeFile(dir+'/node_root.lp',Buffer.from(models_base64['node_root.lp'],'base64'));await fs.writeFile(dir+'/saved-summary.json',input.summary_json);
 const process=spawnSync('target/debug/bisect.exe',['ilp-audit',...Object.keys(input.reports).map(name=>dir+'/'+name),'--verify-summary',dir+'/saved-summary.json','--out',dir+'/new-summary.json'],{encoding:'utf8'});assert.equal(process.status,matched?0:1,process.stderr);if(matched){const actual=JSON.parse(await fs.readFile(dir+'/new-summary.json','utf8'));assert.deepEqual(actual.report_selection,out.summary.report_selection);}cli++;
 if(kind==='scoped'){const sources={reports:{'node.json':{name:'node.json',base64:files['nodes/node.json'].base64}},models:{'node_root.lp':{name:'node_root.lp',base64:models_base64['node_root.lp']}},summary:{name:'audit-summary.json',base64:Buffer.from(scoped).toString('base64')}};const project=createProject({name:'Scoped ILP artifact audit',operation:'audit-ilp-artifacts',lastOperation:'audit-ilp-artifacts',files:{ilpAuditInputs:sources},constraints:[],result:out});await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/expected-summary.json',out.summary_json);}
 cases++;
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:cases,actual_cli_cases:cli,only_matching_scope_retained:true,summary_download_matches_displayed_records:true,legacy_unchanged:true,wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`ILP audit scope retention: ${cases} native/WASM and ${cli} actual CLI cases; matching scope retained in summary bytes, legacy unchanged, failed scope not borrowed.`);
