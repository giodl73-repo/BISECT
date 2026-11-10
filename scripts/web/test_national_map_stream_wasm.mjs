import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gzipSync} from 'node:zlib';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {executeNationalMapBatch,validateNationalMapBatch,nationalMapBatchInput,addNativeMapToNationalBatch} from '../../web/lab/national-map-batch.js';
import {nativeNationalMapRequest,readNativeNationalMapSource} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/national-map-stream';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const requests=[],outcomes=[],source=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')}),sha=b=>createHash('sha256').update(b).digest('hex');
const recorded={execute(request){requests.push(request);try{const result=engine.execute(request);outcomes.push({ok:true,result});return result;}catch(error){outcomes.push({ok:false,error:error.message});throw error;}}};
function refused(request,pattern){assert.throws(()=>recorded.execute(request),pattern);}
function imported(state,year=2020){const id={AK:'02105000400',WA:'53001000000',HI:'15001000000'}[state];return engine.execute({operation:'import-plan-interchange',source_text:'geoid,district\n'+id+',1\n',centroids_json:null,context_json:null,settings:{format:'dra-csv',state,year,label:'stream-'+state,created_at:'2026-10-12T00:00:00Z',source_name:'assignments.csv',as_civic_counter_proposal:false,submitted_by:null,submitted_at:null}}).document_json;}
const ring=(x,y)=>[[x,y],[x+1,y],[x+1,y+1],[x,y+1],[x,y]];
const job=(state,x,y)=>({plan:source(state+'.rplan',imported(state)),geometry:source(state+'.geojson',JSON.stringify({type:'FeatureCollection',features:[{type:'Feature',properties:{GEOID:{AK:'02105000400',WA:'53001000000',HI:'15001000000'}[state]},geometry:{type:'Polygon',coordinates:[ring(x,y)]}}]})),geometry_format:'json',analysis:null,dissolve_method:'batch'});
const jobs={WA:job('WA',-122,47),AK:job('AK',174,52),HI:job('HI',-158,21)},batch={dpi:72,map_type:'districts',states:jobs};
const input=(j,options=batch)=>({document_json:Buffer.from(j.plan.base64,'base64').toString(),unit_geometry_json:Buffer.from(j.geometry.base64,'base64').toString(),dpi:options.dpi,map_type:options.map_type,dissolve_method:j.dissolve_method,...(j.analysis?{analysis_json:Buffer.from(j.analysis.base64,'base64').toString()}:{} )});
refused({operation:'begin-native-national-map',input:{dpi:73,map_type:'districts'}},/DPI/);
refused({operation:'begin-native-national-map',input:{dpi:72,map_type:'rounds'}},/DPI/);
refused({operation:'finish-native-national-map',session:0},/matching/);
const started=recorded.execute({operation:'begin-native-national-map',input:{dpi:72,map_type:'districts'}});
refused({operation:'begin-native-national-map',input:{dpi:72,map_type:'districts'}},/already active/);
refused({operation:'finish-native-national-map',session:started.session},/at least one/);
refused({operation:'append-native-national-map',input:{session:started.session+1,state:'AK',input:input(jobs.AK)}},/matching/);
refused({operation:'append-native-national-map',input:{session:started.session,state:'DC',input:input(jobs.AK)}},/state code/);
refused({operation:'append-native-national-map',input:{session:started.session,state:'AK',input:{...input(jobs.AK),dpi:150}}},/DPI\/type/);
recorded.execute({operation:'append-native-national-map',input:{session:started.session,state:'AK',input:input(jobs.AK)}});
refused({operation:'append-native-national-map',input:{session:started.session,state:'AK',input:input(jobs.AK)}},/already appended/);
refused({operation:'append-native-national-map',input:{session:started.session,state:'HI',input:input(jobs.WA)}},/matching state/);
refused({operation:'append-native-national-map',input:{session:started.session,state:'WA',input:{...input(jobs.WA),document_json:imported('WA',2010)}}},/share a year/);
recorded.execute({operation:'abort-native-national-map',session:started.session});
refused({operation:'finish-native-national-map',session:started.session},/matching/);
for(const dpi of [72,150])for(const map_type of ['districts','political','demographic','compactness']){
 const selected={...batch,dpi,map_type,states:Object.fromEntries(Object.entries(jobs).map(([state,j])=>[state,{...j,analysis:map_type==='districts'?null:source(state+'-analysis.json',JSON.stringify({analyzer:map_type,state,year:2020,districts:[{district:1,dem_pct:.55,pct_minority:.25,polsby_popper:.3}]}))}]))},progress=[];
 const result=await executeNationalMapBatch(recorded,selected,{onProgress:p=>progress.push(p)});
 assert.deepEqual(progress.map(p=>p.states),[1,2,3]);assert.equal(result.units,3);assert.equal(result.districts,3);
 assert.deepEqual(result.provided_states.map(p=>p.state),['AK','HI','WA']);
 const legacy=engine.execute({operation:'render-native-national-map',input:{dpi,map_type,states:Object.fromEntries(Object.entries(selected.states).map(([state,j])=>[state,input(j,selected)]))}});
 for(const key of ['svg_text','png_base64','svg_sha256','png_sha256','provided_states','width','height'])assert.deepEqual(result[key],legacy[key]);
 if(dpi===72&&map_type==='districts'){
  const packageSource=source('synthetic-stream.json',JSON.stringify(selected));assert.deepEqual(nationalMapBatchInput(packageSource),selected);
  const project=createProject({name:'Synthetic national state stream',operation:'render-native-national-map-batch',lastOperation:'render-native-national-map-batch',constraints:[],files:{nationalMapBatchInputs:{source:packageSource}},result});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/synthetic.bisect',JSON.stringify(project));
 }
}
const broken=structuredClone(batch);broken.states.AK.geometry_format='gzip';broken.states.AK.geometry=source('broken.gz','bad gzip');await assert.rejects(()=>executeNationalMapBatch(recorded,broken));
// A failed host decode aborts its session, so another complete map can start.
await executeNationalMapBatch(recorded,batch);
const added=await addNativeMapToNationalBatch(null,{plan:jobs.AK.plan,geometry:jobs.AK.geometry,dpi:72,map_type:'districts',dissolve_method:'batch'});assert.equal(nationalMapBatchInput(added.source).states.AK.geometry_format,'gzip');
assert.throws(()=>validateNationalMapBatch({...batch,states:{XX:jobs.AK}}));
const originalArg=process.argv.find(a=>a.startsWith('--original-states='));let original=null;
if(originalArg){
 const states=originalArg.slice(18).split(','),selected={dpi:72,map_type:'districts',states:{}},legacyStates={};let bytes=0;
 for(const state of states){
  const fixture=state==='AK'?'target/original-alaska-fixed':'target/original-state-geometry-fixed/'+state,project=JSON.parse(await fs.readFile(fixture+'/workflow.bisect','utf8')),proof=JSON.parse(await fs.readFile(fixture+'/evidence.json','utf8')),geometry=Buffer.from(project.files.planGeometry.base64,'base64'),plan=project.result.practitioner_export.document_json;
  assert.equal(sha(geometry),proof.geometry_sha256);assert.equal(proof.full_native_wasm_workflow_match,true);assert.equal(proof.native_yaml_assignment_parity,true);
  selected.states[state]={plan:source(state+'.rplan',plan),geometry:source(state+'.geojson.gz',gzipSync(geometry)),geometry_format:'gzip',analysis:null,dissolve_method:'batch'};
  legacyStates[state]={document_json:plan,unit_geometry_json:geometry.toString(),dpi:72,map_type:'districts',dissolve_method:'batch'};bytes+=geometry.length+Buffer.byteLength(plan);
 }
 const result=await executeNationalMapBatch(recorded,selected),legacySource=source('original-large-national.json',JSON.stringify({dpi:72,map_type:'districts',states:legacyStates}));
 assert(Buffer.from(legacySource.base64,'base64').length>16*1024*1024);nativeNationalMapRequest(legacySource);await readNativeNationalMapSource({name:legacySource.name,size:Buffer.from(legacySource.base64,'base64').length,arrayBuffer:async()=>Buffer.from(legacySource.base64,'base64')});
 assert(result.source_positions>500_000,'The original-source stream must exceed the legacy position bound');assert.throws(()=>engine.execute(nativeNationalMapRequest(legacySource)),/500,000 source positions/);
 const packageSource=source('original-state-map-sources.json',JSON.stringify(selected)),project=createProject({name:'Original source national state stream',operation:'render-native-national-map-batch',lastOperation:'render-native-national-map-batch',constraints:[],files:{nationalMapBatchInputs:{source:packageSource}},result});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/original.bisect',JSON.stringify(project));await fs.writeFile(root+'/original-package.json',JSON.stringify(selected));await fs.writeFile(root+'/original-national.png',Buffer.from(result.png_base64,'base64'));
 original={states,units:result.units,districts:result.districts,raw_source_bytes:bytes,source_positions:result.source_positions,dissolved_positions:result.dissolved_positions,wasm_memory_bytes:engine.memoryBytes()};
}
const file=root+'/requests.jsonl';await fs.writeFile(file,requests.map(r=>JSON.stringify(r)).join('\n')+'\n');
const native=execFileSync('target/debug/examples/execute_tool_sequence.exe',[file],{encoding:'utf8',windowsHide:true,maxBuffer:256*1024*1024}).trim().split('\n').map(s=>JSON.parse(s));assert.deepEqual(native,outcomes);
await fs.writeFile(root+'/evidence.json',JSON.stringify({requests:requests.length,native_wasm_matches:outcomes.length,refusals:outcomes.filter(o=>!o.ok).length,legacy_small_map_output_parity:true,original,wasm_sha256:engine.artifactIdentity.sha256,scope:'Bounded instance-local source streaming; original selected-state capacity when explicitly requested. Full all-state original-source and browser capacity remain separate evidence.'},null,2));
console.log('National map stream: '+outcomes.length+' exact native/WASM protocol results, failed-session cleanup, legacy SVG/PNG/source equivalence and inert project sources passed.'+(original?' Original capacity: '+JSON.stringify(original):''));
