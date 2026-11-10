import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {reportDownload} from '../../web/lab/report-output.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
await fs.mkdir('target/report-comparison-fixtures',{recursive:true});
const requestPath='target/report-comparison-fixtures/request.json';
const bytes=s=>Array.from(Buffer.from(s,'utf8'));
const sha=s=>createHash('sha256').update(s).digest('hex');
function files(mask,label){
 const result={'manifest.json':bytes(JSON.stringify({label,num_districts:2,submission_type:'civic_counter_proposal',submitted_by:'<img src=x onerror=alert(1)>',submitted_at:'supplied date'}))};
 if(mask&1)result['analysis/partisan.json']=bytes('{"per_district_dem_share":[0.55,0.54]}');
 if(mask&2)result['analysis/vra_analysis.json']=bytes('{"mm_count":1}');
 if(mask&4)result['analysis/compactness.json']=bytes('{"districts":[{"polsby_popper":0.4},{"polsby_popper":0.6}]}');
 return result;
}
const base={operation:'render-report-comparison',files_a:files(7,'<script>untrusted</script>'),files_b:files(7,'Other & plan'),label_a:'fallback A',label_b:'fallback B',leaning_threshold:0.55,close_call_band:0.02,generated_at:'2026-10-04T00:00:00Z'};
async function native(request){await fs.writeFile(requestPath,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[requestPath],{encoding:'utf8',stdio:['ignore','pipe','pipe'],maxBuffer:16*1024*1024}));}
let exact=0,rejected=0;
for(let a=0;a<8;a++)for(let b=0;b<8;b++){
 const request={...base,files_a:files(a,'<script>untrusted</script>'),files_b:files(b,'Other & plan')};
 const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;
 assert.equal(result.draft,true);assert.equal(result.receipt.html_sha256,sha(result.html));assert.equal(result.receipt.markdown_sha256,sha(result.markdown));
 for(const [side,mask]of [['plan_a',a],['plan_b',b]]){
  assert.deepEqual(result.availability[side],{partisan:!!(mask&1),demographic:!!(mask&2),compactness:!!(mask&4)});
  assert.equal(result.receipt[side+'_manifest_sha256'],sha(Buffer.from(request['files_'+(side==='plan_a'?'a':'b')]['manifest.json'])));
 }
 assert.ok(!/<script|<img|<iframe/i.test(result.html));assert.match(result.html,/&lt;script&gt;untrusted&lt;\/script&gt;/);assert.match(result.html,/&lt;img src=x onerror=alert\(1\)&gt;/);
 assert.ok(!/<img|<script/i.test(result.markdown));assert.match(result.html,/code>&lt;script&gt;untrusted&lt;\/script&gt;<\/code>/);assert.ok(!result.html.includes('&amp;lt;img'));assert.match(result.html,/DRAFT/);assert.ok(!result.html.includes('APPROVED'));assert.match(result.html,/Assignment differences/);assert.ok(!result.markdown.includes('reassigns 0 tracts'));assert.match(result.markdown,/population totals are unavailable/);
 for(const [bit,phrase,unavailable]of [[1,'Democratic-leaning seats','Partisan comparison unavailable'],[2,'Majority-minority districts','Majority-minority comparison unavailable'],[4,'Compactness (Polsby-Popper','Compactness comparison unavailable']]){
  if((a&bit)&&(b&bit)){assert.ok(result.markdown.includes(phrase));assert.ok(!result.markdown.includes(unavailable));}else assert.ok(result.markdown.includes(unavailable));
 }
 const project=createProject({name:'Imported comparison report',files:{reportFiles:Object.fromEntries(Object.entries(request.files_a).map(([name,bytes])=>[name,Buffer.from(bytes).toString('base64')])),comparisonReportFiles:Object.fromEntries(Object.entries(request.files_b).map(([name,bytes])=>[name,Buffer.from(bytes).toString('base64')])),reportComparisonSettings:{label_a:request.label_a,label_b:request.label_b,leaning_threshold:request.leaning_threshold,close_call_band:request.close_call_band,generated_at:request.generated_at}},operation:'render-report-comparison',constraints:[],result,lastOperation:'render-report-comparison'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);
 const tables=countResultTables(result);assert.equal(tables[0].rows[0][1],result.report.plan_a.label);assert.equal(tables[0].rows[2][1],(a&1)?'1':'Unavailable or incomplete');assert.equal(tables[0].rows[2][2],(b&1)?'1':'Unavailable or incomplete');
 if(a===7&&b===7){await fs.writeFile('target/report-comparison-fixtures/browser-project.bisect',JSON.stringify(project));for(const kind of ['html','markdown','receipt']){const file=await reportDownload(result,kind);assert.equal(Buffer.from(file.bytes).toString('utf8'),kind==='receipt'?JSON.stringify(result.receipt,null,2):result[kind]);}await fs.writeFile('target/report-comparison-fixtures/report.html',result.html);await fs.writeFile('target/report-comparison-fixtures/report.md',result.markdown);await fs.writeFile('target/report-comparison-fixtures/report-receipt.json',JSON.stringify(result.receipt,null,2));}
}
// Present-but-incomplete or physically impossible values cannot produce claims.
for(const mutate of [r=>r.files_a['analysis/partisan.json']=bytes('{"per_district_dem_share":[0.55]}'),r=>r.files_a['analysis/vra_analysis.json']=bytes('{"mm_count":3}'),r=>r.files_a['analysis/compactness.json']=bytes('{"districts":[{"polsby_popper":0.4}]}'),r=>r.files_a['analysis/compactness.json']=bytes('{"mean_polsby_popper":2}')]){
 const request=structuredClone(base);mutate(request);const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;assert.ok(Object.values(result.availability.plan_a).includes(false));assert.match(result.html,/Unavailable or incomplete/);
}
for(const mutate of [r=>delete r.files_a['manifest.json'],r=>delete r.files_b['manifest.json'],r=>r.files_a['../manifest.json']=bytes('{}'),r=>r.files_b['other.html']=bytes('<script/>'),r=>r.label_a='x'.repeat(201),r=>r.label_b='x'.repeat(201),r=>r.leaning_threshold=-1,r=>r.leaning_threshold=2,r=>r.close_call_band=-1,r=>r.close_call_band=2,r=>r.generated_at='x'.repeat(101),r=>r.approved_by='untrusted',r=>r.files_a['manifest.json']=[255],r=>r.files_a['manifest.json']=bytes(JSON.stringify({num_districts:2,label:'x'.repeat(201)})),r=>r.files_b['manifest.json']=bytes(JSON.stringify({num_districts:2,submitted_by:'x'.repeat(201)})),r=>r.files_b['manifest.json']=bytes(JSON.stringify({num_districts:2,submitted_at:'x'.repeat(201)}))]){
 const request=structuredClone(base);mutate(request);assert.throws(()=>engine.execute(request));await assert.rejects(native(request));rejected++;
}
await fs.writeFile('target/report-comparison-fixtures/verification.json',JSON.stringify({exact_native_wasm:exact,matching_rejections:rejected,project_table_roundtrips:64,exact_download_payloads:3,scope:'Shared draft imported-analysis renderer: output byte hashes, unavailable claims, escaped HTML and civic framing. No analysis recomputation or authenticated provenance.'},null,2));
console.log(`Imported comparison reports: ${exact} exact native/WASM cases, ${rejected} matching rejections; escaped HTML, missing/incomplete claims and independent output SHA-256 checks passed.`);
