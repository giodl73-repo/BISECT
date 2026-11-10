import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {commissionAuditRequest,readCommissionAuditSource} from '../../web/lab/commission-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/commission-audit-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const cli=path.resolve(process.platform==='win32'?'target/debug/bisect.exe':'target/debug/bisect');
let exact=0;
for(let n=0;n<12;n++){
 const fixture=JSON.parse(await fs.readFile(`target/commission-report-wasm/request-${Math.min(n,8)}.json`,'utf8'));
 let text=Buffer.from(fixture.files['manifest.json']).toString();
 if(n>=9){const m=JSON.parse(text);m.seed=null;m.binary_sha256='a'.repeat(64);m.adjacency_sha256='b'.repeat(64);m.tiger_sha256='c'.repeat(64);if(n===10){m.binary_sha256='';m.adjacency_sha256='';m.tiger_sha256=null;}if(n===11){m.seed=0;m.created_at='recorded-time';}text=JSON.stringify(m);}
 const base=`${root}/case-${n}`,label=`cli-${n}`,plan=`${base}/v1/2020/plans/${label}`,out=`${base}/audit`;
 await fs.mkdir(plan,{recursive:true});await fs.writeFile(`${plan}/manifest.json`,text);
 // Deliberately no referenced context or certificate; malformed analysis must be ignored in audit-only mode.
 await fs.mkdir(`${plan}/analysis`,{recursive:true});await fs.writeFile(`${plan}/analysis/summary.json`,'{malformed');
 const args=['report','--label',label,'--output-base',base,'--out',out,'--audit-only'];if(n===11)args.push('--audit-with-report');
 const native=spawnSync(cli,args,{encoding:'utf8'});assert.equal(native.status,0,native.stderr);
 const expected=await fs.readFile(`${out}/audit.json`,'utf8');const raw=Buffer.from(text);const source=await readCommissionAuditSource({name:'manifest.json',size:raw.length,arrayBuffer:async()=>raw.buffer.slice(raw.byteOffset,raw.byteOffset+raw.length)});
 const request=commissionAuditRequest(source),result=engine.execute(request);assert.equal(result.audit_json,expected);assert.equal(result.manifest_sha256,createHash('sha256').update(raw).digest('hex'));assert.equal(result.schema_version,'bisect-commission-audit-v1');
 assert.deepEqual(await fs.readdir(out),['audit.json']);
 const project=createProject({name:'Audit-only',files:{commissionAuditSource:source},operation:request.operation,constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 if(n<9){assert.match(result.audit_json,/9007199254740993/);assert.equal(countResultTables(result)[0].rows.find(r=>r[0].includes('seed'))[1],'9007199254740993');}
 if(n===0){await fs.writeFile(`${root}/browser-project.bisect`,JSON.stringify(project));await fs.writeFile(`${root}/expected-audit.json`,expected);}
 exact++;
}
const fixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8'));
const base=`${root}/with-report`,label='companion',plan=`${base}/v1/2020/plans/${label}`,out=`${base}/report`;await fs.mkdir(plan,{recursive:true});
for(const [name,raw]of Object.entries(fixture.files)){await fs.mkdir(path.dirname(`${plan}/${name}`),{recursive:true});await fs.writeFile(`${plan}/${name}`,Buffer.from(raw));}
const native=spawnSync(cli,['report','--label',label,'--output-base',base,'--out',out,'--format','html','json','--audit-with-report'],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);
assert.deepEqual((await fs.readdir(out)).sort(),['audit.json',`${label}_report.html`,`${label}_report.json`].sort());assert.equal(engine.execute(fixture).audit_json,await fs.readFile(`${out}/audit.json`,'utf8'));
let rejects=0;for(const request of [{operation:'assemble-commission-audit',manifest_json:'{bad'},{operation:'assemble-commission-audit',manifest_json:'[]'},{operation:'assemble-commission-audit',manifest_json:'{}',extra:true},{operation:'assemble-commission-audit',manifest_json:'x'.repeat(16*1024*1024+1)}]){assert.throws(()=>engine.execute(request));rejects++;}
let read=false;await assert.rejects(()=>readCommissionAuditSource({name:'huge.json',size:16*1024*1024+1,arrayBuffer:async()=>{read=true;}}));assert.equal(read,false);
await assert.rejects(()=>readCommissionAuditSource({name:'changed.json',size:1,arrayBuffer:async()=>Buffer.from('{}').buffer}));
assert.throws(()=>commissionAuditRequest({name:'x',base64:Buffer.from('{"x":1,"x":2}').toString('base64')}));assert.throws(()=>commissionAuditRequest({name:'x',base64:Buffer.from([255]).toString('base64')}));
await fs.writeFile(`${root}/evidence.json`,JSON.stringify({exact_actual_cli_audit_only:exact,exact_actual_cli_companion:true,ignores_analysis_and_missing_references:true,audit_only_precedence:true,exact_project:true,exact_seed:true,original_manifest_hash:true,rejects,pre_read_limit:true},null,2));
console.log(`Commission audit: ${exact} exact actual CLI audit-only documents, actual CLI companion document, manifest-only requests, exact project/seed/hash and ${rejects} WASM rejects passed.`);
