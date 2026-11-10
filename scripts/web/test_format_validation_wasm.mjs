import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/format-validation-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let parity=0,cli=0,rejections=0;
const legacy={rplan_version:'0.1',metadata:{label:'synthetic',state_fips:'44',state_code:'RI',year:'2020',chamber:'congressional',num_districts:2,population_source:'total',balance_tolerance_pct:0.5,created_at:'recorded',created_by:'test'},assignments:{'44001000100':1,'44001000200':2},geometry:null};
const source=JSON.stringify(legacy),request=(text,strict=false)=>({operation:'validate-native-rplan',source_name:'plan.rplan',document_json:text,strict});
async function native(r){await fs.writeFile(root+'/request.json',JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:24*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r,checkCli=true){const out=engine.execute(r);assert.deepEqual(out,await native(r));parity++;if(checkCli){await fs.writeFile(root+'/plan.rplan',r.document_json);const ref=spawnSync('target/debug/examples/format_validation_reference.exe',[root+'/plan.rplan',String(r.strict)],{encoding:'utf8'});assert.equal(ref.status,out.native_exit_code,ref.stderr);if(out.error)assert.ok(ref.stderr.includes(out.error));if(out.native_summary)assert.ok(ref.stdout.includes(out.native_summary));for(const w of out.warnings)assert.ok(ref.stderr.includes('WARNING: '+w));cli++;}return out;}
const changed=(fn)=>{const p=structuredClone(legacy);fn(p);return JSON.stringify(p);};
const modern=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8');
const cases=[source,changed(p=>p.assignments={'44001000100':1}),changed(p=>p.assignments={}),changed(p=>p.rplan_version='0.3'),changed(p=>p.rplan_version='1.0'),changed(p=>p.rplan_version='unknown'),changed(p=>p.metadata.label=''),changed(p=>p.metadata.num_districts=0),changed(p=>p.assignments={'bad':0,'also-bad':3}),changed(p=>p.geometry={type:'Polygon'}),changed(p=>delete p.metadata.year),changed(p=>p.metadata.num_districts='2'),changed(p=>p.metadata.notes='<script>alert(1)</script>'),'{','null','[]',modern];
for(const text of cases)for(const strict of [false,true])await exact(request(text,strict));
const warning=cases[1];assert.equal((await exact(request(warning))).status,'pass-with-warnings');const strict=await exact(request(warning,true));assert.equal(strict.status,'strict-warning-failure');assert.equal(strict.valid,true);assert.equal(strict.native_exit_code,1);assert.equal((await exact(request(cases[2],true))).status,'pass');
assert.equal((await exact(request(modern,true))).source_version,'0.2');
const padded=source+' '.repeat(3*1024*1024-Buffer.byteLength(source));await exact(request(padded));
for(const r of [{...request(source),extra:true},{...request(source),strict:'true'},{...request(source),source_name:''},request(''),request(padded+' '),request(changed(p=>p.metadata.num_districts=100001))]){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
const settings={strict:true},file={name:'plan.rplan',base64:Buffer.from(warning).toString('base64')};const project=createProject({name:'Native strict validation',files:{auditPlanText:file,nativeValidationSettings:settings},operation:'validate-native-rplan',constraints:[],result:strict,lastOperation:'validate-native-rplan'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(countResultTables(strict)[0].rows[0][1],'strict-warning-failure');assert.throws(()=>parseProject(JSON.stringify({...project,files:{nativeValidationSettings:{strict:'true'}}})));
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,cli,rejections,scope:strict.scope},null,2));console.log(`Native format validation: ${parity} native/WASM matches, ${cli} actual CLI checks and ${rejections} matching boundary rejections passed.`);
