import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {labelImportRequest,labelImportPlanRequest} from '../../web/lab/label-import-files.js';
import {labelAnalysisFromExternalPlan,labelAnalysisRequest,labelAnalysisZip} from '../../web/lab/label-analysis-files.js';
import {labelReportFromAnalysis,labelReportRequest,labelReportZip} from '../../web/lab/label-report-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root=path.resolve('target/external-label-analysis-wasm');await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const sha=text=>createHash('sha256').update(text).digest('hex');
const source=(name,text)=>({name,base64:Buffer.from(text).toString('base64')});
let hostMatches=0,refusals=0;
async function exact(name,request){const file=path.join(root,name+'-request.json');await fs.writeFile(file,JSON.stringify(request));const result=engine.execute(request);const native=spawnSync(path.resolve('target/release/examples/execute_tool_request.exe'),[file],{encoding:'utf8',maxBuffer:64*1024*1024});assert.equal(native.status,0,native.stderr);assert.deepEqual(JSON.parse(native.stdout),result);hostMatches++;await fs.writeFile(path.join(root,name+'-result.json'),JSON.stringify(result,null,2));return result;}
const fixtures={};
for(const name of ['csv','zero-based','geojson','rplan','shapefile','shapefile-no-shx']){
 const project=JSON.parse(await fs.readFile('target/label-import-plan-wasm/'+name+'.bisect','utf8')),v=labelAnalysisFromExternalPlan(project.result,project.files.labelImportInputs,project.files.labelImportContext);v.created='2026-10-12T23:30:00Z';v.types=['manifest','contiguity','splits','statistics'];const request=labelAnalysisRequest(v),result=await exact(name,request);assert.equal(result.external_import_verified,true);assert(countResultTables(result).some(table=>table.rows?.some(row=>row[0]==='Recorded external-source check')));assert.equal(JSON.parse(result.index_json).run_index_sha256,project.result.bindings.index_sha256);assert(result.states.every(state=>state.status==='ok'));assert.equal(result.registry_promoted,true);assert(JSON.parse(result.registry_json).external_grid.analyzed.includes('2020'));await fs.writeFile(path.join(root,name+'.zip'),await labelAnalysisZip(result));const saved=createProject({name,operation:request.operation,files:{labelAnalysisInputs:v},constraints:[],result,lastOperation:request.operation});assert.deepEqual(parseProject(JSON.stringify(saved)),saved);await fs.writeFile(path.join(root,name+'.bisect'),JSON.stringify(saved));fixtures[name]={project,v,request,result};
}
const base=fixtures.csv.project,shape=fixtures.shapefile.project,wa=JSON.parse(base.result.context_json).units.unit_ids,vt=JSON.parse(shape.result.context_json).units.unit_ids;
const combined={...base.files.labelImportInputs,source:source('two-states.csv','GEOID,district\n'+[...wa.map((id,i)=>`${id},${Math.floor(i/3)+1}`),...vt.map((id,i)=>`${id},${i+1}`)].join('\n')+'\n')};
const imported=engine.execute(labelImportRequest(combined));let batch;
for(const [state,context]of [['washington',base.result.context_json],['vermont',shape.result.context_json]]){const selection={state_name:state,index_sha256:sha(imported.index_json),district_basis:'one-based',context:source(state+'.rctx',context)};const prepared=engine.execute(labelImportPlanRequest(combined,selection));batch=labelAnalysisFromExternalPlan(prepared,combined,selection,batch);}
batch.created='2026-10-12T23:30:00Z';batch.types=['manifest','contiguity','splits','statistics'];
const multiple=await exact('two-states',labelAnalysisRequest(batch));assert.equal(multiple.states.length,2);assert(multiple.states.every(s=>s.status==='ok'));
const filtered=await exact('selected-state',labelAnalysisRequest({...batch,states:['vermont']}));assert.equal(filtered.states.length,1);assert.equal(JSON.parse(filtered.index_json).states.washington.status,'skipped');
const unavailable=await exact('unavailable',labelAnalysisRequest({...batch,types:['compactness','election']}));assert(unavailable.states.every(s=>s.status==='unavailable'));assert(Object.values(JSON.parse(unavailable.index_json).states).every(s=>s.compactness==='unavailable'));
const reports=await labelReportFromAnalysis(multiple);reports.settings={generated_at:'2026-10-12T23:35:00Z',format:'html',audit_with_report:false};const report=await exact('reports',labelReportRequest(reports));assert.equal(report.schema_version,'bisect-label-report-v1');
assert(report.states.every(state=>state.status==='unavailable'));
const richFixture=JSON.parse(await fs.readFile('target/analysis-package-wasm/request.json','utf8')).input;
const richRequest=structuredClone(fixtures.csv.request);delete richRequest.input.types;const aligned=richRequest.input.states.washington;richRequest.input.states.washington={...richFixture,document_json:aligned.document_json,context_json:aligned.context_json};
const rich=await exact('rich',richRequest);assert.equal(rich.states[0].status,'ok');const manifest=JSON.parse(rich.files['analysis/external_grid/2020/washington/manifest.json']);assert.equal(manifest.partition_mode,'unrecorded');assert.equal(manifest.seed,null);
await fs.writeFile(path.join(root,'rich.zip'),await labelAnalysisZip(rich));const richInputs={source:source('external-rich-batch.json',JSON.stringify(richRequest.input)),created:richRequest.input.created};await fs.writeFile(path.join(root,'rich.bisect'),JSON.stringify(createProject({name:'Complete external analysis',operation:'run-label-analysis',files:{labelAnalysisInputs:richInputs},constraints:[],result:rich,lastOperation:'run-label-analysis'})));
const richReportInputs=await labelReportFromAnalysis(rich);richReportInputs.settings={generated_at:'2026-10-12T23:35:00Z',format:'html',audit_with_report:false};const generated=await exact('rich-report',labelReportRequest(richReportInputs));assert.equal(generated.states[0].status,'generated');assert(Object.keys(generated.files).some(name=>name.endsWith('.html')));await fs.writeFile(path.join(root,'rich-report.zip'),await labelReportZip(generated));
const yearBatches={};
for(const year of ['2010','2020']){
 const graph={schema_version:1,state:'VT',year,geoids:vt,adjacency:[[1],[0]],population:[100,100],edges:[[0,1,1.5]],areas:[10,10],exterior_perimeters:[1,1],centroids:[[-72,44],[-72.1,44]]};
 const options={structure:'standard-bisect',weights:'geographic',search:'single',districts:2,seed:null,seeds:2,steps:20,percentile:0,alpha_county:0,balance_tolerance:25,area_swing:1.1,iterations:10};
 const context=engine.execute({operation:'export-engine-plan',request:{graph,options},assignments:Object.fromEntries(vt.map((id,i)=>[id,i+1])),label:'Supplied context',chamber:'test',created_at:combined.settings.created}).context_json;
 const v={...shape.files.labelImportInputs,settings:{...shape.files.labelImportInputs.settings,year}},original=engine.execute(labelImportRequest(v));
 const selection={state_name:'vermont',index_sha256:sha(original.index_json),district_basis:'one-based',context:source(year+'.rctx',context)},prepared=engine.execute(labelImportPlanRequest(v,selection));
 const stateBatch=labelAnalysisRequest({...labelAnalysisFromExternalPlan(prepared,v,selection),created:'2026-10-12T23:30:00Z'}).input;delete stateBatch.registry_json;yearBatches[year]=stateBatch;
}
const yearsRequest={operation:'run-label-analysis-years',input:{created:'2026-10-12T23:30:00Z',types:['manifest','contiguity','splits','statistics'],registry_json:JSON.stringify({external_grid:{built:['2010','2020'],analyzed:[],reported:[]}}),batches:yearBatches}};
const years=await exact('years',yearsRequest);assert.equal(years.status,'completed');assert(years.years.every(y=>y.result.external_import_verified===true));const progress=[];assert.deepEqual(engine.execute(yearsRequest,{onProgress:e=>progress.push(e)}),years);assert.equal(progress.length,2);
const resumed=await exact('resumed',{operation:'resume-label-analysis-years',input:yearsRequest.input,checkpoint:progress[0].snapshot});assert.deepEqual(resumed,years);
await fs.writeFile(path.join(root,'years.zip'),await labelAnalysisZip(years));
const bad=[{...fixtures.csv.request,input:{...fixtures.csv.request.input,external_import:null}},{...fixtures.csv.request,input:{...fixtures.csv.request.input,build_index_json:fixtures.csv.request.input.build_index_json+' '} }];
for(const mutate of [r=>r.input.external_import.district_bases.washington='zero-based',r=>delete r.input.external_import.district_bases.washington,r=>r.input.external_import.district_bases.unknown='one-based',r=>r.input.states.washington.context_json=null,r=>r.input.states.washington.document_json=r.input.states.washington.document_json+' ',r=>{const p=JSON.parse(r.input.states.washington.document_json);p.plan.assignment.reverse();r.input.states.washington.document_json=JSON.stringify(p);},r=>r.input.external_import.input.source.base64=source('x','GEOID,district\n53001000100,1\n').base64]){const r=structuredClone(fixtures.csv.request);mutate(r);bad.push(r);}
const dbf=structuredClone(fixtures.shapefile.request);const bytes=Buffer.from(dbf.input.external_import.input.dbf.base64,'base64');bytes[bytes.length-2]^=1;dbf.input.external_import.input.dbf.base64=bytes.toString('base64');bad.push(dbf);
for(const request of bad){assert.throws(()=>engine.execute(request));refusals++;}
for(const mutate of [request=>request.input.batches['2010'].states.vermont.document_json+=' ',request=>{const original=request.input.batches['2010'].external_import.input.dbf;const bytes=Buffer.from(original.base64,'base64');bytes[bytes.length-2]^=1;original.base64=bytes.toString('base64');}]){const request={operation:'resume-label-analysis-years',input:structuredClone(yearsRequest.input),checkpoint:progress[0].snapshot};mutate(request);let events=0;assert.throws(()=>engine.execute(request,{onProgress:()=>events++}));assert.equal(events,0);refusals++;}
assert.throws(()=>labelAnalysisFromExternalPlan(shape.result,shape.files.labelImportInputs,shape.files.labelImportContext,batch));
await fs.writeFile(path.join(root,'two-states.bisect'),JSON.stringify(createProject({name:'External two-state analysis',operation:'run-label-analysis',files:{labelAnalysisInputs:batch,labelImportInputs:combined},constraints:[],result:multiple,lastOperation:'run-label-analysis'})));
await fs.writeFile(path.join(root,'two-states.zip'),await labelAnalysisZip(multiple));
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({hostMatches,refusals,all_four_formats:true,original_index_bytes:true,selected_states:true,unavailable_sources:true,native_reports:true,scope:'Synthetic nine-unit WA and two-unit VT contexts. Original-state capacity, disk registry installation and legal certification unverified.'},null,2));console.log(`External label analysis: ${hostMatches} complete native/WASM matches, ${refusals} source/context/plan/basis refusals, two-state selection, unavailable evidence and native reports passed.`);
