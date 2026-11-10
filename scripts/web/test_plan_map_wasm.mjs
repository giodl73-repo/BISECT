import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
const preparedSite=process.argv[2]??'dist/wasm-merge-validation';
import {execFileSync} from 'node:child_process';
import {planMapModel} from '../../web/lab/plan-map.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject,parseGeometryJson} from '../../web/lab/project.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const text=await fs.readFile('crates/rplan-audit/fixtures/grid3x3-valid.rplan','utf8');
const request={operation:'validate-rplan-text',document_json:text};
await fs.mkdir('target/plan-map-wasm',{recursive:true});
await fs.writeFile('target/plan-map-wasm/request.json',JSON.stringify(request));
const output=engine.execute(request);
assert.deepEqual(output,JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',['target/plan-map-wasm/request.json'],{encoding:'utf8'})));
const document=JSON.parse(output.document_json),ids=document.plan.units.unit_ids;
const feature=(id,i)=>({type:'Feature',properties:{geoid:id},geometry:{type:'Polygon',coordinates:[[[i,0],[i+0.5,0],[i+0.5,0.5],[i,0]]]}});
const geometry={type:'FeatureCollection',features:ids.map(feature)};
const mapped=planMapModel(document,geometry);assert.equal(mapped.units,9);
const shuffled=planMapModel(document,{...geometry,features:[...geometry.features].reverse()});
for(const f of shuffled.features){assert.equal(f.district,document.plan.assignment[ids.indexOf(f.id)]);assert.equal(f.path,mapped.features.find(m=>m.id===f.id).path);}
let rejected=0;
for(const mutate of [g=>g.features.pop(),g=>g.features[0].properties.geoid='unrelated',g=>g.features[1].properties.geoid=ids[0],g=>g.crs={type:'name'},g=>g.year='wrong',g=>g.features[0].geometry.coordinates[0][0][0]=Infinity,g=>g.features[0].geometry.coordinates[0][1][1]=86,g=>g.features[0].geometry.coordinates[0].pop(),g=>g.features[0].geometry.type='Point',g=>g.features[0].geometry.coordinates=[]]){const bad=structuredClone(geometry);mutate(bad);assert.throws(()=>planMapModel(document,bad));rejected++;}
const holes=structuredClone(geometry);holes.features[0].geometry.coordinates.push([[0.1,0.1],[0.2,0.1],[0.2,0.2],[0.1,0.1]]);assert.equal(planMapModel(document,holes).features[0].path.split(' Z').length,3);
const source={name:'units.geojson',base64:Buffer.from(JSON.stringify(geometry)).toString('base64')};
const project=createProject({name:'Map test',files:{planGeometry:source,auditPlanText:{name:'plan.rplan',base64:Buffer.from(text).toString('base64')}},operation:'validate-rplan-text',constraints:[],result:output,lastOperation:'validate-rplan-text'});
assert.equal(parseProject(JSON.stringify(project)).files.planGeometry.base64,source.base64);
let real=0;
try{
 const catalog=JSON.parse(await fs.readFile(preparedSite+'/catalog.json','utf8'));
 for(const state of ['RI','IA','NC']){
  const plan=JSON.parse(await fs.readFile(`target/smc-real-state-wasm/${state}-selected.rplan`,'utf8'));
  const ref=catalog.geometries[`${state}:2020`];
  const path=typeof ref==='string'?ref:ref.path;
  const geo=parseGeometryJson(await fs.readFile(preparedSite+'/'+path,'utf8'));
  assert.equal(planMapModel(plan,geo).units,plan.plan.units.unit_ids.length);real++;
 }
}catch(error){if(process.argv[2]||error.code!=='ENOENT')throw error;}
console.log(`Map: native/WASM validation parity; reordered unit identity, holes, project byte retention; ${rejected} invalid geometry rejections; ${real} prepared-state maps.`);

for(const text of ['{\"features\":[],\"features\":[]}','{\"__proto__\":{}}','['.repeat(65)+'0'+']'.repeat(65)])assert.throws(()=>parseGeometryJson(text));
// Geometry has its own 16 MiB default, independent of the prepared-graph cap.
const geometryCap=16*1024*1024;
assert.equal(parseGeometryJson('{}'+' '.repeat(geometryCap-2)).constructor,Object);
assert.throws(()=>parseGeometryJson('x'.repeat(geometryCap+1)),/16 MiB/);
assert.throws(()=>parseGeometryJson(JSON.stringify('é'.repeat(geometryCap/2))),/16 MiB/);

for(const badText of ['{','x'.repeat(3*1024*1024+1),JSON.stringify({...document,plan:{...document.plan,k:501}}),JSON.stringify({...document,plan:{...document.plan,assignment:[...document.plan.assignment.slice(1)]}})]){
 const req={operation:'validate-rplan-text',document_json:badText};assert.throws(()=>engine.execute(req));await fs.writeFile('target/plan-map-wasm/request.json',JSON.stringify(req));assert.throws(()=>execFileSync('target/release/examples/execute_tool_request.exe',['target/plan-map-wasm/request.json'],{encoding:'utf8',stdio:['ignore','pipe','pipe']}));
}
console.log('Exact plan validation: four malformed/oversized/bounded-shape inputs rejected by both native and WASM.');
