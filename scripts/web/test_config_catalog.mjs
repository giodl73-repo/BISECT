import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {readConfigCatalog,loadConfigCatalogInputs} from '../../web/lab/config-catalog.js';
import {configRunRequest,configuredRunMapDocument,configRunSourceText} from '../../web/lab/config-run.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {planMapModel} from '../../web/lab/plan-map.js';
import {createProject,parseProject,parseGeometryJson} from '../../web/lab/project.js';
const root=process.argv[2]||'dist/wasm-national';
const fetcher=async path=>new Response(await fs.readFile(root+'/'+path.replace(/^\.\.\//,'')));
const catalog=await readConfigCatalog(fetcher);const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const source={name:'catalog-run.yml',base64:Buffer.from('name: published\nalgorithm:\n  structure: standard-bisect\n  engine: metis-core\n  search: single\n  seed: 42\n').toString('base64')};
let cases=0;const evidence=[];
for(const entry of catalog.entries){
 const code=entry.state,next=await loadConfigCatalogInputs(catalog,entry.key,'congressional',fetcher);
 const graph=JSON.parse(configRunSourceText(next.graph));assert.equal(next.settings.districts,entry.districts.congressional);
 const request=configRunRequest(source,{profile:'open'},{graph:next.graph},next.settings),result=engine.execute(request);
 assert.equal(result.result.metrics.contiguous,true);assert.equal(planMapModel(configuredRunMapDocument(result),parseGeometryJson(Buffer.from(next.geometry.base64,'base64').toString('utf8'))).units,graph.geoids.length);
 const project=createProject({name:code,files:{configSource:source,configRunInputs:{graph:next.graph},configRunSettings:next.settings,planGeometry:next.geometry},operation:'run-algorithm-config',constraints:[],result,lastOperation:'run-algorithm-config'});const serialized=JSON.stringify(project);assert.deepEqual(parseProject(serialized),project);cases++;evidence.push({scope:entry.key,units:graph.geoids.length,districts:next.settings.districts,connected:result.result.metrics.contiguous,within_tolerance:result.result.metrics.within_requested_tolerance,max_deviation_percent:result.result.metrics.max_deviation_percent,project_bytes:Buffer.byteLength(serialized)});console.log('Verified '+entry.key+' · '+graph.geoids.length+' units');
}
const probe=catalog.entries[0].key;const original=await fs.readFile(root+'/catalog.json','utf8');
for(const change of [m=>m.graphs[probe].graph_ref='../private.json',m=>m.assets[m.graphs[probe].graph_ref]='0'.repeat(64),m=>m.geometries[probe]='https://example.org/map.json']){const m=JSON.parse(original);change(m);await assert.rejects(readConfigCatalog(async()=>new Response(JSON.stringify(m))));}
await assert.rejects(loadConfigCatalogInputs(catalog,'ZZ:1999','congressional',fetcher));
await assert.rejects(loadConfigCatalogInputs(catalog,probe,'oops',fetcher));
await assert.rejects(loadConfigCatalogInputs(catalog,probe,'congressional',async path=>{const response=await fetcher(path);return new Response((await response.text())+' ');}),/SHA-256/);
await assert.rejects(readConfigCatalog(async()=>new Response('{}',{headers:{'content-length':String(1024*1024+1)}})),/size limit/);
await assert.rejects(readConfigCatalog(async()=>new Response('{"schema_version":1,"schema_version":1}')),/Duplicate/);
await fs.mkdir('target/config-catalog-wasm',{recursive:true});await fs.writeFile('target/config-catalog-wasm/evidence.json',JSON.stringify({cases,scope:'Published configured single-seed congressional runs; exact hashed sources, complete map join and project roundtrip. No general algorithm or legal certification.',states:evidence},null,2));
console.log(`Published configured inputs: ${cases} real-state WASM runs/maps/project roundtrips; hashes, paths, sizes, duplicate JSON and unavailable selections rejected.`);
