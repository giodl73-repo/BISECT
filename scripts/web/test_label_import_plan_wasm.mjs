import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {LABEL_IMPORT_DEFAULTS,labelImportRequest,labelImportPlanRequest,labelImportPlanZip,validateLabelImportContext} from '../../web/lab/label-import-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/label-import-plan-wasm');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const context=await fs.readFile('crates/rplan-audit/fixtures/grid3x3.rctx','utf8');
const raw=JSON.parse(context),ids=raw.units.unit_ids,labels=ids.map((_,i)=>Math.floor(i/3)+1);
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
const sha=text=>createHash('sha256').update(text).digest('hex');
const inputs={source:source('external.csv','GEOID,district\n'+ids.map((id,i)=>`${id},${labels[i]}`).reverse().join('\n')+'\n'),dbf:null,shx:null,registry:null,settings:{...LABEL_IMPORT_DEFAULTS,label:'external_grid',created:'2026-10-12T23:00:00Z'}};
let hostMatches=0,refusals=0;
async function exact(name,v,basis='one-based',rctx=context,state='washington',expectedAssignment=[0,0,0,1,1,1,2,2,2]){
 const imported=engine.execute(labelImportRequest(v));
 const selection={state_name:state,index_sha256:sha(imported.index_json),district_basis:basis,context:source(name==='csv'?'grid3x3.rctx':name+'.rctx',rctx)};
 const request=labelImportPlanRequest(v,selection),result=engine.execute(request),filename=path.join(root,name+'-request.json');await fs.writeFile(filename,JSON.stringify(request));
 const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[filename],{encoding:'utf8',maxBuffer:32*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),result);hostMatches++;
 const document=JSON.parse(result.document_json);assert.deepEqual(document.plan.assignment,expectedAssignment);assert.deepEqual(document.plan.units,JSON.parse(rctx).units);assert.equal(result.context_json,rctx);assert.equal(result.files['imported-assignments.json'],imported.files['runs/external_grid/2020/'+state+'/assignments.json']);assert.equal(result.files['import-index.json'],imported.index_json);assert.equal(document.provenance.producer.method,'external-label-import-context-alignment');assert.equal(document.provenance.producer.engine_options,undefined);assert.equal(result.execution_performed,false);assert.equal(result.bindings.context_sha256,sha(rctx));
 await fs.writeFile(path.join(root,name+'-result.json'),JSON.stringify(result,null,2));await fs.writeFile(path.join(root,name+'.zip'),await labelImportPlanZip(result,v,selection));
 const project=createProject({name,operation:'prepare-label-import-plan',files:{labelImportInputs:v,labelImportContext:selection},constraints:[],lastOperation:'prepare-label-import-plan',result});await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(project));assert.deepEqual(parseProject(JSON.stringify(project)).files.labelImportContext,selection);
 return {request,result,selection};
}
const csv=await exact('csv',inputs);
await exact('zero-based',{...inputs,source:source('zero.csv','GEOID,district\n'+ids.map((id,i)=>`${id},${labels[i]-1}`).join('\n')+'\n')},'zero-based');
await exact('geojson',{...inputs,source:source('external.geojson',JSON.stringify({type:'FeatureCollection',features:ids.map((id,i)=>({type:'Feature',properties:{GEOID:id,district:labels[i]},geometry:null}))}))});
await exact('rplan',{...inputs,source:source('external.rplan',csv.result.document_json)});
const shapeIds=['50001000100','50001000200'];
const graph={schema_version:1,state:'VT',year:'2020',geoids:shapeIds,adjacency:[[1],[0]],population:[100,100],edges:[[0,1,1.5]],areas:[10,10],exterior_perimeters:[1,1],centroids:[[-72,44],[-72.1,44]]};
const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:null,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
const shapeContext=engine.execute({operation:'export-engine-plan',request:{graph,options},assignments:Object.fromEntries(shapeIds.map((id,i)=>[id,i+1])),label:'Supplied synthetic context',chamber:'test',created_at:inputs.settings.created}).context_json;
const shapeInputs={...inputs,source:source('vermont_two_tracts.shp',await fs.readFile('docs/fixtures/import-label/positive/vermont_two_tracts.shp')),dbf:source('vermont_two_tracts.dbf',await fs.readFile('docs/fixtures/import-label/positive/vermont_two_tracts.dbf')),shx:source('vermont_two_tracts.shx',await fs.readFile('docs/fixtures/import-label/positive/vermont_two_tracts.shx'))};
await exact('shapefile',shapeInputs,'one-based',shapeContext,'vermont',[0,1]);
await exact('shapefile-no-shx',{...shapeInputs,shx:null},'one-based',shapeContext,'vermont',[0,1]);
const changeContext=modify=>{const c=JSON.parse(context);modify(c);return JSON.stringify(c);};
const bad=[{...csv.request,input:{...csv.request.input,state_name:'vermont'}},{...csv.request,input:{...csv.request.input,state_name:'../washington'}},{...csv.request,input:{...csv.request.input,expected_index_sha256:'0'.repeat(64)}},{...csv.request,input:{...csv.request.input,district_basis:'zero-based'}},{...csv.request,input:{...csv.request.input,district_basis:'auto'}},{...csv.request,input:{...csv.request.input,context_json:changeContext(c=>c.populations[0]++)}},{...csv.request,input:{...csv.request.input,context_json:changeContext(c=>c.units.year=2010)}},{...csv.request,input:{...csv.request.input,context_json:changeContext(c=>c.units.unit_ids.reverse())}},{...csv.request,input:{...csv.request.input,context_json:' '.repeat(8*1024*1024+1)}},{...csv.request,extra:true},{...csv.request,input:{...csv.request.input,extra:true}}];
for(const labelsText of [ids.slice(1).map(id=>`${id},1`),ids.map(id=>`${id},2`),ids.map(id=>`${id},501`),[...ids.map(id=>`${id},1`),'53001001000,1']]){const v={...inputs,source:source('bad.csv','GEOID,district\n'+labelsText.join('\n')+'\n')};const imported=engine.execute(labelImportRequest(v));bad.push(labelImportPlanRequest(v,{...csv.selection,index_sha256:sha(imported.index_json)}));}
for(const request of bad){assert.throws(()=>engine.execute(request));refusals++;}
assert.throws(()=>validateLabelImportContext({...csv.selection,unexpected:true}));
const tampered=structuredClone(csv.result);tampered.files['context.rctx']+=' ';await assert.rejects(()=>labelImportPlanZip(tampered,inputs,csv.selection));
const analysis=engine.execute({operation:'analyze-plan-package',input:{document_json:csv.result.document_json,context_json:context,geography_json:null,demographic_csv:null,urban_csv:null,election_csv:null,tolerance_percent:1,election_settings:{bootstrap_seed:'42',bootstrap_replicates:1000},compactness:null}});
await fs.writeFile(path.join(root,'analysis-result.json'),JSON.stringify(analysis,null,2));assert.equal(analysis.schema_version,'bisect-plan-analysis-package-v1');assert.equal(analysis.execution_performed,undefined);
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({hostMatches,refusals,context_byte_exact:true,external_provenance:true,scope:'Nine-unit native grid fixture: CSV, GeoJSON, RPLAN and both numbering bases; two-unit shapefile fixture with and without SHX. Native shared kernel additionally tests all six unit kinds. No original-state capacity or authentication claim.'},null,2));
console.log(`Imported-plan bridge: ${hostMatches} full native/WASM matches, ${refusals} refusals, exact context, native analysis and inert source-preserving projects passed.`);
