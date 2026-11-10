import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/commission-fallback-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const cli=path.resolve(process.platform==='win32'?'target/debug/bisect.exe':'target/debug/bisect');
const fixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8'));
const cases=[['missing',null],['malformed',Buffer.from('{bad')],['utf8',Buffer.from([255])],['typed',Buffer.from('{"seed":"wrong"}')],['empty',Buffer.alloc(0)],['typed-refs',Buffer.from('{"seed":"wrong","rctx_path":"obsolete.rctx","audit_certificate_path":"obsolete-certificate.json"}')],['supplied',Buffer.from(Buffer.from(fixture.files['manifest.json']).toString().replace('report_0','recorded-label').replace('2020','2010'))]];
let full=0,audits=0;
for(const [name,raw]of cases){
 const base=`${root}/${name}`,label=`fallback-${name}`,year='2020',plan=`${base}/v1/${year}/plans/${label}`,out=`${base}/report`,auditOut=`${base}/audit`;await fs.mkdir(plan,{recursive:true});
 const files=structuredClone(fixture.files);delete files['manifest.json'];if(raw!==null)files['manifest.json']=Array.from(raw);if(name==='typed-refs'){files['obsolete.rctx']=[255];files['obsolete-certificate.json']=[255];}
 for(const [p,b]of Object.entries(files)){await fs.mkdir(path.dirname(`${plan}/${p}`),{recursive:true});await fs.writeFile(`${plan}/${p}`,Buffer.from(b));}
 const args=['report','--label',label,'--year',year,'--output-base',base];
 let run=spawnSync(cli,[...args,'--out',out,'--format','html','json','--audit-with-report'],{encoding:'utf8'});assert.equal(run.status,0,run.stderr);
 const expected=await fs.readFile(`${out}/${label}_report.json`,'utf8'),nativeClock=JSON.parse(expected).generated_at;
 const request={operation:'assemble-commission-report',files,generated_at:fixture.generated_at,fallback:{label,year}},result=engine.execute(request);
 assert.equal(result.report_json,expected.replace(nativeClock,fixture.generated_at));assert.equal(result.report_html,(await fs.readFile(`${out}/${label}_report.html`,'utf8')).replaceAll(nativeClock,fixture.generated_at));assert.equal(result.audit_json,await fs.readFile(`${out}/audit.json`,'utf8'));
 assert.deepEqual(result.manifest_resolution,{fallback_used:name!=='supplied',source_status:name==='missing'?'missing':name==='supplied'?'supplied':'invalid'});
 if(raw){assert.equal(result.source_sha256['manifest.json'],createHash('sha256').update(raw).digest('hex'));}else assert.equal(result.source_sha256['manifest.json'],undefined);
 assert.match(countResultTables(result)[0].rows.find(r=>r[0]==='Manifest resolution')[1],name==='supplied'?/Supplied metadata/:/Native minimal fallback/);
 assert.equal(result.label,name==='supplied'?'recorded-label':label);assert.equal(result.year,name==='supplied'?'2010':year);full++;
 run=spawnSync(cli,[...args,'--out',auditOut,'--audit-only'],{encoding:'utf8'});assert.equal(run.status,0,run.stderr);
 const audit=engine.execute({operation:'assemble-commission-audit',...(raw?{manifest_bytes:Array.from(raw)}:{}),fallback:{label,year}});assert.equal(audit.audit_json,await fs.readFile(`${auditOut}/audit.json`,'utf8'));assert.deepEqual(audit.manifest_resolution,result.manifest_resolution);assert.equal(audit.manifest_sha256,raw?createHash('sha256').update(raw).digest('hex'):null);audits++;
 await fs.writeFile(`${root}/${name}-result.json`,JSON.stringify(result));
 if(name!=='supplied'){assert.throws(()=>engine.execute({...request,fallback:undefined}));assert.throws(()=>engine.execute({operation:'assemble-commission-audit',...(raw?{manifest_bytes:Array.from(raw)}:{})}));}
}
const missingAnalysis=structuredClone(fixture);delete missingAnalysis.files['manifest.json'];delete missingAnalysis.files['analysis/compactness.json'];assert.throws(()=>engine.execute({...missingAnalysis,fallback:{label:'legacy',year:'2020'}}));
const base={operation:'assemble-commission-audit',fallback:{label:'x',year:'2020'}};
let rejects=0;for(const request of [{...base,fallback:{...base.fallback,extra:1}},{...base,fallback:{label:'x'.repeat(201),year:'2020'}},{...base,manifest_json:'{}',manifest_bytes:[123,125]},{...base,manifest_bytes:Array(16*1024*1024+1).fill(0)}]){assert.throws(()=>engine.execute(request));rejects++;}
const context=JSON.parse(await fs.readFile('target/commission-report-wasm/request-4.json','utf8'));context.files['context.rctx']=Array.from(Buffer.from('{}'));assert.throws(()=>engine.execute({...context,fallback:base.fallback}));
await fs.writeFile(`${root}/evidence.json`,JSON.stringify({actual_cli_full_reports:full,actual_cli_audit_only:audits,exact_json_html_companion:true,only_native_clock_normalized:true,missing_malformed_utf8_typed:true,empty_manifest:true,obsolete_references_ignored:true,valid_manifest_precedence:true,exact_raw_source_hashes:true,fallback_opt_in:true,context_error_not_masked:true,required_analysis_not_masked:true,fallback_status_table:true,rejects,ui_exposure:false},null,2));
console.log(`Commission fallback: ${full} actual CLI full JSON/HTML/companion reports and ${audits} actual CLI audit-only documents match, original corrupt bytes hashed, valid metadata takes precedence and ${rejects} request bounds reject.`);
