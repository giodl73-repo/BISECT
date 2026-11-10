// Explicit local source validation; this gate never downloads or relabels data.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {censusPreparationRequest,preparedTractRunSources} from '../../web/lab/tract-input-files.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {TUI_RUN_FORM_DEFAULTS,tuiRunRequest} from '../../web/lab/tui-run-files.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const kind=process.argv[2]||'block-group';
const prefixes={tract:'data/2020/tiger/tracts/tl_2020_44_tract/tl_2020_44_tract','block-group':'target/authentic-bg/RI/tiger/tl_2020_44_bg',block:'target/real-census-preparation/RI/block/tl_2020_44_tabblock20',county:'target/real-census-preparation/county-national/tl_2020_us_county'};
assert(Object.hasOwn(prefixes,kind),'Supply tract, block-group or block with original local RI Census sources.');
const root=path.resolve('target/real-census-preparation/RI/'+kind);await fs.mkdir(root,{recursive:true});
const prefix=prefixes[kind],geo='data/2020/redistricting/rhode_island/rigeo2020.pl',pop='data/2020/redistricting/rhode_island/ri000012020.pl';
const source=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')}),hash=text=>createHash('sha256').update(text).digest('hex');
const tiger={kind:{tract:'tracts','block-group':'block-groups',block:'blocks',county:'counties'}[kind],year:2020};
for(const part of ['shp','dbf','shx'])tiger[part]=source(path.basename(prefix)+'.'+part+'.gz',gzipSync(await fs.readFile(prefix+'.'+part)));
const files={tigerInputs:tiger,pl94Inputs:{year:2020,geography:source('rigeo2020.pl.gz',gzipSync(await fs.readFile(geo))),population:source('ri000012020.pl.gz',gzipSync(await fs.readFile(pop)))},tractPreparationSettings:{state:'RI',projection:source(path.basename(prefix)+'.prj',await fs.readFile(prefix+'.prj'))}};
const wasm=await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'),engine=await instantiateEngine(wasm),start=performance.now();
console.log('Starting original RI 2020 '+kind+' source preparation.');
const prepared=process.argv.includes('--reuse-prepared')?JSON.parse(await fs.readFile(root+'/prepared.json','utf8')):engine.execute(censusPreparationRequest(files));await fs.writeFile(root+'/prepared.json',JSON.stringify(prepared));const graph=JSON.parse(prepared.graph_json);
assert.equal(prepared.population,'1097379');assert.equal(prepared.unit_kind,kind);assert.equal(prepared.execution_performed,false);
const native=JSON.parse(execFileSync('target/debug/examples/census_prepare_reference.exe',['RI','2020',kind,prefix+'.shp',geo,pop],{encoding:'utf8',maxBuffer:128*1024*1024}));
for(const key of ['schema_version','unit_kind','state','year','geoids','population','adjacency','areas'])assert.deepEqual(graph[key],native.graph[key]);
let maximumDelta=0;function numeric(a,b){if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>numeric(v,b[i]));}else{const delta=Math.abs(a-b);assert(Number.isFinite(delta)&&delta<=1e-5);maximumDelta=Math.max(maximumDelta,delta);}}
assert.deepEqual(graph.edges.map(e=>e.slice(0,2)),native.graph.edges.map(e=>e.slice(0,2)));numeric(graph.edges.map(e=>e[2]),native.graph.edges.map(e=>e[2]));numeric(graph.centroids,native.graph.centroids);numeric(graph.exterior_perimeters,native.graph.exterior_perimeters);assert.deepEqual(JSON.parse(prepared.geometry_json),native.geometry);assert.deepEqual(prepared.bridges.map(b=>[b.from,b.to]),native.bridges.map(b=>[b.from,b.to]));numeric(prepared.bridges.map(b=>b.cost),native.bridges.map(b=>b.cost));
assert.equal(prepared.graph_sha256,hash(prepared.graph_json));assert.equal(prepared.geometry_sha256,hash(prepared.geometry_json));
const next=preparedTractRunSources(prepared),template=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,seed:'42',workers:1,years:['2020']}));
Object.assign(files,{configSource:source(template.generated_filename,template.yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:next.graph},configRunSettings:{...next.settings,districts:kind==='county'?1:2,native_build:true,capture_rounds:false},planGeometry:next.geometry,tuiRunSettings:{form:{...TUI_RUN_FORM_DEFAULTS,location:'RI',year:'2020',resolution:kind==='block-group'?'block_group':kind,seed:'42',label:'real_ri_'+kind.replace('-','_')},created_at:'2026-10-08T00:00:00Z'}});
console.log('Prepared '+prepared.units+' units; starting typed native-retry Run.');
if(kind==='county'){const two=tuiRunRequest(files.tuiRunSettings,files.configSource,files.configSettings,files.configRunInputs,{...files.configRunSettings,districts:2});assert.throws(()=>engine.execute(two),/balance .*exceeds .*research limit/);}
const run=engine.execute(tuiRunRequest(files.tuiRunSettings,files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings));await fs.writeFile(root+'/run.json',JSON.stringify(run));assert.equal(run.status,'completed');assert.equal(run.configured_run.result.metrics.contiguous,true);assert.equal(run.configured_run.result.metrics.within_requested_tolerance,true);
const oraclePath=root+'/oracle.json';await fs.writeFile(oraclePath,JSON.stringify({graph,yaml_text:run.effective_yaml,options:{districts:files.configRunSettings.districts},attempt:run.configured_run.build_report.accepted_attempt}));const oracle=JSON.parse(execFileSync('target/debug/examples/config_seed_reference.exe',[oraclePath],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(run.configured_run.result.assignments,oracle.assignments);
const plan=JSON.parse(run.practitioner_export.document_json),context=JSON.parse(run.practitioner_export.context_json);assert.equal(plan.plan.units.unit_kind,kind);assert.equal(context.units.unit_kind,kind);assert.deepEqual(JSON.parse(run.practitioner_export.context_json),run.practitioner_export.context);assert.equal(context.context_hash,run.practitioner_export.context_hash);assert(Buffer.byteLength(run.practitioner_export.context_json)<=8*1024*1024);assert.deepEqual(plan.plan.units.unit_ids,graph.geoids);assert.deepEqual(context.populations,graph.population);
const archive=createWorkbenchArchive({name:'Original RI '+kind+' sources and native Run',files,operation:'run-tui-configured',result:run,lastOperation:'run-tui-configured'});assert.deepEqual(parseWorkbenchArchive(archive).files,files);await fs.writeFile(root+'/run.bisect-workbench',archive);
const receipt={wasm_sha256:hash(wasm),preparation_executed:!process.argv.includes('--reuse-prepared'),state:'RI',year:2020,unit_kind:kind,units:prepared.units,population:prepared.population,edges:prepared.edges,bridges:prepared.bridges.length,graph_bytes:Buffer.byteLength(prepared.graph_json),geometry_bytes:Buffer.byteLength(prepared.geometry_json),archive_bytes:Buffer.byteLength(archive),memory_bytes:engine.memoryBytes(),elapsed_seconds:(performance.now()-start)/1000,maximum_projected_delta_m:maximumDelta,accepted_attempt:run.configured_run.build_report.accepted_attempt,metrics:run.configured_run.result.metrics,native_assignment_parity:true,scope:'Original local Census source/native filesystem graph parity and native-retry configured Run at selected RI resolution; memory includes this process preparation/run/serialization, not browser peak; no full-state national capacity or source authenticity certification claim.'};await fs.writeFile(root+'/receipt.json',JSON.stringify(receipt,null,2));console.log(JSON.stringify(receipt));
