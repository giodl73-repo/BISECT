import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/ilp-audit-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture='docs/examples/rplan-benchmark-packages/U.16+branch-and-cut-path8-benchmark';
const report=await fs.readFile(fixture+'/ilp-solve-report.json','utf8'),model=await fs.readFile(fixture+'/node_root.lp');
const input={reports:{'node.json':report},models_base64:{'node_root.lp':model.toString('base64')},summary_json:null};
const sha=b=>createHash('sha256').update(b).digest('hex');
const requests=[];
function run(v){const request={operation:'audit-ilp-artifacts',input:v};const result=engine.execute(request);requests.push([request,result]);return result;}
const valid=run(input);assert.equal(valid.status,'verified');assert.equal(valid.summary.checked,1);assert.equal(valid.summary.passed,1);assert.equal(valid.model_sha256['node_root.lp'],sha(model));assert.equal(valid.report_sha256['node.json'],sha(report));assert.equal(valid.summary.records[0].proof_status,'proved-optimal');
const matched=run({...input,summary_json:valid.summary_json});assert.equal(matched.supplied_summary_status,'match');
const bad=run({...input,models_base64:{'node_root.lp':Buffer.from('changed model bytes').toString('base64')}});assert.equal(bad.status,'failed');assert.match(bad.summary.records[0].error,/hash mismatch/);
const stale=run({...input,models_base64:{'node_root.lp':Buffer.from('changed model bytes').toString('base64')},summary_json:valid.summary_json});assert.equal(stale.supplied_summary_status,'mismatch');
const missing=run({...input,models_base64:{}});assert.equal(missing.summary.failed,1);assert.match(missing.summary.records[0].error,/not selected/);
const malformed=run({...input,reports:{'node.json':'{bad'}});assert.equal(malformed.summary.records[0].outcome,undefined);assert.match(malformed.summary.records[0].error,/parse solve report/);
const noArtifact=JSON.parse(report);delete noArtifact.model_artifact;assert.match(run({...input,reports:{'node.json':JSON.stringify(noArtifact)}}).summary.records[0].error,/no model_artifact/);
for(const path of ['../outside.lp','/absolute.lp','C:/drive.lp','folder\\model.lp']){const v=JSON.parse(report);v.model_artifact.path=path;assert.equal(run({...input,reports:{'node.json':JSON.stringify(v)}}).status,'failed');}
const multi=run({...input,reports:{'node.json':report,'broken.json':'{bad'}});assert.equal(multi.summary.checked,2);assert.equal(multi.summary.failed,1);assert.equal(multi.summary.passed,1);
const fabricated=JSON.parse(report);fabricated.audit_summary.outcome='supplied-outcome-claim';assert.equal(run({...input,reports:{'node.json':JSON.stringify(fabricated)}}).summary.records[0].outcome,'supplied-outcome-claim');
const nested=run({...input,reports:{'nodes/node.json':report},models_base64:{'nodes/node_root.lp':model.toString('base64')}});assert.equal(nested.summary.records[0].model_path,'nodes/node_root.lp');
const modelAtLimit=Buffer.alloc(8*1024*1024,0xa5),limitReport=JSON.parse(report);limitReport.model_artifact.sha256=sha(modelAtLimit);assert.equal(run({...input,reports:{'node.json':JSON.stringify(limitReport)},models_base64:{'node_root.lp':modelAtLimit.toString('base64')}}).status,'verified');
let rejects=0;for(const v of [{...input,reports:{}},{...input,reports:{'../node.json':report}},{...input,reports:{'NODE.json':report,'node.json':report}},{...input,models_base64:{'node_root.lp':'%%%'}},{...input,reports:{'node.json':'x'.repeat(8*1024*1024+1)}},{...input,summary_json:'x'.repeat(1024*1024+1)},{...input,reports:Object.fromEntries(Array.from({length:513},(_,i)=>[`${i}.json`,report]))},{...input,models_base64:{'node_root.lp':Buffer.alloc(8*1024*1024+1).toString('base64')}},{...input,models_base64:{'node_root.lp':modelAtLimit.toString('base64'),'other.lp':modelAtLimit.toString('base64')}}]){assert.throws(()=>engine.execute({operation:'audit-ilp-artifacts',input:v}));rejects++;}
for(const [i,[req,out]]of requests.entries()){const path=`${root}/native-request-${i}.json`;await fs.writeFile(path,JSON.stringify(req));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[path],{encoding:'utf8',maxBuffer:8*1024*1024})),out);}
// Actual CLI reads the same selected bytes from a disposable fixture directory.
const cliDir=root+'/cli';await fs.mkdir(cliDir,{recursive:true});await fs.writeFile(cliDir+'/node.json',report);await fs.writeFile(cliDir+'/node_root.lp',model);
const cli=spawnSync('../../debug/bisect.exe',['ilp-audit','node.json','--json'],{cwd:cliDir,encoding:'utf8'});
assert.equal(cli.status,0,cli.stderr);assert.deepEqual(JSON.parse(cli.stdout),valid.summary);
await fs.writeFile(cliDir+'/audit-summary.json',valid.summary_json);
const matchedCli=spawnSync('../../debug/bisect.exe',['ilp-audit','node.json','--json','--verify-summary','audit-summary.json'],{cwd:cliDir,encoding:'utf8'});assert.equal(matchedCli.status,0,matchedCli.stderr);assert.deepEqual(JSON.parse(matchedCli.stdout),valid.summary);
await fs.writeFile(cliDir+'/node_root.lp','changed model bytes');
const badCli=spawnSync('../../debug/bisect.exe',['ilp-audit','node.json','--json'],{cwd:cliDir,encoding:'utf8'});assert.equal(badCli.status,1);assert.deepEqual(JSON.parse(badCli.stdout),bad.summary);
const staleCli=spawnSync('../../debug/bisect.exe',['ilp-audit','node.json','--verify-summary','audit-summary.json'],{cwd:cliDir,encoding:'utf8'});assert.equal(staleCli.status,1);assert.match(staleCli.stderr,/summary mismatch/);
await fs.writeFile(root+'/result.json',JSON.stringify(valid));await fs.writeFile(root+'/summary.json',valid.summary_json);await fs.writeFile(root+'/evidence.json',JSON.stringify({native_wasm_cases:requests.length,actual_cli_cases:4,rejections:rejects,stale_summary:true,report_claims_not_authenticated:true},null,2));
console.log(`ILP audit: ${requests.length} native/WASM cases, four actual native CLI cases, ${rejects} boundary rejects, missing/tampered/unsafe artifacts and stale summaries passed.`);
const helpers=await import('../../web/lab/ilp-audit-files.js');
const {createProject,parseProject}=await import('../../web/lab/project.js');
const src=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')});
const files={reports:{'node.json':src('node.json',report)},models:{'node_root.lp':src('node_root.lp',model)},summary:null};assert.deepEqual(helpers.ilpAuditRequest(files),{operation:'audit-ilp-artifacts',input});
assert.equal(Buffer.from(helpers.ilpAuditSummaryBytes(valid)).toString(),valid.summary_json);assert.equal(Buffer.from(helpers.ilpAuditSummaryBytes(bad)).toString(),bad.summary_json);
assert.throws(()=>helpers.ilpAuditSummaryBytes({...valid,summary:{...valid.summary,passed:0}}),/differ/);
const project=createProject({name:'Native ILP model artifact audit',files:{ilpAuditInputs:files},operation:'audit-ilp-artifacts',constraints:[],result:valid,lastOperation:'audit-ilp-artifacts'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/project.bisect',JSON.stringify(project));
const conflictFiles={reports:files.reports,models:{'node_root.lp':src('node_root.lp','changed model bytes')},summary:src('audit-summary.json',valid.summary_json)};
const conflictProject=createProject({name:'ILP model conflict diagnostics',files:{ilpAuditInputs:conflictFiles},operation:'audit-ilp-artifacts',constraints:[],result:stale,lastOperation:'audit-ilp-artifacts'});await fs.writeFile(root+'/conflict-project.bisect',JSON.stringify(conflictProject));await fs.writeFile(root+'/conflict-result.json',JSON.stringify(stale));
const selected=(name,bytes,relative)=>({name,size:bytes.length,webkitRelativePath:relative,arrayBuffer:async()=>Buffer.from(bytes)});
const folder=await helpers.readIlpAuditFolder([selected('node.json',Buffer.from(report),'audit/nodes/node.json'),selected('node_root.lp',model,'audit/nodes/node_root.lp')]);assert.equal(engine.execute(helpers.ilpAuditRequest(folder)).status,'verified');assert.ok(folder.models['nodes/node_root.lp']);
const withSummary=await helpers.readIlpAuditFolder([selected('node.json',Buffer.from(report),'audit/node.json'),selected('node_root.lp',model,'audit/node_root.lp'),selected('audit-summary.json',Buffer.from(valid.summary_json),'audit/audit-summary.json')]);assert.equal(engine.execute(helpers.ilpAuditRequest(withSummary)).supplied_summary_status,'match');
const binary=Uint8Array.from([0,255,128,10]),binaryReport=JSON.parse(report);binaryReport.model_artifact.sha256=sha(binary);const binarySources={reports:await helpers.readIlpAuditFiles([selected('node.json',Buffer.from(JSON.stringify(binaryReport)))],'reports'),models:await helpers.readIlpAuditFiles([selected('node_root.lp',binary)],'models'),summary:null};assert.equal(engine.execute(helpers.ilpAuditRequest(binarySources)).status,'verified');
const emptyModels=await helpers.readIlpAuditFiles([selected('empty.lp',new Uint8Array())],'models');assert.equal(emptyModels['empty.lp'].base64,'');
let read=false;await assert.rejects(()=>helpers.readIlpAuditFiles([{name:'big.lp',size:8*1024*1024+1,arrayBuffer:async()=>{read=true;}}],'models'));assert.equal(read,false);
await assert.rejects(()=>helpers.readIlpAuditFolder([selected('node.json',Buffer.from(report),'audit/node.json'),selected('NODE.json',Buffer.from(report),'audit/NODE.json')]),/colliding/);
const invalidUtf=await helpers.readIlpAuditFiles([selected('bad.json',Uint8Array.from([0xc3,0x28]))],'reports');assert.equal(engine.execute(helpers.ilpAuditRequest({reports:invalidUtf,models:{},summary:null})).status,'failed');
const malformedSource=await helpers.readIlpAuditFiles([selected('bad.json',Buffer.from('{bad'))],'reports');assert.equal(engine.execute(helpers.ilpAuditRequest({reports:malformedSource,models:{},summary:null})).status,'failed');
console.log('ILP helpers: exact binary/empty model and report bytes, nested folder/summary selection, native summary export and tamper guard, project roundtrip and pre-read/path rejection and native invalid-UTF-8 report diagnostics passed.');
