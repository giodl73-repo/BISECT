import fs from 'node:fs/promises';
import path from 'node:path';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const root=process.argv[2]||'dist/wasm-national';
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const digest=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const wasm=await fs.readFile(path.join(root,'bisect_wasm.wasm'));assert.equal(digest(wasm),manifest.wasm_sha256);
const engine=await instantiateEngine(wasm),states=[];
for(const state of manifest.catalog.states){
 const key=state.code+':2020',entry=manifest.graphs[key];assert.ok(entry,'Missing '+key);
 const bytes=await fs.readFile(path.join(root,entry.graph_ref));assert.equal(digest(bytes),manifest.assets[entry.graph_ref]);
 const native=await fs.readFile(`runs/lab/cache/${state.code.toLowerCase()}_adjacency_2020.adj.bin`);assert.equal(digest(native),entry.native_graph_sha256);
 const graph=JSON.parse(bytes);assert.ok(graph.geoids.every(id=>id.startsWith(state.fips)));
 const result=engine.execute({graph,options:{structure:'standard-bisect',weights:'geographic',search:'single',districts:1,seed:42,seeds:1,steps:1,percentile:0,alpha_county:0,balance_tolerance:5,area_swing:1.1,iterations:10}});
 assert.equal(result.metrics.contiguous,true,key);assert.equal(result.metrics.population,graph.population.reduce((sum,value)=>sum+value,0));
 assert.equal(result.metrics.units,graph.geoids.length);assert.equal(Object.keys(result.assignments).length,graph.geoids.length);
 const geometryRef=manifest.geometries[key],geometryBytes=await fs.readFile(path.join(root,geometryRef));assert.equal(digest(geometryBytes),manifest.assets[geometryRef]);
 const geometry=JSON.parse(geometryBytes),ids=new Set(geometry.features.map(f=>f.properties.geoid));
 assert.equal(geometry.state,state.code);assert.equal(geometry.year,'2020');assert.equal(ids.size,geometry.features.length);assert.equal(ids.size,graph.geoids.length);
 assert.ok(graph.geoids.every(id=>ids.has(id)));assert.ok(engine.memoryBytes()<=128*1024*1024);
 states.push({state:state.code,units:graph.geoids.length,population:result.metrics.population,memory_bytes:engine.memoryBytes()});
}
assert.equal(states.length,50);assert.equal(new Set(states.map(s=>s.state)).size,50);
const report={states:states.length,units:states.reduce((n,s)=>n+s.units,0),population:states.reduce((n,s)=>n+s.population,0),maximum_wasm_memory_bytes:Math.max(...states.map(s=>s.memory_bytes)),detail:states,scope:'Input integrity, schema, native binary hash, graph connectivity and geometry joins; one-district validation does not prove allocated multi-district algorithm validity.'};
await fs.writeFile('target/wasm-national-input-validation.json',JSON.stringify(report,null,2));
console.log(JSON.stringify({...report,detail:undefined}));
