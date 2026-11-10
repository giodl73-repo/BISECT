// Synthetic islands on both sides of +/-180, with original Alaska opt-in.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {readNativeCampaignRuntime} from './native-campaign-runtime.mjs';
const root='target/antimeridian-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const reference=process.env.BISECT_NATIVE_REFERENCE||'target/release/examples/execute_tool_request.exe';
const sha=b=>createHash('sha256').update(b).digest('hex');
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const ring=(x,y,w=1,h=1)=>[[x,y],[x+w,y],[x+w,y+h],[x,y+h],[x,y]];
const geometry=JSON.stringify({type:'FeatureCollection',features:[{type:'Feature',properties:{GEOID:'02105000400'},geometry:{type:'MultiPolygon',coordinates:[[ring(174,52)],[ring(-178,52)],[ring(-151,60,2,2)]]}}]});
const imported=engine.execute({operation:'import-plan-interchange',source_text:'geoid,district\n02105000400,1\n',centroids_json:null,context_json:null,settings:{format:'dra-csv',state:'AK',year:2020,label:'synthetic-antimeridian',created_at:'2026-10-12T00:00:00Z',source_name:'assignments.csv',as_civic_counter_proposal:false,submitted_by:null,submitted_at:null}});
let matches=0;
async function exact(name,request){
 const file=root+'/'+name+'-request.json';await fs.writeFile(file,JSON.stringify(request));
 const result=engine.execute(request),native=JSON.parse(execFileSync(reference,[file],{encoding:'utf8',windowsHide:true,maxBuffer:128*1024*1024}));assert.deepEqual(result,native);matches++;return result;
}
function pathsWithin(svg,minX,maxX,minY,maxY){
 let count=0;for(const match of svg.matchAll(/<path d="([^"]+)"/g)){
  const values=match[1].match(/-?\d+(?:\.\d+)?/g).map(Number);assert.equal(values.length%2,0);
  for(let i=0;i<values.length;i+=2){assert(values[i]>=minX&&values[i]<=maxX,'Path longitude escaped display bounds');assert(values[i+1]>=minY&&values[i+1]<=maxY,'Path latitude escaped display bounds');count++;}
 }assert(count>0);return count;
}
for(const map_type of ['districts','political','demographic','compactness'])for(const dpi of [72,150]){
 const analysis_json=JSON.stringify({analyzer:map_type,state:'AK',year:2020,districts:[{district:1,dem_pct:.55,pct_minority:.25,polsby_popper:.3}]}),input={document_json:imported.document_json,unit_geometry_json:geometry,dpi,map_type,dissolve_method:'batch',...(map_type==='districts'?{}:{analysis_json})};
 const state=await exact('state-'+map_type+'-'+dpi,{operation:'render-native-district-map',input});assert.equal(state.width,8*dpi);assert.equal(state.height,Math.round(8*dpi*10/37));pathsWithin(state.svg_text,0,state.width,0,state.height);assert.equal(state.geometry_source_sha256,sha(geometry));
 const nationalInput={dpi,map_type,states:{AK:input}},national=await exact('national-'+map_type+'-'+dpi,{operation:'render-native-national-map',input:nationalInput});assert.equal(national.provided_states.length,1);pathsWithin(national.svg_text,0,national.width*.28,national.height*.75,national.height);
 if(map_type==='districts'){
  const project=createProject({name:'Synthetic antimeridian islands',operation:'render-native-district-map',constraints:[],lastOperation:'render-native-district-map',files:{nativeMapInputs:{plan:source('synthetic.rplan',imported.document_json),geometry:source('synthetic-islands.geojson',geometry),dpi,dissolve_method:'batch'}},result:state});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/state-'+dpi+'.bisect',JSON.stringify(project));
  const nationalProject=createProject({name:'Synthetic Aleutian inset',operation:'render-native-national-map',constraints:[],lastOperation:'render-native-national-map',files:{nativeNationalMapSource:source('synthetic-national.json',JSON.stringify(nationalInput))},result:national});assert.deepEqual(parseProject(JSON.stringify(nationalProject)),nationalProject);await fs.writeFile(root+'/national-'+dpi+'.bisect',JSON.stringify(nationalProject));
 }
}
let counterfactual=null;
if(process.argv.includes('--counterfactual')){
const runtime=await readNativeCampaignRuntime('target/original-all-state-wasm'),old=await instantiateEngine(await fs.readFile(runtime.wasm));
const baselineInput={document_json:imported.document_json,unit_geometry_json:geometry,dpi:72,map_type:'districts',dissolve_method:'batch'};
const oldState=old.execute({operation:'render-native-district-map',input:baselineInput});assert(oldState.height<30);
const oldNational=old.execute({operation:'render-native-national-map',input:{dpi:72,map_type:'districts',states:{AK:baselineInput}}});
assert.throws(()=>pathsWithin(oldNational.svg_text,0,oldNational.width*.28,oldNational.height*.75,oldNational.height),/Path longitude escaped display bounds/);
const compactness={operation:'analyze-plan-compactness',document_json:imported.document_json,unit_geometry_json:geometry,coordinate_basis:'wgs84-degrees',crs_label:'EPSG:4326',dissolve_method:'batch'};
assert.deepEqual(engine.execute(compactness),old.execute(compactness));
counterfactual={old_state_height:oldState.height,old_inset_fails_bounds:true,unchanged_compactness:true};
}
if(process.argv.includes('--original-ak')){
 const project=JSON.parse(await fs.readFile('target/original-alaska-fixed/workflow.bisect','utf8'));
 const geometry=Buffer.from(project.files.planGeometry.base64,'base64').toString('utf8'),plan=project.result.practitioner_export.document_json;
 const request={operation:'render-native-national-map',input:{dpi:72,map_type:'districts',states:{AK:{document_json:plan,unit_geometry_json:geometry,dpi:72,map_type:'districts',dissolve_method:'batch'}}}};
 const result=await exact('original-ak-national',request);assert.equal(result.units,177);pathsWithin(result.svg_text,0,result.width*.28,result.height*.75,result.height);
 const file=root+'/original-ak-national.json';await fs.writeFile(file,JSON.stringify(request.input));execFileSync('target/debug/examples/native_national_map_reference.exe',[file,root+'/original-ak-native'],{windowsHide:true});assert.equal(result.svg_text,await fs.readFile(root+'/original-ak-native.svg','utf8'));assert.deepEqual(Buffer.from(result.png_base64,'base64'),await fs.readFile(root+'/original-ak-native.png'));
 await fs.writeFile(root+'/original-ak-national.bisect',JSON.stringify(createProject({name:'Original Alaska national inset',operation:request.operation,lastOperation:request.operation,constraints:[],files:{nativeNationalMapSource:source('original-ak-national.json',JSON.stringify(request.input))},result})));
}
await fs.writeFile(root+'/evidence.json',JSON.stringify({matches,wasm_sha256:engine.artifactIdentity.sha256,counterfactual,original_alaska:process.argv.includes('--original-ak'),scope:'Display-only antimeridian correction. Synthetic islands plus optional complete original prepared Alaska input. Source geometries, assignments and analytical measurements retain original coordinates; full nationwide engine capability remains a separate requirement.'},null,2));
console.log('Antimeridian WASM: '+matches+' whole native matches, state bounds/Aleutian inset path visibility and exact sources/projects'+(counterfactual?', pinned-runtime compactness equality':'')+' passed.');
