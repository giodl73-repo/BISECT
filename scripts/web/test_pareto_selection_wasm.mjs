import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
const preparedSite=process.argv[2]??'dist/wasm-merge-validation';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {paretoSelectionRequest} from '../../web/lab/package-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {parseGeometryJson} from '../../web/lab/project.js';
import {planMapModel} from '../../web/lab/plan-map.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/pareto-selection-wasm';await fs.mkdir(root,{recursive:true});
const context_json=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const settings={k:3,population:12,generations:5,tolerance:.2,base_seed:'18446744073709551615',democratic_shares:[.1,.2,.3,.4,.5,.6,.7,.8,.9],minority_shares:[.9,.8,.7,.6,.5,.4,.3,.2,.1]};
const sampled=engine.execute({operation:'sample-pareto-context',context_json,settings});
const index=sampled.diagnostics.find(d=>d.passes_final_population_tolerance&&d.all_nonempty_and_connected).frontier_index;
const selection={frontier_index:index,label:'Native selected Pareto fixture',generated_at:'2026-10-06T00:00:00Z'};
const base={operation:'select-pareto-plan',context_json,settings,selection,frontier_ndjson:sampled.ndjson};let matched=0,rejected=0;
async function native(req){await fs.writeFile(root+'/request.json',JSON.stringify(req));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe'],maxBuffer:32*1024*1024}));}
async function exact(req){const out=engine.execute(req);assert.deepEqual(out,await native(req));matched++;return out;}
async function reject(req){assert.throws(()=>engine.execute(req));await assert.rejects(()=>native(req));rejected++;}
const output=await exact(base);assert.equal(output.replay_verified,true);assert.equal(output.base_seed,settings.base_seed);
assert.deepEqual(Object.keys(output.files).sort(),['audit-certificate.json','manifest.json','selected-frontier.rctx','selected-frontier.rplan']);
for(const [name,text] of Object.entries(output.files)){assert.equal(createHash('sha256').update(text).digest('hex'),output.file_hashes[name]);await fs.writeFile(root+'/'+name,text);}
const document=JSON.parse(output.document_json),labels=JSON.parse(sampled.ndjson.split('\n')[index]).plan;assert.deepEqual(document.plan.assignment,labels.map(d=>d-1));assert.deepEqual(document.plan.units.unit_ids,JSON.parse(context_json).units.unit_ids);
const manifest=JSON.parse(output.files['manifest.json']);assert.equal(manifest.base_seed,settings.base_seed);assert.equal(manifest.audit_result,'pass');assert.equal(manifest.audit_certificate_sha256,output.file_hashes['audit-certificate.json']);assert.ok(output.algorithm_lineage_json.includes('18446744073709551615'));
await exact({operation:'validate-rplan-text',document_json:output.document_json});
assert.equal((await exact({operation:'verify-audit-text',document_json:output.document_json,context_json:output.context_json,certificate_json:output.files['audit-certificate.json']})).verified,true);
const profile=JSON.parse(output.files['audit-certificate.json']).legal_profile;
// Certificates carry a summary; replay auditing requires the complete native profile.
delete profile.legal_disclaimer;profile.contiguity_required=true;profile.nesting_rule={type:'not-evaluated'};
await exact({operation:'audit-rplan-text',document_json:output.document_json,context_json:output.context_json,profile_json:JSON.stringify(profile),constraints:['population','contiguity'],generated_at_utc:selection.generated_at,lineage_json:output.algorithm_lineage_json});
const failing=sampled.diagnostics.find(d=>!d.passes_final_population_tolerance);await reject({...base,selection:{...selection,frontier_index:failing.frontier_index}});
for(const req of [{...base,selection:{...selection,frontier_index:1000}},{...base,selection:{...selection,label:''}},{...base,frontier_ndjson:base.frontier_ndjson+'\n'},{...base,settings:{...settings,base_seed:'42'}},{...base,frontier_ndjson:'x'.repeat(8*1024*1024+1)}])await reject(req);
const lines=base.frontier_ndjson.trimEnd().split('\n'),record=JSON.parse(lines[0]);record.ec++;lines[0]=JSON.stringify(record);const meta=JSON.parse(lines.pop());meta.file_sha256=createHash('sha256').update(lines.join('\n')+'\n').digest('hex');await reject({...base,frontier_ndjson:lines.join('\n')+'\n'+JSON.stringify(meta)+'\n'});
const files={auditContextText:{name:'original.rctx',base64:Buffer.from(context_json).toString('base64')},paretoFrontier:{name:'frontier.ndjson',base64:Buffer.from(sampled.ndjson).toString('base64')},paretoSettings:settings,paretoSelection:selection};assert.deepEqual(paretoSelectionRequest(files,settings,selection),base);
const project=createProject({name:selection.label,files,operation:base.operation,constraints:[],result:output,lastOperation:base.operation});assert.equal(parseProject(JSON.stringify(project)).files.paretoFrontier.base64,files.paretoFrontier.base64);await fs.writeFile(root+'/browser.bisect',JSON.stringify(project));await fs.writeFile(root+'/fixture.json',JSON.stringify({base,output}));
console.log(`Pareto selection: ${matched} native/WASM matches, ${rejected} rejected failed/tampered/replay inputs; four native files, exact seeds/hashes, plan conversion, lineage audit and certificate verified.`);

// Prepared Census inputs exercise package replay and exact geographic identity at state scale.
try {
 const context_json=await fs.readFile('target/smc-real-state-wasm/RI.rctx','utf8');
 const settings={k:2,population:12,generations:5,tolerance:.02,base_seed:'42',democratic_shares:null,minority_shares:null};
 const frontier=engine.execute({operation:'sample-pareto-context',context_json,settings});
 const candidate=frontier.diagnostics.find(d=>d.passes_final_population_tolerance&&d.all_nonempty_and_connected);
 assert.ok(candidate,'Prepared RI frontier must have a packageable candidate');
 const selection={frontier_index:candidate.frontier_index,label:'Rhode Island selected Pareto plan',generated_at:'2026-10-06T00:00:00Z'};
 const output=await exact({operation:'select-pareto-plan',context_json,settings,selection,frontier_ndjson:frontier.ndjson});
 assert.equal((await exact({operation:'verify-audit-text',document_json:output.document_json,context_json:output.context_json,certificate_json:output.files['audit-certificate.json']})).verified,true);
 const catalog=JSON.parse(await fs.readFile(preparedSite+'/catalog.json','utf8'));
 const ref=catalog.geometries['RI:2020'],path=typeof ref==='string'?ref:ref.path;
 const geometryText=await fs.readFile(preparedSite+'/'+path,'utf8');
 const map=planMapModel(JSON.parse(output.document_json),parseGeometryJson(geometryText));assert.equal(map.units,250);
 const files={auditContextText:{name:'RI.rctx',base64:Buffer.from(context_json).toString('base64')},paretoFrontier:{name:'RI-frontier.ndjson',base64:Buffer.from(frontier.ndjson).toString('base64')},paretoSettings:settings,paretoSelection:selection,planGeometry:{name:'RI.geojson',base64:Buffer.from(geometryText).toString('base64')}};
 await fs.writeFile(root+'/RI-browser.bisect',JSON.stringify(createProject({name:selection.label,files,operation:'select-pareto-plan',constraints:[],result:output,lastOperation:'select-pareto-plan'})));
 console.log('Prepared RI selection: two additional native/WASM matches, certificate verified and 250 exact unit-ID map joins.');
} catch(error) {if(process.argv[2]||error.code!=='ENOENT')throw error;console.log('Prepared RI inputs absent: state-scale selection not exercised.');}
