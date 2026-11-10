import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/interchange-wasm';await fs.mkdir(root,{recursive:true});let matched=0,rejected=0;
const text=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8'),document=JSON.parse(text);
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(document_json){const req={operation:'export-rplan-interchange',document_json};const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;for(const [name,text] of Object.entries(out.files))assert.equal(out.file_sha256[name],createHash('sha256').update(text).digest('hex'));return out;}
async function reject(document_json){const req={operation:'export-rplan-interchange',document_json};assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const out=await exact(text),gc=JSON.parse(out.files['gerrychain.json']);assert.equal(gc.gerrychain_version_target,'2.3');assert.deepEqual(gc.assignment,Object.fromEntries(document.plan.units.unit_ids.map((id,i)=>[id,document.plan.assignment[i]+1])));
const expected='geoid,district\n'+Object.entries(gc.assignment).sort(([a],[b])=>a.localeCompare(b)).map(([id,d])=>`${id},${d}\n`).join('');assert.equal(out.files['assignments.csv'],expected);
assert.ok(JSON.parse(out.files['plan.geojson']).features.every(f=>f.geometry===null));
const legacy={rplan_version:'0.1',metadata:{label:'Legacy fixture',state_fips:'53',state_code:'WA',year:'2020',chamber:'congressional',num_districts:3,population_source:'total',balance_tolerance_pct:1,created_at:'2026-10-06T00:00:00Z',created_by:'fixture'},assignments:gc.assignment,geometry:null};
assert.equal((await exact(JSON.stringify(legacy))).files['assignments.csv'],out.files['assignments.csv']);
const geometric=structuredClone(document);geometric.geometry={type:'FeatureCollection',features:[{type:'Feature',properties:{district_id:1},geometry:{type:'Polygon',coordinates:[[[-71,41],[-70,41],[-70,42],[-71,41]]]}}]};const geom=await exact(JSON.stringify(geometric));assert.deepEqual(JSON.parse(geom.files['plan.geojson']),geometric.geometry);
for(const mutate of [d=>d.plan.k=501,d=>d.plan.assignment[0]=999,d=>d.plan.units.unit_ids[1]=d.plan.units.unit_ids[0],d=>d.plan.units.unit_universe_hash='sha256:wrong']){const bad=structuredClone(document);mutate(bad);await reject(JSON.stringify(bad));}
await reject('x'.repeat(3*1024*1024+1));
const files={auditPlanText:{name:'grid.rplan',base64:Buffer.from(text).toString('base64')}};const project=createProject({name:'Interchange fixture',files,operation:'export-rplan-interchange',constraints:[],result:out,lastOperation:'export-rplan-interchange'});assert.equal(parseProject(JSON.stringify(project)).files.auditPlanText.base64,files.auditPlanText.base64);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));for(const [name,text] of Object.entries(out.files))await fs.writeFile(root+'/'+name,text);
let real=0;for(const state of ['RI','IA','NC']){let text;try{text=await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8');}catch(e){if(e.code==='ENOENT')continue;throw e;}const out=await exact(text),plan=JSON.parse(out.document_json).plan,gc=JSON.parse(out.files['gerrychain.json']);assert.equal(Object.keys(gc.assignment).length,plan.units.unit_ids.length);for(let i=0;i<plan.assignment.length;i++)assert.equal(gc.assignment[plan.units.unit_ids[i]],plan.assignment[i]+1);real++;}
console.log(`Interchange: ${matched} native/WASM matches (${real} prepared states), ${rejected} rejected inputs; exact hashes, one-based IDs, CSV order, legacy conversion, native geometry retention and inert project bytes.`);
