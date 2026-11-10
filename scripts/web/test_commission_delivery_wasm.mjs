import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {commissionReportZip,commissionFormatAvailable,validateCommissionDeliverySettings} from '../../web/lab/commission-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/commission-delivery-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const cli=path.resolve(process.platform==='win32'?'target/debug/bisect.exe':'target/debug/bisect');
const fixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8'));
let exact=0;
for(const formats of [['html'],['json'],['html','json'],['json','html','json']])for(const audit_with_report of [false,true]){
 const n=exact,label=`delivery-${n}`,base=`${root}/case-${n}`,plan=`${base}/v1/2020/plans/${label}`,out=`${base}/reports`;await fs.mkdir(plan,{recursive:true});
 for(const [p,bytes]of Object.entries(fixture.files)){await fs.mkdir(path.dirname(`${plan}/${p}`),{recursive:true});await fs.writeFile(`${plan}/${p}`,Buffer.from(bytes));}
 const args=['report','--label',label,'--output-base',base,'--out',out,'--format',...formats];if(audit_with_report)args.push('--audit-with-report');const native=spawnSync(cli,args,{encoding:'utf8'});assert.equal(native.status,0,native.stderr);
 const clock=JSON.parse(formats.includes('json')?await fs.readFile(`${out}/${label}_report.json`,'utf8'):JSON.stringify({generated_at:(await fs.readFile(`${out}/${label}_report.html`,'utf8')).match(/<strong>Generated:<\/strong> ([^<]+)/)[1].trim()})).generated_at;
 const delivery={label,formats,audit_with_report},result=engine.execute({...fixture,delivery});const names=(await fs.readdir(out)).sort();assert.deepEqual(Object.keys(result.report_files).sort(),names);
 for(const name of names){const raw=await fs.readFile(`${out}/${name}`,'utf8'),expected=name==='audit.json'?raw:raw.replaceAll(clock,fixture.generated_at);assert.equal(result.report_files[name],expected);assert.equal(result.report_file_sha256[name],createHash('sha256').update(expected).digest('hex'));}
 assert.equal(commissionFormatAvailable(result,'html'),formats.includes('html'));assert.equal(commissionFormatAvailable(result,'json'),formats.includes('json'));assert.equal(commissionFormatAvailable(result,'audit'),audit_with_report);if(!formats.includes('html'))assert.equal(result.report_html,null);
 const zip=commissionReportZip(result);await fs.writeFile(`${root}/delivery-${n}.zip`,zip);await fs.writeFile(`${root}/delivery-${n}-files.json`,JSON.stringify(result.report_files));
 const source=Object.fromEntries(Object.entries(fixture.files).map(([p,b])=>[p,{name:p.split('/').at(-1),base64:Buffer.from(b).toString('base64')}]));
 const project=createProject({name:'Selected native reports',operation:'assemble-commission-report',files:{commissionInputs:{files:source,generated_at:fixture.generated_at},commissionDeliverySettings:delivery},constraints:[],result,lastOperation:'assemble-commission-report'});assert.deepEqual(parseProject(JSON.stringify(project)),project);if(n===5)await fs.writeFile(`${root}/browser-project.bisect`,JSON.stringify(project));
 assert.throws(()=>commissionReportZip({...result,report_files:{...result.report_files,'../bad':result.report_json}}));assert.throws(()=>commissionReportZip({...result,report_files:{...result.report_files,[names[0]]:'changed'}}));exact++;
}
let rejects=0;for(const delivery of [{label:'../escape',formats:['json'],audit_with_report:false},{label:'x',formats:[],audit_with_report:false},{label:'x',formats:['pdf'],audit_with_report:false},{label:'x',formats:['json'],audit_with_report:false,extra:1}]){assert.throws(()=>engine.execute({...fixture,delivery}));assert.throws(()=>validateCommissionDeliverySettings(delivery));rejects++;}
const audit=engine.execute({operation:'assemble-commission-audit',manifest_bytes:fixture.files['manifest.json']});await fs.writeFile(`${root}/audit.zip`,commissionReportZip(audit));await fs.writeFile(`${root}/audit-files.json`,JSON.stringify({'audit.json':audit.audit_json}));
await fs.writeFile(`${root}/evidence.json`,JSON.stringify({actual_cli_deliveries:exact,exact_filenames_bytes_hashes:true,format_availability:true,duplicate_formats_native:true,exact_projects:true,zips_pending_independent_check:true,rejects,pdf:false},null,2));
console.log(`Commission delivery: ${exact} actual CLI filename/byte/hash matches, selected HTML/JSON/companion availability, duplicate native formats, exact projects and ${rejects} request rejects passed.`);
