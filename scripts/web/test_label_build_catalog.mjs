import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {readConfigCatalog,loadConfigCatalogJobs} from '../../web/lab/config-catalog.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {labelBuildRequest} from '../../web/lab/label-build-files.js';
const root=process.argv[2]||'dist/wasm-lab-build';
const fetcher=async path=>new Response(await fs.readFile(root+'/'+path.replace(/^\.\.\//,'')));
const catalog=await readConfigCatalog(fetcher);
const keys=catalog.entries.slice(0,2).map(e=>e.key);let cases=0;
for(const chamber of ['congressional','house','senate']){
 const next=await loadConfigCatalogJobs(catalog,keys,chamber,fetcher);
 const jobs=JSON.parse(Buffer.from(next.source.base64,'base64').toString('utf8'));
 assert.equal(jobs.length,keys.length);assert.deepEqual(next.years,[...new Set(next.selected.map(e=>e.year))]);
 for(let n=0;n<jobs.length;n++){const entry=catalog.entries.find(e=>e.key===keys[n]);const original=JSON.parse(await fs.readFile(root+'/'+catalog.manifest.graphs[entry.key].graph_ref,'utf8'));assert.deepEqual(jobs[n].graph,original);assert.equal(jobs[n].districts,entry.districts[chamber]);assert.equal(jobs[n].chamber,chamber);assert.equal(jobs[n].demographics,null);}
 const inputs={config:{name:'catalog.yml',base64:Buffer.from('name: catalog\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n').toString('base64')},jobs:next.source,registry:null,settings:{label:'catalog',profile:'open',years:next.years,force:false,created_at:'2026-10-07T00:00:00Z'}};
 const project=createProject({name:'Catalog batch',files:{labelBuildInputs:inputs},operation:'run-label-build',constraints:[],result:null,lastOperation:null});
 assert.deepEqual(labelBuildRequest(parseProject(JSON.stringify(project)).files.labelBuildInputs).input.jobs,jobs);cases++;
}
let rejects=0;
for(const [selection,chamber]of [[[], 'house'],[[keys[0],keys[0]],'house'],[['ZZ:1999'],'house'],[keys,'invalid']]){let fetched=false;await assert.rejects(()=>loadConfigCatalogJobs(catalog,selection,chamber,()=>{fetched=true;}));assert.equal(fetched,false);rejects++;}
await assert.rejects(()=>loadConfigCatalogJobs(catalog,[keys[0]],'congressional',async p=>new Response((await(await fetcher(p)).text())+' ')),/SHA-256/);rejects++;
const bad=structuredClone(catalog);bad.entries.find(e=>e.key===keys[0]).districts.house=null;let fetched=false;await assert.rejects(()=>loadConfigCatalogJobs(bad,[keys[0]],'house',()=>{fetched=true;}));assert.equal(fetched,false);rejects++;
await fs.mkdir('target/label-build-catalog',{recursive:true});await fs.writeFile('target/label-build-catalog/evidence.json',JSON.stringify({cases,rejects,keys,scope:'Published graph preparation and chamber allocations; no engine execution or full national capacity claim.'},null,2));
console.log(`Build catalog: ${cases} real-input chamber batches with exact graphs/project roundtrip and ${rejects} rejects passed.`);
