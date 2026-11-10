import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {configuredRunMapInputs} from '../../web/lab/configured-map-files.js';
import {nativeMapRequest,nativeMapDownload} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const make=()=>({schema_version:'bisect-configured-run-v1',engine_request:{graph:{state:'RI',year:'2020',geoids:['a','b']},options:{districts:2}},result:{assignments:{a:1,b:2}},practitioner_export:{schema_version:'bisect-configured-plan-export-v1',document_json:JSON.stringify({plan:{k:2,units:{state:'RI',year:2020,unit_kind:'tract',unit_ids:['a','b']},assignment:[0,1]}})}});
const result=make(),before=structuredClone(result),next=configuredRunMapInputs(result);
assert.equal(Buffer.from(next.plan.base64,'base64').toString(),result.practitioner_export.document_json);
assert.equal(next.geometry,null);assert.equal(next.analysis,null);assert.equal(next.round_assignments,null);assert.equal(next.map_type,'districts');assert.equal(next.source_mode,undefined);
assert.deepEqual(result,before);
for(const edit of [p=>p.units.state='GB',p=>p.units.year=2010,p=>p.units.unit_kind='imported',p=>p.units.unit_ids.reverse(),p=>p.assignment.reverse(),p=>p.k=3]){
 const candidate=make(),doc=JSON.parse(candidate.practitioner_export.document_json);edit(doc.plan);candidate.practitioner_export.document_json=JSON.stringify(doc);assert.throws(()=>configuredRunMapInputs(candidate),/does not match/);
}
const missing=make();delete missing.practitioner_export;assert.throws(()=>configuredRunMapInputs(missing),/Prepare native files/);
const geometry=source('geometry.json','{"type":"FeatureCollection","features":[]}'),withGeometry=configuredRunMapInputs(result,geometry);assert.deepEqual(withGeometry.geometry,geometry);withGeometry.geometry.name='changed';assert.equal(geometry.name,'geometry.json');
const ancient=make();ancient.engine_request.graph.year='0200';const doc=JSON.parse(ancient.practitioner_export.document_json);doc.plan.units.year=200;ancient.practitioner_export.document_json=JSON.stringify(doc);configuredRunMapInputs(ancient);
console.log('PASS configured map handoff: exact bytes, scope/unit/assignment binding, no-geometry path, fresh map settings and immutable sources');
if(process.argv.includes('--native')){
 const root='target/configured-map-handoff';await fs.mkdir(root,{recursive:true});
 const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
 for(const [state,kind]of [['GB','precinct'],['ZZ','imported'],['RI','precinct']]){
  const stem=state==='RI'?'target/native-custom-units/'+kind+'-2024':'target/custom-jurisdiction-engine/'+state+'-'+kind;
  const configured=JSON.parse(await fs.readFile(stem+'-configured-result.json','utf8'));
  const ids=configured.engine_request.graph.geoids;
  const geo={type:'FeatureCollection',state,year:2024,features:ids.map((geoid,i)=>{const x=-3+(i%25)*.002,y=52+Math.floor(i/25)*.002;return {type:'Feature',properties:{geoid},geometry:{type:'Polygon',coordinates:[[[x,y],[x+.001,y],[x+.001,y+.001],[x,y+.001],[x,y]]]}};})};
  const inputs=configuredRunMapInputs(configured,source('SYNTHETIC-'+state+'-geometry.json',JSON.stringify(geo))),request=nativeMapRequest(inputs);
  inputs.dpi=72;request.input.dpi=72;
  const nativeRequest=root+'/'+state+'.request.json';await fs.writeFile(nativeRequest,JSON.stringify(request));
  const actual=engine.execute(request),proc=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[nativeRequest],{encoding:'utf8',windowsHide:true,timeout:180000,maxBuffer:64*1024*1024});assert.equal(proc.status,0,proc.stderr);assert.deepEqual(actual,JSON.parse(proc.stdout));
  for(const kind of ['png','svg']){const output=await nativeMapDownload(actual,kind);await fs.writeFile(root+'/'+state+'.'+kind,output.bytes);}
  const packet=createProject({name:'SYNTHETIC '+state+' configured map handoff',operation:'run-algorithm-config',lastOperation:'run-algorithm-config',files:{...JSON.parse(await fs.readFile(stem+'.bisect','utf8')).files,planGeometry:inputs.geometry},constraints:[],result:configured});assert.deepEqual(parseProject(JSON.stringify(packet)),packet);await fs.writeFile(root+'/'+state+'.bisect',JSON.stringify(packet));await fs.writeFile(root+'/'+state+'-map.json',JSON.stringify(actual));
 }
 await fs.writeFile(root+'/evidence.json',JSON.stringify({wasm:engine.artifactIdentity,native_whole_maps:3,synthetic_custom_units_and_geometry:true,downloads_verified:true,source_projects_roundtrip:true,published:false},null,2)+'\n');
 console.log('PASS configured map handoff: three complete native/WASM map replies and original PNG/SVG downloads');
}
