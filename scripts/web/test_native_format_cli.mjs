import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {exactNativeJsonText} from '../../web/lab/package-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3),hash=b=>createHash('sha256').update(b).digest('hex');
await fs.mkdir('target/native-format-cli',{recursive:true});const root=await fs.mkdtemp(path.resolve('target/native-format-cli/run-'));
// Pin the owning runner's existing binaries in place; avoid duplicate large files.
const native=path.resolve(option('native')||'target/release/bisect.exe'),wasm=path.resolve(option('wasm')||process.env.BISECT_TEST_WASM||'target/cffi-legacy-gates/default.wasm');
const pin=async file=>({path:path.relative(process.cwd(),file).replaceAll('\\','/'),sha256:hash(await fs.readFile(file))}),nativePin=await pin(native),wasmPin=await pin(wasm),engine=await instantiateEngine(await fs.readFile(wasm));
const legacy={rplan_version:'0.1',metadata:{label:'synthetic',state_fips:'44',state_code:'RI',year:'2020',chamber:'congressional',num_districts:2,population_source:'total',balance_tolerance_pct:0.5,created_at:'recorded',created_by:'test'},assignments:{'44001000100':1,'44001000200':2},geometry:null};
const changed=fn=>{const value=structuredClone(legacy);fn(value);return JSON.stringify(value);},text=JSON.stringify(legacy),modern=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'),records=[];
for(const [label,document_json]of [['legacy',text],['warning',changed(p=>p.assignments={'44001000100':1})],['empty-assignments',changed(p=>p.assignments={})],['bad-geoid',changed(p=>p.assignments={'bad':1})],['modern',modern],['invalid-json','{invalid'],['bom','\ufeff'+text],['crlf',JSON.stringify(legacy,null,2).replaceAll('\n','\r\n')],['hostile-label',changed(p=>p.metadata.label='<img src=x onerror=alert(1)>')]])for(const strict of [false,true]){
 const name=label+(strict?'-strict':''),dir=path.join(root,name);await fs.mkdir(dir);const file=path.join(dir,'plan.rplan');await fs.writeFile(file,document_json);
 const args=['validate','--file',file,...(strict?['--strict']:[])],run=spawnSync(native,args,{encoding:'utf8',windowsHide:true});if(run.error)throw run.error;
 const request={operation:'validate-native-rplan',source_name:'plan.rplan',document_json,strict},result=engine.execute(request);
 assert.equal(result.native_exit_code,run.status,name);if(result.error)assert(run.stderr.includes(result.error),name+' error');if(result.native_summary)assert.equal(run.stdout.trimEnd(),result.native_summary,name+' summary');for(const warning of result.warnings)assert(run.stderr.includes('WARNING: '+warning),name+' warning');
 assert.equal(result.source_sha256,'sha256:'+hash(Buffer.from(document_json)),name+' original byte identity');
 const project=createProject({name,operation:request.operation,files:{auditPlanText:{name:'plan.rplan',base64:Buffer.from(document_json).toString('base64')},nativeValidationSettings:{strict}},constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(path.join(dir,'project.bisect'),JSON.stringify(project));
 records.push({name,native:{arguments:args,exit_code:run.status,stdout:run.stdout,stderr:run.stderr},request,result});
}
const source=bytes=>({name:'boundary.rplan',base64:Buffer.from(bytes).toString('base64')});
assert.equal(exactNativeJsonText(source(Buffer.from('\ufeff'+text))).charCodeAt(0),0xfeff);
assert.throws(()=>exactNativeJsonText(source(Buffer.from([255]))));
assert.throws(()=>exactNativeJsonText(source(Buffer.alloc(3*1024*1024+1,32))));
assert.deepEqual(await pin(native),nativePin);assert.deepEqual(await pin(wasm),wasmPin);
const sources=[];for(const file of ['crates/bisect-cli/src/main.rs','crates/bisect-cli/src/validate.rs','crates/bisect-report/src/format_validation.rs','crates/bisect-wasm/src/format_validation.rs','crates/rplan-audit/fixtures/grid3x3-valid.rplan','scripts/web/test_native_format_cli.mjs','web/lab/package-files.js','web/lab/workbench.js'])sources.push(await pin(file));
const evidence={status:'passed-actual-CLI-native-format',sources,native:nativePin,wasm:wasmPin,records,scope:'Actual native validate CLI, preserved native summary/warning/failure and raw input SHA. Legacy/modern, strict/non-strict, malformed/BOM and CRLF. Rust parser contract only; browser decoding needs its separate actual UI check. No population/contiguity/certification/full parent acceptance.'};await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');console.log(JSON.stringify({status:evidence.status,root}));
