import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root='target/commission-report-wasm';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
let exact=0,rejects=0;
for(let n=0;n<9;n++){
 const request=JSON.parse(await fs.readFile(`${root}/request-${n}.json`,'utf8'));
 const expected=await fs.readFile(`${root}/expected-${n}.json`,'utf8');
 const result=engine.execute(request);assert.equal(result.report_json,expected);assert.equal(result.report_html,await fs.readFile(`${root}/expected-${n}.html`,'utf8'));assert.equal((result.report_html.match(/<h2>/g)||[]).length,10);if(n===8){assert.ok(!result.report_html.includes('<script>'));assert.ok(!result.report_html.includes('<img src=x'));assert.match(result.report_html,/&lt;script&gt;/);}assert.equal(result.schema_version,'bisect-commission-report-v1');
 for(const [path,raw]of Object.entries(request.files))assert.equal(result.source_sha256[path],createHash('sha256').update(Buffer.from(raw)).digest('hex'));
 assert.match(result.report_json,/9007199254740993/);assert.equal(Object.keys(JSON.parse(result.report_json).sections).length,10);
 assert.match(result.source_boundary,/not recomputed or authenticated/);exact++;
}
const base=JSON.parse(await fs.readFile(`${root}/request-4.json`,'utf8'));
const bytes=t=>Array.from(Buffer.from(t));
const mutations=[r=>delete r.files['analysis/summary.json'],r=>r.files['analysis/compactness.json']=bytes('{bad'),r=>r.files['analysis/partisan.json']=[255],r=>delete r.files['context.rctx'],r=>delete r.files['audit/certificate.json'],r=>r.files['audit/certificate.json']=bytes('{bad'),r=>r.files['context.rctx']=bytes('{}'),r=>r.files['../evil']=[],r=>r.files['other.json']=bytes('{}'),r=>r.generated_at='',r=>r.extra=true,r=>{let m=JSON.parse(Buffer.from(r.files['manifest.json']).toString());m.rctx_path='C:\\secret';r.files['manifest.json']=bytes(JSON.stringify(m));}];
for(const mutate of mutations){const r=structuredClone(base);mutate(r);assert.throws(()=>engine.execute(r));rejects++;}
await fs.writeFile(`${root}/evidence.json`,JSON.stringify({exact_filesystem_native_reports:exact,rejects,ten_sections:true,exact_i64_json_text:true,source_hashes:true,ui_exposure:false,exact_native_html:true,escaped_supplied_text:true,pdf:false},null,2));
console.log(`Commission report WASM: ${exact} exact filesystem-native ten-section JSON and HTML reports, original-byte hashes, exact i64 text and ${rejects} rejects passed.`);
