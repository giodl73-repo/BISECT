// Full 50-state national rendering from genuine native-verified tract plans.
// Explicit algorithm choices are recorded; failed default runs are never
// silently retried here and no synthetic districts replace missing evidence.
import fs from 'node:fs/promises';
import {appendFileSync} from 'node:fs';
import assert from 'node:assert/strict';
import {gunzipSync} from 'node:zlib';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {executeNationalMapBatch,readNationalMapBatchSource} from '../../web/lab/national-map-batch.js';
import {createStoredWorkbenchArchive,parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';
const root='target/original-national-map-stream';await fs.mkdir(root,{recursive:true});
const standard=new Set('AL AZ CA CO FL GA IN MA NC NJ NY OH OK PA TN TX UT VA'.split(' '));
const corrected=new Set(['AK','LA','MS']);
const manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
const states=Object.keys(manifest.states).filter(s=>s!=='DC').sort();assert.equal(states.length,50);
const source=(name,b)=>({name,base64:Buffer.from(b).toString('base64')}),sha=b=>createHash('sha256').update(b).digest('hex');
// Check every generation prerequisite before exporting or rendering any state.
const receipts={},fixtures={},proofs={};
for(const state of states){
 const receipt=JSON.parse(await fs.readFile('target/original-all-state-wasm/'+state+'/receipt.json','utf8'));assert(receipt.native_source_graph_parity);receipts[state]=receipt;
 if(standard.has(state)||corrected.has(state)){
  const prefix=standard.has(state)?'target/original-state-algorithm/standard-bisect/'+state:state==='AK'?'target/original-alaska-fixed':'target/original-state-geometry-fixed/'+state;
  const proof=JSON.parse(await fs.readFile(prefix+'/evidence.json','utf8'));assert(proof.full_native_wasm_workflow_match&&proof.native_yaml_assignment_parity,state);assert.equal(proof.source_graph_sha256,receipt.graph_sha256);assert.equal(proof.geometry_sha256,receipt.geometry_sha256);
  if(standard.has(state))assert.equal(proof.requested_structure,'standard-bisect');
  proofs[state]=proof;fixtures[state]=prefix;
 }else assert(receipt.status==='verified'&&receipt.native_full_run_workflow_parity&&receipt.native_assignment_parity,state);
}
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const template=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,seed:'42',workers:1,years:['2020']}));
const batch={dpi:72,map_type:'districts',states:{}},provenance=[];
let units=0,population=0n,districts=0;
for(const state of states){
 const prefix='target/original-all-state-wasm/'+state,receipt=receipts[state],compressed=await fs.readFile(prefix+'/display-geometry.geojson.gz');assert.equal(sha(gunzipSync(compressed)),receipt.geometry_sha256);
 let plan;
 if(fixtures[state]){
  const project=JSON.parse(await fs.readFile(fixtures[state]+'/workflow.bisect','utf8'));plan=project.result.practitioner_export.document_json;assert.deepEqual(project.result,JSON.parse(await fs.readFile(fixtures[state]+'/result.json','utf8')));assert.equal(Buffer.from(project.result.workflow.files_base64['plan.rplan'],'base64').toString('utf8'),plan);assert.equal(sha(Buffer.from(plan)),proofs[state].files['plan.rplan']);
 }else{
  const graphBytes=gunzipSync(await fs.readFile(prefix+'/engine-graph.json.gz'));assert.equal(sha(graphBytes),receipt.graph_sha256);
  const request={operation:'export-configured-plan',input:{graph:JSON.parse(graphBytes),yaml_text:template.yaml_text,source_name:template.generated_filename,profile:'open',chamber:'congressional',districts:receipt.districts},assignments:JSON.parse(await fs.readFile(prefix+'/assignments.json','utf8')),label:'original_'+state.toLowerCase(),created_at:'2026-10-12T00:00:00Z',attempt:receipt.accepted_attempt};
  const exported=engine.execute(request),scratch=root+'/export-request.json';await fs.writeFile(scratch,JSON.stringify(request));const native=JSON.parse(execFileSync(process.env.BISECT_NATIVE_REFERENCE||'target/release/examples/execute_tool_request.exe',[scratch],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024}));assert.deepEqual(exported,native);plan=exported.document_json;
 }
 const document=JSON.parse(plan);assert.equal(document.plan.units.state,state);assert.equal(document.plan.k,receipt.districts);assert.equal(document.plan.units.unit_ids.length,receipt.units);assert.equal(document.plan.assignment.length,receipt.units);
 batch.states[state]={plan:source(state+'.rplan',plan),geometry:source(state+'.geojson',compressed),geometry_format:'gzip',analysis:null,dissolve_method:'batch'};
 provenance.push({state,structure:standard.has(state)?'standard-bisect':'apportion-regions',generation_wasm_sha256:proofs[state]?.wasm_sha256||receipt.wasm_sha256,graph_sha256:receipt.graph_sha256,geometry_sha256:receipt.geometry_sha256,units:receipt.units,districts:receipt.districts,accepted_attempt:proofs[state]?.accepted_attempt??receipt.accepted_attempt,recorded_seed:document.provenance.producer.engine_options.seed});
 units+=receipt.units;population+=BigInt(receipt.population);districts+=receipt.districts;
}
assert.equal(units,84208);assert.equal(population,330759736n);assert.equal(districts,435);
const packageBytes=Buffer.from(JSON.stringify(batch));await fs.writeFile(root+'/package.json',packageBytes);await fs.writeFile(root+'/plan-provenance.json',JSON.stringify(provenance,null,2));
const selected=await readNationalMapBatchSource({name:'original-50-state-plans.json',size:packageBytes.length,arrayBuffer:async()=>packageBytes.buffer.slice(packageBytes.byteOffset,packageBytes.byteOffset+packageBytes.byteLength)});
const requestFile=root+'/requests.jsonl';await fs.writeFile(requestFile,'');const outcomes=[];
const recorded={execute(request){ // Synchronous append avoids retaining all raw geometries in JS.
 appendFileSync(requestFile,JSON.stringify(request)+'\n');
 try{const result=engine.execute(request);outcomes.push({ok:true,result});return result;}catch(error){outcomes.push({ok:false,error:error.message});throw error;}
}};
try{
 const result=await executeNationalMapBatch(recorded,batch,{onProgress:p=>console.log(JSON.stringify({state:p.state,prepared:p.states,units:p.units,districts:p.districts,dissolved_positions:p.dissolved_positions}))});
 assert.equal(result.units,units);assert.equal(result.districts,districts);assert.equal(result.provided_states.length,50);
 const native=execFileSync('target/debug/examples/execute_tool_sequence.exe',[requestFile],{encoding:'utf8',windowsHide:true,maxBuffer:128*1024*1024}).trim().split('\n').map(s=>JSON.parse(s));assert.deepEqual(native,outcomes);
 const archive=await createStoredWorkbenchArchive({name:'Original 50-state national map',files:{nationalMapBatchInputs:{source:selected}},operation:'render-native-national-map-batch',constraints:[],result,lastOperation:'render-native-national-map-batch'}),bytes=typeof archive==='string'?Buffer.from(archive):archive;
 const restored=await parseWorkbenchArchiveBytes(new Uint8Array(bytes));assert.deepEqual(restored.result,result);assert.deepEqual(restored.files.nationalMapBatchInputs.source,selected);
 await fs.writeFile(root+'/national.bisect-workbench',bytes);await fs.writeFile(root+'/result.json',JSON.stringify(result));await fs.writeFile(root+'/national.png',Buffer.from(result.png_base64,'base64'));
 await fs.writeFile(root+'/evidence.json',JSON.stringify({states:50,units,districts,population:String(population),package_bytes:packageBytes.length,native_wasm_matches:native.length,wasm_sha256:engine.artifactIdentity.sha256,wasm_memory_bytes:engine.memoryBytes(),source_positions:result.source_positions,dissolved_positions:result.dissolved_positions,exact_backup_reopen:true,scope:'Full original prepared 2020 tract geography and native-verified plans, with explicit per-state algorithm selection and retained generation identities. This proves national rendering/backup, not all algorithms, resolutions, legal certification or full browser capacity.'},null,2));
 console.log('Original national map: all 50 states, 84208 tracts and 435 districts; exact native/WASM rendering and source-preserving archive reopening passed.');
}catch(error){await fs.writeFile(root+'/failure.json',JSON.stringify({error:error.message,wasm_sha256:engine.artifactIdentity.sha256,completed_responses:outcomes.length,package_bytes:packageBytes.length,scope:'Full requested 50-state original source rendering failed; no replacement, truncation or relaxed geometry checks.'},null,2));throw error;}
