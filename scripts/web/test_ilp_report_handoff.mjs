import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {ilpReportAuditProject} from '../../web/lab/ilp-report-handoff.js';
import {ilpAuditRequest,validateIlpAuditInputs} from '../../web/lab/ilp-audit-files.js';
import {parseProject} from '../../web/lab/project.js';
const root='target/ilp-report-handoff';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let cases=0,refusals=0;
for(const name of ['solved','failed']){
 const saved=JSON.parse(await fs.readFile('target/ilp-node-inspection/'+name+'.bisect','utf8'));
 const packet=engine.execute({operation:'inspect-native-ilp-run',source_json:Buffer.from(saved.files.ilpInspectionInputs.source.base64,'base64').toString('utf8')});
 const project=await ilpReportAuditProject(packet);assert.equal(project.operation,'audit-ilp-artifacts');assert.equal(project.result,null);assert.equal(project.lastOperation,null);assert.deepEqual(parseProject(JSON.stringify(project)),project);
 const request=ilpAuditRequest(project.files.ilpAuditInputs),result=engine.execute(request),requestFile=root+'/'+name+'.json';await fs.writeFile(requestFile,JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[requestFile],{encoding:'utf8'})));assert.equal(result.status,'verified');
 const cliDir=path.resolve(root,'cli-'+name);await fs.mkdir(cliDir,{recursive:true});for(const [filename,text]of Object.entries(packet.files)){await fs.mkdir(path.dirname(path.join(cliDir,filename)),{recursive:true});await fs.writeFile(path.join(cliDir,filename),text);}
 const cli=spawnSync(path.resolve('target/debug/bisect.exe'),['ilp-audit','--dir','.','--json'],{cwd:cliDir,encoding:'utf8'});assert.equal(cli.status,0,cli.stderr);const normalize=v=>({...v,records:v.records.map(r=>({...r,report:r.report.replaceAll('\\','/').replace(/^\.\//,''),model_path:r.model_path.replaceAll('\\','/').replace(/^\.\//,'')}))});assert.deepEqual(normalize(JSON.parse(cli.stdout)),normalize(result.summary));
 assert.deepEqual(Object.fromEntries(Object.entries(result.report_sha256)),Object.fromEntries(Object.entries(packet.file_sha256).filter(([p])=>p.endsWith('.json'))));
 for(const mutate of [p=>p.files[Object.keys(p.files)[0]]+=' ',p=>p.file_sha256[Object.keys(p.files)[0]]='0'.repeat(64),p=>p.files['../node.lp']='x',p=>p.nodes[0].path='../root',p=>p.nodes.push(structuredClone(p.nodes[0])),p=>p.nodes[0].lp_text+=' ',p=>delete p.files[Object.keys(p.files)[0]]]){const forged=structuredClone(packet);mutate(forged);await assert.rejects(()=>ilpReportAuditProject(forged));refusals++;}
 await fs.writeFile(root+'/'+name+'-audit.bisect',JSON.stringify(project));cases++;
}
// Exercise the raised artifact count with explicitly replicated native fixtures,
// not a claim that a 512-node engine run was performed.
const fixture='docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark',report=await fs.readFile(fixture+'/ilp-solve-report.json','utf8'),model=await fs.readFile(fixture+'/node_root.lp');
const inputs={reports:{},models:{},summary:null};for(let i=0;i<512;i++){inputs.reports[`${i}/report.json`]={name:'report.json',base64:Buffer.from(report).toString('base64')};inputs.models[`${i}/node_root.lp`]={name:'node_root.lp',base64:model.toString('base64')};}validateIlpAuditInputs(inputs);const request=ilpAuditRequest(inputs),result=engine.execute(request);assert.equal(result.summary.checked,512);assert.equal(result.summary.passed,512);
const file=root+'/512.json';await fs.writeFile(file,JSON.stringify(request));assert.deepEqual(result,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[file],{encoding:'utf8',maxBuffer:8*1024*1024})));
const project=await import('../../web/lab/project.js').then(m=>m.createProject({name:'Replicated native fixture artifact capacity',operation:'audit-ilp-artifacts',files:{ilpAuditInputs:inputs},constraints:[],result:null,lastOperation:null}));assert.deepEqual(parseProject(JSON.stringify(project)),project);
const oversized=structuredClone(inputs);oversized.reports['extra.json']={name:'extra.json',base64:Buffer.from(report).toString('base64')};assert.throws(()=>validateIlpAuditInputs(oversized));assert.throws(()=>engine.execute(ilpAuditRequest({...inputs,reports:inputs.reports,models:{...inputs.models,'extra.lp':{name:'extra.lp',base64:'eA=='}}})));refusals+=2;
const empty={schema_version:'bisect-native-ilp-run-reports-v1',nodes:[],files:{},file_sha256:{}};await assert.rejects(()=>ilpReportAuditProject(empty),/no solver artifacts/);refusals++;
await fs.writeFile(root+'/evidence.json',JSON.stringify({packet_native_host_wasm_and_cli_cases:cases,replicated_fixture_capacity:512,refusals,inert_source_preserving_projects:true,failed_dispatch_artifacts_audited_without_plan_certification:true,native_cli_sha256:createHash('sha256').update(await fs.readFile('target/debug/bisect.exe')).digest('hex'),wasm_sha256:engine.artifactIdentity.sha256},null,2));console.log(`ILP report handoff: ${cases} real node packets/native directory audits, 512 replicated fixture records, ${refusals} refusals; exact sources and inert audit projects passed.`);
