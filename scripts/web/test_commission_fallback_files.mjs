import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {readCommissionFolder,commissionRequest,commissionFallback,readCommissionAuditSource,commissionAuditRequest,validateCommissionInputs,validateCommissionFallbackSettings} from '../../web/lab/commission-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/commission-fallback-wasm';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const fixture=JSON.parse(await fs.readFile('target/commission-report-wasm/request-0.json','utf8'));
const file=(p,b)=>({name:p.split('/').at(-1),webkitRelativePath:'legacy/'+p,size:b.length,arrayBuffer:async()=>Uint8Array.from(b).buffer});
let exact=0;
for(const name of ['missing','malformed','utf8','typed','empty','typed-refs','supplied']){
 const base=`${root}/${name}/v1/2020/plans/fallback-${name}`;const entries=[];
 for(const p of ['manifest.json','analysis/summary.json','analysis/contiguity.json','analysis/compactness.json','obsolete.rctx','obsolete-certificate.json']){try{entries.push(file(p,await fs.readFile(`${base}/${p}`)));}catch(e){if(e.code!=='ENOENT')throw e;}}
 let extraRead=false;entries.push({...file('private.txt',Buffer.from('private')),arrayBuffer:async()=>{extraRead=true;throw new Error('unrelated');}});
 if(['missing','malformed','utf8','empty','typed-refs'].includes(name))await assert.rejects(()=>readCommissionFolder(entries));
 const inputs=await readCommissionFolder(entries,{allowFallback:true});inputs.generated_at=fixture.generated_at;const settings={enabled:true,label:`fallback-${name}`,year:'2020'},fallback=commissionFallback(settings);
 const request=commissionRequest(inputs,fallback),result=engine.execute(request);assert.deepEqual(result,JSON.parse(await fs.readFile(`${root}/${name}-result.json`,'utf8')));assert.equal(extraRead,false);
 const project=createProject({name:`Legacy ${name}`,files:{commissionInputs:inputs,commissionFallbackSettings:settings},operation:'assemble-commission-report',constraints:[],result,lastOperation:'assemble-commission-report'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
 if(name==='utf8')await fs.writeFile(`${root}/browser-legacy-project.bisect`,JSON.stringify(project));
 // Turning fallback off does not modify retained opaque bytes or source tags.
 const off={...settings,enabled:false};assert.equal(commissionFallback(off),undefined);const saved=parseProject(JSON.stringify({...project,files:{...project.files,commissionFallbackSettings:off}}));assert.deepEqual(saved.files.commissionInputs,{...inputs,files:{...inputs.files}});if(name!=='supplied')assert.throws(()=>engine.execute(commissionRequest(inputs,undefined)));
 const raw=entries.find(f=>f.name==='manifest.json');const source=raw?await readCommissionAuditSource(raw,{allowOpaque:true}):undefined;const audit=engine.execute(commissionAuditRequest(source,fallback,{opaque:true}));assert.equal(audit.audit_json,result.audit_json);
 const auditProject=createProject({name:`Legacy audit ${name}`,files:{...(source?{commissionAuditOpaqueSource:source}:{}),commissionFallbackSettings:settings},operation:'assemble-commission-audit',constraints:[],result:audit,lastOperation:'assemble-commission-audit'});assert.deepEqual(parseProject(JSON.stringify(auditProject)),auditProject);exact++;
}
for(const v of [{enabled:true,label:'x',year:'2020',extra:true},{enabled:'true',label:'x',year:'2020'},{enabled:true,label:'é'.repeat(101),year:'2020'}])assert.throws(()=>validateCommissionFallbackSettings(v));
assert.throws(()=>commissionFallback({enabled:true,label:'',year:'2020'}));
const inputs={files:{'analysis/summary.json':{name:'x',base64:Buffer.from('{"__proto__":{}}').toString('base64')}},generated_at:'x',opaque_manifest:true};assert.throws(()=>validateCommissionInputs(inputs));
let read=false;await assert.rejects(()=>readCommissionAuditSource({name:'x',size:16*1024*1024+1,arrayBuffer:async()=>{read=true;}},{allowOpaque:true}));assert.equal(read,false);
await fs.writeFile(`${root}/files-evidence.json`,JSON.stringify({exact_native_source_requests:exact,exact_full_and_audit_projects:true,opaque_empty_and_utf8_retention:true,missing_manifest_folder:true,ignored_unrelated_file:true,disabled_fallback_preserves_sources:true,unsafe_analysis_rejected:true,pre_read_limit:true},null,2));
console.log(`Legacy commission sources: ${exact} native-equivalent folder/opaque-byte requests and full/audit project roundtrips, opt-in/no-source fallback, preserved disabled sources and bounded safe analysis passed.`);
