import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {gzipSync} from 'node:zlib';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {tractPreparationRequest,preparedTractRunSources,tractPreparationDownload} from '../../web/lab/tract-input-files.js';
import {CONFIG_TEMPLATE_DEFAULTS,configTemplateRequest} from '../../web/lab/config-template.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/tract-preparation-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const hash=b=>createHash('sha256').update(b).digest('hex'),source=(name,b)=>({name,base64:Buffer.from(b).toString('base64')});
const prj='GEOGCS["GCS_North_American_1983",DATUM["D_North_American_1983"],UNIT["Degree",0.0174532925199433]]';
let maxDelta=0;
function floats(a,b){if(Array.isArray(a)){assert.equal(a.length,b.length);a.forEach((v,i)=>floats(v,b[i]));}else{assert.equal(Number.isFinite(a)&&Number.isFinite(b),true);maxDelta=Math.max(maxDelta,Math.abs(a-b));assert.equal(a,b,'Prepared geometry must match native exactly under shared portable math');}}
function graphParity(a,b){for(const key of ['schema_version','state','year','geoids','population','adjacency','areas'])assert.deepEqual(a[key],b[key]);assert.deepEqual(a.edges.map(e=>e.slice(0,2)),b.edges.map(e=>e.slice(0,2)));floats(a.edges.map(e=>e[2]),b.edges.map(e=>e[2]));floats(a.centroids,b.centroids);floats(a.exterior_perimeters,b.exterior_perimeters);}
async function check(name,files,shpPath,geoPath,popPath){
 const request=tractPreparationRequest(files),result=engine.execute(request);await fs.writeFile(root+'/request.json',JSON.stringify(request));const options={encoding:'utf8',maxBuffer:128*1024*1024},host=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],options)),native=JSON.parse(execFileSync('target/debug/examples/tract_prepare_reference.exe',[files.tractPreparationSettings.state,String(files.pl94Inputs.year),shpPath,geoPath,popPath],options));
 const graph=JSON.parse(result.graph_json),hostGraph=JSON.parse(host.graph_json);graphParity(hostGraph,graph);assert.deepEqual(native.graph,hostGraph);assert.deepEqual(native.geometry,JSON.parse(result.geometry_json));assert.deepEqual(host.geometry_json,result.geometry_json);assert.deepEqual(host.sources,result.sources);assert.deepEqual(native.population,result.population);assert.deepEqual(result.bridges.map(b=>[b.from,b.to]),native.bridges.map(b=>[b.from,b.to]));floats(result.bridges.map(b=>b.cost),native.bridges.map(b=>b.cost));assert.equal(result.graph_sha256,hash(result.graph_json));assert.equal(host.graph_sha256,hash(host.graph_json));assert.equal(result.geometry_sha256,hash(result.geometry_json));assert.equal(result.execution_performed,false);assert.equal(Buffer.from(tractPreparationDownload(result,'graph').bytes).toString(),result.graph_json);
 const project=createProject({name:'Tract inputs '+name,files,operation:'prepare-tract-engine-input',constraints:[],result,lastOperation:'prepare-tract-engine-input'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));return {result,graph,files};
}
for(const year of [2020,2010,2000]){
 const tiger=JSON.parse(await fs.readFile('target/tiger-source-wasm/tracts-tracts.bisect','utf8')).files.tigerInputs;tiger.year=year;
 const pl=JSON.parse(await fs.readFile('target/pl94-source-wasm/'+year+'.bisect','utf8')).files.pl94Inputs;
 await fs.writeFile(root+'/geo.pl',Buffer.from(pl.geography.base64,'base64'));await fs.writeFile(root+'/pop.pl',pl.population?Buffer.from(pl.population.base64,'base64'):Buffer.from('unused'));
 const files={tigerInputs:tiger,pl94Inputs:pl,tractPreparationSettings:{state:'RI',projection:source('units.prj',prj)}};const prepared=await check(String(year),files,'target/tiger-source-wasm/sources/tracts/units.shp',root+'/geo.pl',root+'/pop.pl');assert.deepEqual(prepared.graph.population,[11,7]);assert.equal(prepared.result.population,'18');
 const gzipFiles={...files,pl94Inputs:{...pl,geography:source('geo.pl.gz',gzipSync(Buffer.from(pl.geography.base64,'base64'))),population:pl.population?source('pop.pl.gz',gzipSync(Buffer.from(pl.population.base64,'base64'))):null}};const zipped=await check(year+'-gzip',gzipFiles,'target/tiger-source-wasm/sources/tracts/units.shp',root+'/geo.pl',root+'/pop.pl');assert.equal(zipped.result.graph_json,prepared.result.graph_json);assert.equal(zipped.result.sources.geography_decoded.decoded_sha256,hash(Buffer.from(pl.geography.base64,'base64')));
}
const valid=JSON.parse(await fs.readFile(root+'/2020.bisect','utf8')).files,request=tractPreparationRequest(valid);
for(const input of [{...request.input,state:'IA'},{...request.input,tiger:{...request.input.tiger,year:2010}},{...request.input,projection:source('bad.prj','PROJCS["wrong"]')},{...request.input,pl94:{...request.input.pl94,population:null}}])assert.throws(()=>engine.execute({operation:request.operation,input}));
const corrupted=gzipSync(Buffer.from(valid.pl94Inputs.geography.base64,'base64'));corrupted[corrupted.length-8]^=1;assert.throws(()=>engine.execute({...request,input:{...request.input,pl94:{...request.input.pl94,geography:source('bad.gz',corrupted)}}}));
const extraPopulation=source('foreign.pl',Buffer.from(valid.pl94Inputs.population.base64,'base64').toString()+'PLST|AA|000|01|9999999|1\n'); // Unmatched segment records follow native skip semantics.
assert.equal(engine.execute({...request,input:{...request.input,pl94:{...request.input.pl94,population:extraPopulation}}}).population,'18');
// Exercise the shared join and accumulation guards using valid native segment syntax.
for(const [first,second,message] of [[0,0,/full state population/],[-1,7,/nonnegative native engine/],[2147483648,7,/native engine weights/],['9223372036854775807',7,/State population overflow/]]){
 const text=Buffer.from(valid.pl94Inputs.population.base64,'base64').toString().replace('|0000001|11|','|0000001|'+first+'|').replace('|0000002|7|','|0000002|'+second+'|');
 assert.throws(()=>engine.execute({...request,input:{...request.input,pl94:{...request.input.pl94,population:source('guard.pl',text)}}}),message);
}
console.log('Tract preparation: six native-host/WASM/filesystem graph cases across three vintages/plain+gzip, exact population/IDs/adjacency/areas/display map, exact projected floats (maximum delta '+maxDelta+'), source/hash/project/download preservation and scope/PRJ/CRC guards passed.');

// Geometry uses the existing four-million-value display parser. The ordinary
// project parser's smaller complexity budget is unrelated to a saved map byte source.
const largeMap=JSON.stringify({type:'FeatureCollection',features:[{type:'Feature',properties:{geoid:'44001000100'},geometry:{type:'Polygon',coordinates:[Array.from({length:180000},()=>[0,0])]}}]});
const largeResult={schema_version:'bisect-tract-input-preparation-v1',status:'prepared',execution_performed:false,state:'RI',year:'2020',suggested_districts:2,graph_json:'{}',geometry_json:largeMap};
assert.equal(Buffer.from(preparedTractRunSources(largeResult).geometry.base64,'base64').toString(),largeMap);
console.log('Tract handoff: display geometry above the ordinary JSON complexity budget preserves exact bytes through the dedicated bounded geometry parser.');

if(process.argv.includes('--local-ri')){
 const prefix='data/2020/tiger/tracts/tl_2020_44_tract/tl_2020_44_tract',geoPath='data/2020/redistricting/rhode_island/rigeo2020.pl',popPath='data/2020/redistricting/rhode_island/ri000012020.pl';
 const tiger={kind:'tracts',year:2020,shp:null,dbf:null,shx:null};for(const part of ['shp','dbf','shx'])tiger[part]=source('tl_2020_44_tract.'+part,await fs.readFile(prefix+'.'+part));
 const files={tigerInputs:tiger,pl94Inputs:{year:2020,geography:source('rigeo2020.pl.gz',gzipSync(await fs.readFile(geoPath))),population:source('ri000012020.pl.gz',gzipSync(await fs.readFile(popPath)))},tractPreparationSettings:{state:'RI',projection:source('tl_2020_44_tract.prj',await fs.readFile(prefix+'.prj'))}};
 const {result,graph}=await check('ri2020',files,prefix+'.shp',geoPath,popPath);assert.equal(result.units,250);assert.equal(result.population,'1097379');
 const config=engine.execute(configTemplateRequest({...CONFIG_TEMPLATE_DEFAULTS,seed:'42',workers:1,years:['2020']})),runRequest={operation:'run-algorithm-config',input:{yaml_text:config.yaml_text,source_name:config.generated_filename,profile:'open',graph,chamber:'congressional',districts:2}};const run=engine.execute(runRequest);await fs.writeFile(root+'/ri2020-run-result.json',JSON.stringify(run));
 await fs.writeFile(root+'/run-request.json',JSON.stringify(runRequest));const hostRun=JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/run-request.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(hostRun.result.assignments,run.result.assignments);assert.equal(Object.keys(run.result.assignments).length,250);assert.equal(new Set(Object.values(run.result.assignments)).size,2);assert.equal(run.result.metrics.contiguous,true);assert.equal(run.result.metrics.within_requested_tolerance,true);
 await fs.writeFile(root+'/native-yaml.json',JSON.stringify({graph,yaml_text:config.yaml_text,options:{districts:2}}));const nativeRun=JSON.parse(execFileSync('target/debug/examples/config_seed_reference.exe',[root+'/native-yaml.json'],{encoding:'utf8',maxBuffer:32*1024*1024}));assert.deepEqual(nativeRun.assignments,run.result.assignments);
 const next=preparedTractRunSources(result),runFiles={...files,configSource:source(config.generated_filename,config.yaml_text),configSettings:{profile:'open'},configRunInputs:{graph:next.graph},configRunSettings:next.settings,planGeometry:next.geometry};const runProject=createProject({name:'Prepared RI Census run',files:runFiles,operation:'run-algorithm-config',constraints:[],result:run,lastOperation:'run-algorithm-config'});assert.deepEqual(parseProject(JSON.stringify(runProject)),runProject);await fs.writeFile(root+'/ri2020-run.bisect',JSON.stringify(runProject));
 console.log('Actual RI: 250 tracts, 1097379 population, '+result.bridges.length+' explicit derived bridges; maximum projected graph delta '+maxDelta+'; compressed source-preserving project and native-host/WASM/native YAML assignment-identical metis-core Run saved and reopened inertly. Connected, within tolerance, deviation '+run.result.metrics.max_deviation_percent+'%.');
}
