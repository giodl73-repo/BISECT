import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
await fs.mkdir('target/report-side-fixtures',{recursive:true});
const requestPath='target/report-side-fixtures/request.json';
const bytes=text=>Array.from(Buffer.from(text,'utf8'));
const hash=bytes=>createHash('sha256').update(Buffer.from(bytes)).digest('hex');
const base={operation:'inspect-report-side',label:'fallback',leaning_threshold:0.55,files:{'manifest.json':bytes('{ "label":"<script>untrusted</script>", "num_districts":2, "submission_type":"civic_counter_proposal" }')}};
let exact=0,rejected=0,browserProject;
async function native(request){await fs.writeFile(requestPath,JSON.stringify(request));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[requestPath],{encoding:'utf8',maxBuffer:16*1024*1024}));}
for(const partisan of [null,'{"districts":[{"dem_share":0.55},{"dem_share":0.54}]}','{"per_district_dem_share":[0.55,0.54]}','[{"dem_share":0.55},{"dem_share":0.54}]'])for(const compactness of [null,'{"mean_polsby_popper":0.5}','{"districts":[{"polsby_popper":0.4},{"polsby_popper":0.6}]}']){
 const request=structuredClone(base);
 if(partisan)request.files['analysis/partisan.json']=bytes(partisan);
 if(compactness)request.files['analysis/compactness.json']=bytes(compactness);
 if(partisan)request.files['analysis/vra_analysis.json']=bytes('{"mm_count":1}');
 const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;
 const side=result.native_report_side;
 assert.equal(side.label,'<script>untrusted</script>');assert.equal(side.manifest_sha256,hash(request.files['manifest.json']));
 for(const [name,original]of Object.entries(request.files))if(name.startsWith('analysis/'))assert.equal(side.analysis_sha256[name.slice(9)],hash(original));
 assert.equal(side.leaning_seats,partisan?1:0);assert.equal(side.mm_count,partisan?1:0);assert.equal(side.mean_pp,compactness?0.5:null);
 assert.equal(result.usable_analysis.partisan,!!partisan);assert.equal(result.usable_analysis.demographic,!!partisan);assert.equal(result.usable_analysis.compactness,!!compactness);assert.equal(result.supplied_analysis.partisan,!!partisan);assert.equal(result.supplied_analysis.compactness,!!compactness);
 assert.match(result.source_boundary,/not recomputed or authenticated/);
 const project=createProject({name:'Native report inspection',files:{reportFiles:Object.fromEntries(Object.entries(request.files).map(([name,original])=>[name,Buffer.from(original).toString('base64')])),reportSettings:{label:request.label,leaning_threshold:request.leaning_threshold}},operation:'inspect-report-side',constraints:[],result,lastOperation:'inspect-report-side'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 const tables=countResultTables(result);assert.equal(tables[0].rows[0][1],side.label);assert.equal(tables[0].rows[3][1],partisan?'1':'Unavailable or incomplete');assert.equal(tables[0].rows[4][1],partisan?'1':'Unavailable or incomplete');assert.equal(tables[0].rows[5][1],compactness?'0.5':'Unavailable or incomplete');
 if(partisan&&compactness)browserProject=project;

}
// Presence alone does not establish a usable measurement: exercise the real
// native reader with empty objects and a partial district-share series.
for(const partisan of ['{}','{"per_district_dem_share":[0.55]}']){
 const request=structuredClone(base);
 request.files['analysis/partisan.json']=bytes(partisan);
 request.files['analysis/vra_analysis.json']=bytes('{}');
 request.files['analysis/compactness.json']=bytes('{}');
 const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;
 assert.deepEqual(result.supplied_analysis,{partisan:true,demographic:true,compactness:true});
 assert.deepEqual(result.usable_analysis,{partisan:false,demographic:false,compactness:false});
 const tables=countResultTables(result);
 for(const row of [3,4,5])assert.equal(tables[0].rows[row][1],'Unavailable or incomplete');
 const project=createProject({name:'Incomplete report inspection',files:{reportFiles:Object.fromEntries(Object.entries(request.files).map(([name,original])=>[name,Buffer.from(original).toString('base64')])),reportSettings:{label:request.label,leaning_threshold:request.leaning_threshold}},operation:'inspect-report-side',constraints:[],result,lastOperation:'inspect-report-side'});
 assert.deepEqual(parseProject(JSON.stringify(project)),project);
 await fs.writeFile('target/report-side-fixtures/browser-incomplete-project.bisect',JSON.stringify(project));
}
for(const [name,text,flag] of [
 ['analysis/partisan.json','{"per_district_dem_share":[0.55]}','partisan'],
 ['analysis/vra_analysis.json','{"mm_count":3}','demographic'],
 ['analysis/compactness.json','{"districts":[{"polsby_popper":0.4}]}','compactness'],
 ['analysis/compactness.json','{"mean_polsby_popper":2}','compactness'],
]){
 const request=structuredClone(base);request.files[name]=bytes(text);
 const result=engine.execute(request);assert.deepEqual(result,await native(request));exact++;
 assert.equal(result.usable_analysis[flag],false);
 const row={partisan:3,demographic:4,compactness:5}[flag];assert.equal(countResultTables(result)[0].rows[row][1],'Unavailable or incomplete');
}
const fallback=structuredClone(base);fallback.label='literal / label';fallback.files['manifest.json']=bytes('{"num_districts":2}');const fallbackResult=engine.execute(fallback);assert.deepEqual(fallbackResult,await native(fallback));assert.equal(fallbackResult.native_report_side.label,fallback.label);exact++;
for(const mutate of [r=>delete r.files['manifest.json'],r=>r.files['../manifest.json']=bytes('{}'),r=>r.files['unexpected.json']=bytes('{}'),r=>r.leaning_threshold=-0.1,r=>r.leaning_threshold=1.1,r=>r.label='x'.repeat(201),r=>r.files['manifest.json']=bytes('{"num_districts":0}'),r=>r.files['manifest.json']=bytes('{"num_districts":501}'),r=>r.files['manifest.json']=bytes('{"num_districts":"bad"}'),r=>r.files['manifest.json']=[255],r=>r.files['analysis/partisan.json']=bytes('{"districts":[{"dem_share":null}]}'),r=>r.files['analysis/partisan.json']=bytes(JSON.stringify({per_district_dem_share:Array(501).fill(0.5)})),r=>r.files['analysis/vra_analysis.json']=bytes('{"mm_count":-1}'),r=>r.files['analysis/compactness.json']=bytes('{"districts":[{"polsby_popper":null}]}'),r=>r.files['analysis/partisan.json']=Array(4*1024*1024+1).fill(32),r=>r.extra=true]){
 const request=structuredClone(base);mutate(request);assert.throws(()=>engine.execute(request));await fs.writeFile(requestPath,JSON.stringify(request));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',[requestPath],{stdio:'pipe'}));rejected++;
}
await fs.writeFile('target/report-side-fixtures/browser-project.bisect',JSON.stringify(browserProject));
await fs.writeFile('target/report-side-fixtures/verification.json',JSON.stringify({exact_native_wasm:exact,matching_rejections:rejected,scope:'Native imported report reader with project roundtrips and result-table availability checks; no analysis recomputation or authenticated provenance.'},null,2));
console.log(`Report side inspection: ${exact} exact native/WASM cases, ${rejected} matching rejections, original-byte hashes and explicit missing-analysis boundaries passed.`);
