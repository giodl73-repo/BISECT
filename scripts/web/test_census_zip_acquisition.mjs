import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {CENSUS_ARCHIVE_ENDPOINTS} from '../../web/lab/census-endpoints.js';
import {acquireFetchSource,fetchAcquisitionChoices,publicFetchUrl} from '../../web/lab/fetch-acquisition.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {tigerRequest} from '../../web/lab/tiger-files.js';
import {pl94Request} from '../../web/lab/pl94-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {fletchStageFromAcquisition,fletchStageRequest,fletchStagedCacheZip} from '../../web/lab/fletch-stage-files.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8')),root='target/census-zip-acquisition';await fs.mkdir(root,{recursive:true});
let checked=0;
for(const year of ['2000','2010','2020']){
 const plan=engine.execute({operation:'plan-fetch-sources',input:{manifest_json:null,states:Object.keys(manifest.states),year,data_types:['tiger','redistricting'],selected_paths:[]}});
 for(const {item} of fetchAcquisitionChoices(plan)){assert.equal(publicFetchUrl(item),item.url);assert.equal(CENSUS_ARCHIVE_ENDPOINTS[`${item.state_code}:${year}:${item.kind}`],item.url);checked++;}
 assert.equal(fetchAcquisitionChoices(plan).length,plan.items.filter(i=>i.url?.startsWith('https://www2.census.gov/')).length);
}
assert.equal(checked,Object.keys(CENSUS_ARCHIVE_ENDPOINTS).length);
const selection={manifest_json:null,states:['RI'],year:'2020',data_types:['tiger','redistricting'],selected_paths:[]},plan=engine.execute({operation:'plan-fetch-sources',input:selection});
const archiveReader=async(file,settings,{signal})=>{signal?.throwIfAborted();return extractCensusZip(new Uint8Array(await file.arrayBuffer()),settings);};
for(const kind of ['tiger','pl94171']){
 const choice=fetchAcquisitionChoices(plan).find(v=>v.item.kind===kind),bytes=await fs.readFile('target/census-archive-tests/'+(kind==='tiger'?'deflated':'pl2020')+'.zip');let calls=0;
 const acquired=await acquireFetchSource(plan,choice.index,{archiveReader,fetcher:async(url,options)=>{calls++;assert.equal(url,choice.item.url);assert.equal(options.credentials,'omit');assert.equal(options.redirect,'error');return new Response(bytes);}});
 assert.equal(calls,1);assert.equal(acquired.receipt.source_sha256,createHash('sha256').update(bytes).digest('hex'));assert.equal(acquired.inputs.archive.execution_performed,false);assert.equal(acquired.inputs.state,'RI');assert.equal(acquired.raw_archive_source.source.base64,bytes.toString('base64'));assert.equal(acquired.raw_archive_source.url,choice.item.url);
 const sources=acquired.inputs.archive.sources,inputs=kind==='tiger'?{kind:'tracts',year:2020,shp:sources.shp,dbf:sources.dbf,shx:sources.shx||null}:{year:2020,geography:sources.geography,population:sources.population};
 const prepared=engine.execute(kind==='tiger'?tigerRequest(inputs):pl94Request(inputs));assert.equal(kind==='tiger'?prepared.record_count:prepared.block_count,2);
 const project=createProject({name:'Acquired Census '+kind,files:{fetchArchiveSource:acquired.raw_archive_source,fetchAcquisitionReceipt:acquired.receipt,[kind==='tiger'?'tigerInputs':'pl94Inputs']:inputs,...(kind==='tiger'?{tractPreparationSettings:{state:'RI',projection:sources.prj||null}}:{})},operation:kind==='tiger'?'prepare-tiger-source':'prepare-pl94-source',constraints:[],result:null,lastOperation:null});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+kind+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+kind+'-acquired.json',JSON.stringify(acquired));
 for(const fetcher of [async()=>new Response('small',{headers:{'content-length':String(64*1024*1024+1)}}),async()=>new Response('not ZIP')])await assert.rejects(()=>acquireFetchSource(plan,choice.index,{archiveReader,fetcher}));
 const mismatch=async(...args)=>({...await archiveReader(...args),archive_sha256:'0'.repeat(64)});await assert.rejects(()=>acquireFetchSource(plan,choice.index,{archiveReader:mismatch,fetcher:async()=>new Response(bytes)}),/identity/);
 assert.equal((await acquireFetchSource(plan,choice.index,{archiveReader,fetcher:async()=>new Response('missing',{status:404})})).inputs,null);
 for(const url of [choice.item.url+'?x=1',choice.item.url.replace('www2.census.gov','attacker.invalid')])assert.throws(()=>publicFetchUrl({...choice.item,url}));
}
await fs.writeFile(root+'/plan-result.json',JSON.stringify(plan));await fs.writeFile(root+'/plan.bisect',JSON.stringify(createProject({name:'Acquire RI Census ZIPs',files:{fetchPlanInputs:{source:{name:'selection.json',base64:Buffer.from(JSON.stringify(selection)).toString('base64')}}},operation:'plan-fetch-sources',constraints:[],result:plan,lastOperation:'plan-fetch-sources'})));
console.log(`Census ZIP acquisition: ${checked} endpoint identities checked against compiled native source plans, streamed fixture acquisition, archive CRC/hash handoff, actual WASM reader preparation, inert source/receipt project persistence, size/corruption/identity/404/URL guards passed. Public network availability is not asserted.`);
if(process.argv.includes('--local-ri')){
 execFileSync('python',['-c',String.raw`
import zipfile,pathlib
r=pathlib.Path('target/census-zip-acquisition')
with zipfile.ZipFile(r/'ri-tiger.zip','w',compression=8) as z:
 for ext in ['shp','dbf','shx','prj']:
  p=pathlib.Path('data/2020/tiger/tracts/tl_2020_44_tract/tl_2020_44_tract.'+ext);z.write(p,p.name)
with zipfile.ZipFile(r/'ri-pl94.zip','w',compression=8) as z:
 for name in ['rigeo2020.pl','ri000012020.pl']:
  p=pathlib.Path('data/2020/redistricting/rhode_island')/name;z.write(p,p.name)
`]);
 const acquired={};for(const kind of ['tiger','pl94171']){const choice=fetchAcquisitionChoices(plan).find(v=>v.item.kind===kind),bytes=await fs.readFile(root+'/'+(kind==='tiger'?'ri-tiger':'ri-pl94')+'.zip');acquired[kind]=await acquireFetchSource(plan,choice.index,{archiveReader,fetcher:async()=>new Response(bytes)});}
 const tiger=acquired.tiger.inputs.archive.sources,pl94=acquired.pl94171.inputs.archive.sources,files={tigerInputs:{kind:'tracts',year:2020,shp:tiger.shp,dbf:tiger.dbf,shx:tiger.shx},pl94Inputs:{year:2020,geography:pl94.geography,population:pl94.population},tractPreparationSettings:{state:'RI',projection:tiger.prj},fetchArchiveSource:acquired.pl94171.raw_archive_source,fetchAcquisitionReceipt:acquired.pl94171.receipt};
 const result=engine.execute({operation:'prepare-tract-engine-input',input:{state:'RI',tiger:files.tigerInputs,pl94:files.pl94Inputs,projection:tiger.prj}});assert.equal(result.units,250);assert.equal(result.population,'1097379');const expected=JSON.parse(await fs.readFile('target/tract-preparation-wasm/ri2020-result.json','utf8'));assert.equal(result.graph_json,expected.graph_json);assert.equal(result.geometry_json,expected.geometry_json);
 const saved=createProject({name:'RI sources acquired from ZIP fixtures',files,operation:'prepare-tract-engine-input',constraints:[],result,lastOperation:'prepare-tract-engine-input'});assert.deepEqual(parseProject(JSON.stringify(saved)),saved);await fs.writeFile(root+'/ri2020.bisect',JSON.stringify(saved));
 const inputs=fletchStageFromAcquisition(files,'123'),staged=engine.execute(fletchStageRequest(inputs,files));assert.equal(staged.source_sha256,files.fetchAcquisitionReceipt.source_sha256);const cacheFiles={...files,fletchStageInputs:inputs},archive=createWorkbenchArchive({name:'RI original archive cache',files:cacheFiles,operation:'stage-fletch-cache-source',constraints:[],result:staged,lastOperation:'stage-fletch-cache-source'});assert.deepEqual(parseWorkbenchArchive(archive).files,cacheFiles);assert.equal(archive.split(files.fetchArchiveSource.source.base64).length-1,1);await fs.writeFile(root+'/ri2020-cache.bisect-workbench',archive);await fs.writeFile(root+'/ri2020-cache.zip',await fletchStagedCacheZip(staged,inputs,files));
 console.log('Actual RI source archives: streamed fixture ZIP acquisition, lossless selected/raw sources, identical 250-tract graph/display map, source-preserving project roundtrip, receipt-bound raw PL94 cache staging and original-object ZIP/single-copy archive export passed. No public HTTP success claim.');
}
