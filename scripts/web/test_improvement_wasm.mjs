import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createProject,parseProject} from '../../web/lab/project.js';
import {improvementRequest} from '../../web/lab/package-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/improvement-wasm';await fs.mkdir(root,{recursive:true});
const canonical=v=>Array.isArray(v)?'['+v.map(canonical).join(',')+']':v&&typeof v==='object'?'{'+Object.keys(v).sort().map(k=>JSON.stringify(k)+':'+canonical(v[k])).join(',')+'}':JSON.stringify(v);
const sha=s=>createHash('sha256').update(s).digest('hex');
const plan=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'));
const ctx=JSON.parse(await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8'));
const settings={tolerance_percent:34,label:'Improvement fixture',generated_at_utc:'2026-10-04T00:00:00Z'};
let matches=0,rejections=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matches++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejections++;}
const request=()=>({operation:'improve-rplan-text',document_json:JSON.stringify(plan),context_json:JSON.stringify(ctx),settings});
for(const tolerance_percent of [0,34,50,100]){const out=await exact({...request(),settings:{...settings,tolerance_percent}});assert.ok(out.summary.final_edge_cut<=out.summary.initial_edge_cut);for(const [name,text] of Object.entries(out.files))assert.equal(out.file_sha256[name],sha(text));await exact({operation:'verify-audit-text',certificate_json:out.files['audit-certificate.json'],document_json:out.files['improved.rplan'],context_json:out.files['improved.rctx']});}
// Known native improvement, 3 crossing edges to 1.
const units={...plan.plan.units,unit_kind:'imported',unit_ids:Array.from({length:6},(_,i)=>'u'+i)};delete units.unit_universe_hash;units.unit_universe_hash='sha256:'+sha(canonical(units));
plan.plan.units=units;plan.plan.k=2;plan.plan.assignment=[0,0,0,1,1,1];plan.plan.display_labels=['1','2'];
ctx.units=units;ctx.populations=Array(6).fill(100);ctx.graph.adjacency=[[1,2],[0],[0,3,4,5],[2,4],[2,3,5],[2,4]].map(ns=>ns.map(to=>({to,kind:'boundary'})));ctx.subdivisions=null;ctx.demographics=null;ctx.geometry=null;
ctx.context_hash='sha256:'+sha(canonical({units:ctx.units,graph:ctx.graph,populations:ctx.populations,source_hashes:ctx.source_hashes}));
const improved=await exact(request());assert.equal(improved.summary.initial_edge_cut,3);assert.equal(improved.summary.final_edge_cut,1);assert.deepEqual(JSON.parse(improved.files['improved.rplan']).plan.assignment,[0,0,1,1,1,1]);
for(const s of [{...settings,tolerance_percent:-1},{...settings,generated_at_utc:''},{...settings,label:''},{...settings,extra:1}])await reject({...request(),settings:s});
for(const req of [{...request(),document_json:'{'},{...request(),context_json:'{'},{...request(),document_json:'x'.repeat(3*1024*1024+1)}])await reject(req);
await fs.writeFile(root+'/improved-result.json',JSON.stringify(improved));await fs.writeFile(root+'/fixture-request.json',JSON.stringify(request()));
console.log(`Improvement WASM: ${matches} native/WASM matches, ${rejections} matching rejections; exact package hashes and certificate bindings verified.`);

const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const files={auditPlanText:source('fixture.rplan',request().document_json),auditContextText:source('fixture.rctx',request().context_json),improvementSettings:settings};
assert.deepEqual(improvementRequest(files,settings),request());
const project=createProject({name:'Synthetic native improvement',files,operation:'improve-rplan-text',constraints:[],result:improved,lastOperation:'improve-rplan-text'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
assert.ok(countResultTables(improved)[0].rows.some(row=>row[0]==='Final boundary edges'&&row[1]==='1'));
await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));
