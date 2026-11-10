import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeMapRequest,nativeMapDownload,readNativeMapSource} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const root='target/native-map-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const real=process.argv.includes('--local-ri'),prior=JSON.parse(await fs.readFile(real?'target/full-state-compactness-wasm/RI.bisect':'target/plan-compactness-wasm/triangle.bisect','utf8')),inputs={plan:prior.files.auditPlanText,geometry:prior.files.compactnessInputs.geometry,dpi:150,dissolve_method:real?'balanced':'sequential'};
if(real){
 execFileSync('target/debug/examples/native_map_reference.exe',['--tiger','data/2020/tiger/tracts/tl_2020_44_tract/tl_2020_44_tract.shp',root+'/ri-native-geographic.json']);
 inputs.geometry={name:'ri-native-geographic.json',base64:(await fs.readFile(root+'/ri-native-geographic.json')).toString('base64')};
 assert.throws(()=>engine.execute(nativeMapRequest({...inputs,geometry:prior.files.compactnessInputs.geometry})),/longitude\/latitude/);
 assert.throws(()=>engine.execute(nativeMapRequest({...inputs,dissolve_method:'sequential'})),/work budget/);
 const simplified=JSON.parse(await fs.readFile('target/tract-preparation-wasm/ri2020-result.json','utf8')).geometry_json;assert.throws(()=>engine.execute(nativeMapRequest({...inputs,geometry:{name:'simplified.json',base64:Buffer.from(simplified).toString('base64')}})),/self-intersects/);
}
await fs.writeFile(root+'/plan.json',Buffer.from(inputs.plan.base64,'base64'));await fs.writeFile(root+'/geometry.json',Buffer.from(inputs.geometry.base64,'base64'));
for(const dpi of [72,100,150,200,300]){
 const value={...inputs,dpi},result=engine.execute(nativeMapRequest(value)),stem=root+'/'+(real?'ri':'fixture')+'-'+dpi;
 const dimensions=execFileSync('target/debug/examples/native_map_reference.exe',[root+'/plan.json',root+'/geometry.json',String(dpi),stem,value.dissolve_method],{encoding:'utf8'}).trim().split(' ').map(Number);assert.deepEqual([result.width,result.height],dimensions);assert.equal(Math.max(...dimensions),8*dpi);assert.equal(result.svg_text,await fs.readFile(stem+'.svg','utf8'));const png=Buffer.from(result.png_base64,'base64');assert.deepEqual(png,await fs.readFile(stem+'.png'));assert.equal(png.readUInt32BE(16),result.width);assert.equal(png.readUInt32BE(20),result.height);assert.equal(result.execution_performed,false);assert.equal(Buffer.from((await nativeMapDownload(result,'svg')).bytes).toString(),result.svg_text);assert.deepEqual(Buffer.from((await nativeMapDownload(result,'png')).bytes),png);
 const project=createProject({name:'Native map '+dpi,files:{nativeMapInputs:value},operation:'render-native-district-map',constraints:[],result,lastOperation:'render-native-district-map'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,project.files);await fs.writeFile(stem+'.bisect',JSON.stringify(project));await assert.rejects(()=>nativeMapDownload({...result,svg_text:result.svg_text+' '},'svg'),/changed/);await assert.rejects(()=>nativeMapDownload({...result,png_sha256:'0'.repeat(64)},'png'),/changed/);assert(!/<script|foreignObject|href=|onload=/i.test(result.svg_text));
}
if(!real){
 const geometry=JSON.parse(Buffer.from(inputs.geometry.base64,'base64')),jsonSource=value=>({name:'geometry.json',base64:Buffer.from(JSON.stringify(value)).toString('base64')});
 const baseline=engine.execute(nativeMapRequest(inputs));for(const kind of ['geoid','id']){const g=structuredClone(geometry);for(const f of g.features){const id=f.properties.GEOID;delete f.properties.GEOID;if(kind==='id')f.id=id;else f.properties.geoid=id;}assert.equal(engine.execute(nativeMapRequest({...inputs,geometry:jsonSource(g)})).svg_text,baseline.svg_text);}
 for(const change of [g=>g.features.pop(),g=>g.features[1].properties.GEOID=g.features[0].properties.GEOID,g=>g.features[0].properties.GEOID='unrelated',g=>g.features[0].properties.geoid='conflicting',g=>g.state='NC',g=>g.year='2030',g=>g.crs={type:'name',properties:{name:'EPSG:3857'}},g=>g.features[0].geometry.coordinates[0][0][0]=181]){const g=structuredClone(geometry);change(g);assert.throws(()=>engine.execute(nativeMapRequest({...inputs,geometry:jsonSource(g)})));}
 const many=structuredClone(geometry);many.features[0].geometry={type:'Polygon',coordinates:[Array.from({length:500001},()=>[0,0])]};const manyText=JSON.stringify(many),manySource=await readNativeMapSource({name:'many-positions.json',size:Buffer.byteLength(manyText),arrayBuffer:async()=>Buffer.from(manyText)},'geometry');assert.throws(()=>engine.execute(nativeMapRequest({...inputs,geometry:manySource})),/500,000 positions/);
 assert.throws(()=>nativeMapRequest({...inputs,dpi:73}));assert.throws(()=>engine.execute({operation:'render-native-district-map',input:{...nativeMapRequest(inputs).input,dpi:4294967295}}));await assert.rejects(()=>readNativeMapSource({name:'duplicate.json',size:13,arrayBuffer:async()=>Buffer.from('{"a":1,"a":2}')},'geometry'),/duplicate/i);
}
console.log(`${real?'Actual RI supplied geometry':'Synthetic nine-unit multipart/holes geometry'} native map: all five DPIs, exact native/WASM SVG and PNG bytes/dimensions, source/output hashes, project/archive retention, changed-output rejection and ${real?'250-tract native dissolve/render parity':'identity alias/universe/scope/CRS/coordinate/DPI/duplicate guards'} passed.`);
