import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {generatedReports,generatedReportText} from '../../web/lab/generated-report-files.js';
const root='target/generated-report-files';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile(process.env.BISECT_TEST_WASM||'target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const sha=text=>createHash('sha256').update(text).digest('hex');
const comparisonProject=JSON.parse(await fs.readFile('target/report-comparison-fixtures/browser-project.bisect','utf8'));
const comparison={operation:'render-report-comparison',...comparisonProject.files.reportComparisonSettings,...Object.fromEntries([['files_a','reportFiles'],['files_b','comparisonReportFiles']].map(([key,source])=>[key,Object.fromEntries(Object.entries(comparisonProject.files[source]).map(([p,b])=>[p,[...Buffer.from(b,'base64')]]))]))};await fs.writeFile(root+'/comparison-request.json',JSON.stringify(comparison));
const cases=[['commission','target/commission-report-wasm/request-0.json'],['label','target/label-report-wasm/native-request-1.json'],['years','target/label-report-years-wasm/html-request.json'],['years-json','target/label-report-years-wasm/json-request.json'],['workflow','target/state-workflow-wasm/browser-request.json'],['delivery-html','target/state-workflow-wasm/delivery-html-request.json'],['delivery-json','target/state-workflow-wasm/delivery-request.json'],['workflow-partial','target/state-workflow-wasm/missing-evidence-request.json'],['workflow-unavailable','target/state-workflow-wasm/bad-geometry-request.json'],['comparison',root+'/comparison-request.json']];
let nativeMatches=0,reports=0;const results={};
for(const [name,file]of cases){
 const request=JSON.parse(await fs.readFile(file,'utf8')),actual=engine.execute(request),native=JSON.parse(execFileSync(path.resolve(process.env.BISECT_TEST_NATIVE_TOOL||'target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(actual,native);nativeMatches++;results[name]=actual;
 const entries=generatedReports(actual);assert.deepEqual(entries,generatedReports(native));
 for(const report of entries){const bytes=await generatedReportText(actual,report.key);assert.equal(bytes.text,report.encoding==='text'?report.source:Buffer.from(report.source,'base64').toString());assert.equal(bytes.sha256,sha(bytes.text));reports++;}
 await fs.writeFile(`${root}/${name}.json`,JSON.stringify(actual));
}
assert.equal(generatedReports(results.years).length,2);assert.equal(generatedReports(results['years-json']).length,0);assert.equal(generatedReports(results['delivery-json']).length,0);assert.equal(generatedReports(results['workflow-unavailable']).length,0);assert.equal(generatedReports(results['workflow-partial']).length,1);
const batch={schema_version:'bisect-state-run-batch-v1',states:[{state_code:'RI',status:'ok',result:results.workflow},{state_code:'VT',status:'failed',result:results.workflow}]};assert.equal(generatedReports(batch).length,1);assert.equal((await generatedReportText(batch,generatedReports(batch)[0].key)).text,(await generatedReportText(results.workflow,generatedReports(results.workflow)[0].key)).text);
let guards=0;
const reject=async(result,key)=>{await assert.rejects(()=>generatedReportText(result,key));guards++;};
await reject(results.label,'missing');
for(const change of [r=>r.files[generatedReports(r)[0].path]+='changed',r=>delete r.file_sha256[generatedReports(r)[0].path],r=>r.file_sha256[generatedReports(r)[0].path]='z'.repeat(64),r=>{r.files['../unsafe.html']='bad';r.file_sha256['../unsafe.html']=sha('bad');},r=>r.files[generatedReports(r)[0].path]='x'.repeat(25*1024*1024+1)]){
 const r=structuredClone(results.label),key=generatedReports(r)[0].key;change(r);await reject(r,key);
}
for(const raw of ['%%%','Zh==',Buffer.from([255]).toString('base64')]){
 const r=structuredClone(results.workflow),key=generatedReports(r)[0].key,p=generatedReports(r)[0].path;r.workflow.files_base64[p]=raw;r.workflow.file_sha256[p]=sha(Buffer.from(raw,'base64'));await reject(r,key);
}
const legacy=structuredClone(results.commission);legacy.delivery_settings={label:'chosen',formats:['html'],audit_with_report:false};legacy.report_files={'chosen_report.html':legacy.report_html};legacy.report_file_sha256={'chosen_report.html':sha(legacy.report_html)};assert.equal((await generatedReportText(legacy,'commission')).text,results.commission.report_html);legacy.report_files['chosen_report.html']='other';await reject(legacy,'commission');
const duplicate=structuredClone(batch);duplicate.states.push(duplicate.states[0]);assert.throws(()=>generatedReports(duplicate));guards++;
assert.deepEqual(generatedReports({schema_version:'unknown',files:{'unsafe.html':'bad'}}),[]);
await fs.writeFile(`${root}/evidence.json`,JSON.stringify({wasm_sha256:engine.artifactIdentity.sha256,native_result_matches:nativeMatches,exact_report_sources:reports,guards,synthetic_native_reports:true,partial_success_reports_retained:true,json_only_and_failed_stages_unavailable:true,legacy_commission_supported:true,report_hashes_do_not_authenticate_sources:true},null,2));
console.log(`Generated report files: ${nativeMatches} complete Native/WASM matches, ${reports} exact HTML source/hash checks and ${guards} source/path/encoding/size refusals passed.`);
