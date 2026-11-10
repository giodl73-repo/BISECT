import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {readStateCensusArchives} from '../../web/lab/state-source-files.js';
import {extractCensusZip} from '../../web/lab/census-archive.js';
import {tractPreparationRequest,preparedTractRunSources} from '../../web/lab/tract-input-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {createWorkbenchArchive,parseWorkbenchArchive} from '../../web/lab/workbench-archive.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const tiger=new File([await fs.readFile('target/census-archive-tests/deflated.zip')],'tracts.zip'),pl94=new File([await fs.readFile('target/census-archive-tests/pl2020.zip')],'pl94.zip');
const archiveReader=async(file,settings,{signal})=>{signal.throwIfAborted();return extractCensusZip(new Uint8Array(await file.arrayBuffer()),settings);};
const prepared=await readStateCensusArchives(tiger,pl94,{state:'RI',year:2020,archiveReader}),result=engine.execute(tractPreparationRequest(prepared.files));
assert.equal(result.population,'18');assert.equal(prepared.archives.length,2);assert.equal(result.state,'RI');
const reference=JSON.parse(await fs.readFile('target/tract-preparation-wasm/2020-result.json','utf8'));assert.equal(result.graph_json,reference.graph_json);assert.equal(result.geometry_json,reference.geometry_json);
const project=createProject({name:'Paired Census sources',files:prepared.files,operation:'prepare-tract-engine-input',constraints:[],result,lastOperation:'prepare-tract-engine-input'});assert.deepEqual(parseProject(JSON.stringify(project)),project);
assert.deepEqual(parseWorkbenchArchive(createWorkbenchArchive(project)).files,prepared.files);preparedTractRunSources(result);
await fs.mkdir('target/state-source-files',{recursive:true});await fs.writeFile('target/state-source-files/paired.bisect',JSON.stringify(project));
for(const year of [2000,2010]){const population=new File([await fs.readFile(`target/census-archive-tests/pl${year}.zip`)],'population.zip');const selected=await readStateCensusArchives(tiger,population,{state:'RI',year,archiveReader}),output=engine.execute(tractPreparationRequest(selected.files)),expected=JSON.parse(await fs.readFile(`target/tract-preparation-wasm/${year}-result.json`,'utf8'));assert.equal(output.graph_json,expected.graph_json);assert.equal(output.geometry_json,expected.geometry_json);}
let reads=0;for(const [state,year]of [['XX',2020],['RI',2021]])await assert.rejects(()=>readStateCensusArchives(tiger,pl94,{state,year,archiveReader:async()=>{reads++;}}));assert.equal(reads,0);
await assert.rejects(()=>readStateCensusArchives(tiger,null,{state:'RI',year:2020,archiveReader:async()=>{reads++;}}));assert.equal(reads,0);
const cancelled=new AbortController();cancelled.abort();await assert.rejects(()=>readStateCensusArchives(tiger,pl94,{state:'RI',year:2020,signal:cancelled.signal,archiveReader:async()=>{reads++;}}));assert.equal(reads,0);
await assert.rejects(()=>readStateCensusArchives(new File([Buffer.alloc(22)],'bad.zip'),pl94,{state:'RI',year:2020,archiveReader}));
await assert.rejects(()=>readStateCensusArchives(tiger,pl94,{state:'RI',year:2020,archiveReader:async(f,s,o)=>{const a=await archiveReader(f,s,o);if(s.kind==='tiger')delete a.sources.prj;return a;}}),/PRJ/);
const wrong=await readStateCensusArchives(tiger,pl94,{state:'IA',year:2020,archiveReader});assert.throws(()=>engine.execute(tractPreparationRequest(wrong.files)),/selected state/);
let companionCancelled=false;await assert.rejects(()=>readStateCensusArchives(tiger,pl94,{state:'RI',year:2020,archiveReader:async(f,s,{signal})=>s.kind==='tiger'?Promise.reject(new Error('bad geometry')):new Promise((resolve,reject)=>signal.addEventListener('abort',()=>{companionCancelled=true;reject(new Error('companion cancelled'));},{once:true}))}),/bad geometry/);assert.equal(companionCancelled,true);
console.log('Paired state sources: real ZIP extraction, exact existing native-oracle graph/display output, state guard, project/archive retention and configuration handoff; invalid selections, missing PRJ, corrupt ZIP and companion cancellation passed.');

if(process.argv.includes('--local-ri')){
 const geometry=new File([await fs.readFile('target/census-zip-acquisition/ri-tiger.zip')],'ri-tiger.zip'),population=new File([await fs.readFile('target/census-zip-acquisition/ri-pl94.zip')],'ri-pl94.zip');
 const selected=await readStateCensusArchives(geometry,population,{state:'RI',year:2020,archiveReader}),output=engine.execute(tractPreparationRequest(selected.files)),expected=JSON.parse(await fs.readFile('target/tract-preparation-wasm/ri2020-result.json','utf8'));
 assert.equal(output.units,250);assert.equal(output.population,'1097379');assert.equal(output.graph_json,expected.graph_json);assert.equal(output.geometry_json,expected.geometry_json);
 const archive=createWorkbenchArchive({name:'Actual RI paired Census sources',files:selected.files,operation:'prepare-tract-engine-input',constraints:[],result:output,lastOperation:'prepare-tract-engine-input'});assert.deepEqual(parseWorkbenchArchive(archive).files,selected.files);await fs.writeFile('target/state-source-files/ri2020.bisect-workbench',archive);
 console.log('Actual local RI paired ZIPs: 250 tracts and 1097379 population, exact native-oracle graph/map and source-preserving archive roundtrip passed.');
}
