import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {nativeMapRequest,nativeMapDownload,readNativeMapSource,validateNativeMapInputs} from '../../web/lab/map-export-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const real=process.argv.includes('--local-ri'),root='target/native-choropleth-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const prior=JSON.parse(await fs.readFile('target/native-map-wasm/'+(real?'ri':'fixture')+'-150.bisect','utf8')),inputs=prior.files.nativeMapInputs;
const plan=JSON.parse(Buffer.from(inputs.plan.base64,'base64')).plan;
const source=value=>({name:'analysis.json',base64:Buffer.from(typeof value==='string'?value:JSON.stringify(value)).toString('base64')});
const hash=text=>createHash('sha256').update(text).digest('hex');
const field={political:'dem_pct',demographic:'pct_minority',compactness:'polsby_popper'};
const prefix=root+'/'+(real?'ri':'fixture');await fs.writeFile(prefix+'-plan.json',Buffer.from(inputs.plan.base64,'base64'));await fs.writeFile(prefix+'-geometry.json',Buffer.from(inputs.geometry.base64,'base64'));
for(const map_type of Object.keys(field)){
 let analysis={analyzer:map_type,state:plan.units.state,year:String(plan.units.year),plan_hash:prior.result.plan_hash,districts:Array.from({length:plan.k},(_,i)=>({district:i+1,[field[map_type]]:i/(plan.k-1)}))};
 if(real&&map_type==='compactness')analysis=JSON.parse(await fs.readFile('target/full-state-compactness-wasm/native.json','utf8'));
 const analysisSource=source(analysis),text=Buffer.from(analysisSource.base64,'base64').toString();await fs.writeFile(prefix+'-analysis.json',text);
 for(const dpi of [72,100,150,200,300]){
  const settings={...inputs,dpi,map_type,analysis:analysisSource},result=engine.execute(nativeMapRequest(settings)),stem=prefix+'-'+map_type+'-'+dpi;
  const dimensions=execFileSync('target/debug/examples/native_map_reference.exe',[prefix+'-plan.json',prefix+'-geometry.json',String(dpi),stem,settings.dissolve_method||'sequential',map_type,prefix+'-analysis.json'],{encoding:'utf8'}).trim().split(' ').map(Number);
  assert.deepEqual([result.width,result.height],dimensions);assert.equal(result.svg_text,await fs.readFile(stem+'.svg','utf8'));assert.deepEqual(Buffer.from(result.png_base64,'base64'),await fs.readFile(stem+'.png'));assert.equal(result.map_type,map_type);assert.equal(result.analysis_source_sha256,hash(text));assert.deepEqual(result.fallback_districts,[]);assert.equal(result.execution_performed,false);assert.equal(Buffer.from((await nativeMapDownload(result,'svg')).bytes).toString(),result.svg_text);assert.notEqual(result.svg_text,prior.result.svg_text);
  const project=createProject({name:map_type+' native map',operation:'render-native-district-map',lastOperation:'render-native-district-map',constraints:[],files:{nativeMapInputs:settings},result});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,project.files);await fs.writeFile(stem+'.bisect',JSON.stringify(project));
 }
 // Native fallback semantics: absent row, missing/null/nonnumeric metric retain the original defaults.
 for(const districts of [[],[{district:1}],[{district:1,[field[map_type]]:null}],[{district:1,[field[map_type]]:'unavailable'}]]){
  const fallback={...analysis,districts},fallbackText=JSON.stringify(fallback);await fs.writeFile(prefix+'-analysis.json',fallbackText);const settings={...inputs,map_type,analysis:source(fallback)},result=engine.execute(nativeMapRequest(settings));assert.deepEqual(result.fallback_districts,Array.from({length:plan.k},(_,i)=>i+1));execFileSync('target/debug/examples/native_map_reference.exe',[prefix+'-plan.json',prefix+'-geometry.json','150',prefix+'-fallback',settings.dissolve_method||'sequential',map_type,prefix+'-analysis.json']);assert.equal(result.svg_text,await fs.readFile(prefix+'-fallback.svg','utf8'));assert.deepEqual(Buffer.from(result.png_base64,'base64'),await fs.readFile(prefix+'-fallback.png'));
 }
 for(const change of [v=>v.analyzer='wrong',v=>v.state='wrong',v=>v.year=2030,v=>v.plan_hash='wrong',v=>v.districts.push(v.districts[0]),v=>v.districts[0].district=0,v=>v.districts[0].district=plan.k+1,v=>v.districts=null]){const bad=structuredClone(analysis);change(bad);assert.throws(()=>engine.execute(nativeMapRequest({...inputs,map_type,analysis:source(bad)})));}
 assert.throws(()=>engine.execute(nativeMapRequest({...inputs,map_type})),/Select native analysis/);
 assert.throws(()=>engine.execute(nativeMapRequest({...inputs,map_type,analysis:source('{"districts":[],"districts":[]}')})),/Duplicate/);
 assert.throws(()=>engine.execute(nativeMapRequest({...inputs,map_type,analysis:source(' '.repeat(3*1024*1024+1))}))); // byte bound before parse
}
assert.throws(()=>engine.execute(nativeMapRequest({...inputs,analysis:source({districts:[]})})),/Clear analysis/);
assert.throws(()=>validateNativeMapInputs({...inputs,map_type:'splits'}));assert.throws(()=>engine.execute({...nativeMapRequest(inputs),input:{...nativeMapRequest(inputs).input,map_type:'splits'}}),/Unsupported/);assert.throws(()=>engine.execute({...nativeMapRequest(inputs),input:{...nativeMapRequest(inputs).input,map_type:'political',analysis_json:' '.repeat(3*1024*1024+1)}}),/3 MiB/);assert.throws(()=>validateNativeMapInputs({...inputs,extra:true}));
await assert.rejects(()=>readNativeMapSource({name:'large.json',size:3*1024*1024+1},'analysis'));
console.log(`${real?'Actual RI 250-unit original geography':'Synthetic multipart/holes fixture'}: three native choropleth types × five DPIs exact SVG/PNG bytes, analysis identity, fallback defaults, scope/district/duplicate/byte guards and inert project/archive retention passed.`);
