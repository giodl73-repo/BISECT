import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {smcSelectionRequest} from '../../web/lab/package-files.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/smc-selection-wasm';await fs.mkdir(root,{recursive:true});
const context_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const ensemble=JSON.parse(await fs.readFile('target/smc-context-wasm/browser-result.json','utf8'));
const settings={particle_index:0,label:'Selected ensemble particle',chamber:'congressional',created_at:'2026-10-04T00:00:00Z'};
const base={operation:'select-smc-plan',context_json,ensemble_ndjson:ensemble.ndjson,settings};
let parity=0,rejections=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));parity++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejections++;}
let selected;
for(const particle_index of [0,15,31]){selected=await exact({...base,settings:{...settings,particle_index}});const document=JSON.parse(selected.document_json),original=JSON.parse(ensemble.ndjson.split('\n')[particle_index]);assert.deepEqual(document.plan.assignment,original.plan.map(d=>d-1));assert.equal(selected.replay_verified,true);assert.equal(selected.base_seed,'18446744073709551615');assert.deepEqual(document.provenance.source_hashes,JSON.parse(context_json).source_hashes);}
for(const change of [{particle_index:32},{particle_index:-1},{label:''},{chamber:''},{created_at:''},{label:'x'.repeat(201)},{unknown:1}])await reject({...base,settings:{...settings,...change}});
const lines=ensemble.ndjson.trimEnd().split('\n');const first=JSON.parse(lines[0]);first.plan.reverse();lines[0]=JSON.stringify(first);const meta=JSON.parse(lines.at(-1));meta.file_sha256=createHash('sha256').update(lines.slice(0,-1).join('\n')+'\n').digest('hex');lines[lines.length-1]=JSON.stringify(meta);await reject({...base,ensemble_ndjson:lines.join('\n')+'\n'});
await reject({...base,ensemble_ndjson:ensemble.ndjson.replace('2.0-chacha12-u64-v1','unknown')});await reject({...base,ensemble_ndjson:ensemble.ndjson+'\n'});await reject({...base,ensemble_ndjson:'x'.repeat(8*1024*1024+1)});
const profile=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid-certificate.json','utf8')).legal_profile;delete profile.legal_disclaimer;profile.contiguity_required=true;profile.nesting_rule={type:'not-evaluated'};
const audit=await exact({operation:'audit-rplan-text',document_json:selected.document_json,context_json,profile_json:JSON.stringify(profile),constraints:['population','contiguity'],generated_at_utc:settings.created_at,lineage_json:selected.algorithm_lineage_json});assert.equal(audit.result,'pass');
await exact({operation:'verify-audit-text',certificate_json:audit.certificate_json,document_json:selected.document_json,context_json});
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});const files={auditContextText:source('grid.rctx',context_json),smcEnsemble:source('ensemble.ndjson',ensemble.ndjson),smcSelection:{...settings,particle_index:31}};assert.deepEqual(smcSelectionRequest(files,files.smcSelection),{...base,settings:files.smcSelection});
const project=createProject({name:'Synthetic replayed SMC particle',files,operation:'select-smc-plan',constraints:[],result:selected,lastOperation:'select-smc-plan'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/browser-result.json',JSON.stringify(selected));await fs.writeFile(root+'/profile.json',JSON.stringify(profile));
console.log(`SMC selection: ${parity} native/WASM matches, ${rejections} matching rejections; assignment conversion, forged rehashed-payload rejection, lineage audit and exact project bytes passed.`);
