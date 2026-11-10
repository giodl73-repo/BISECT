// Follow-up recipe; the original apportion-regions declaration is unchanged.
import assert from 'node:assert/strict';
import {realDataMatrixRequest} from './real-data-matrix-case.mjs';
export async function fullChamberMatrixRequest(engine,row){
 const request=await realDataMatrixRequest(engine,row);assert.equal(row.structure,'standard-bisect');assert.equal(row.balance_tolerance_percent,0.5);assert(request.input.run.yaml_text.includes('structure: apportion-regions'));
 request.input.run.yaml_text=request.input.run.yaml_text.replace('structure: apportion-regions','structure: standard-bisect');return request;
}
export function nativeMatrixFiles(graph){
 const u32=v=>{const b=Buffer.alloc(4);b.writeUInt32LE(v);return b;},i64=v=>{const b=Buffer.alloc(8);b.writeBigInt64LE(BigInt(v));return b;},f64=v=>{const b=Buffer.alloc(8);b.writeDoubleLE(v);return b;};
 const n=graph.geoids.length;assert.equal(graph.population.length,n);assert.equal(graph.adjacency.length,n);assert.equal(graph.areas.length,n);assert.equal(graph.exterior_perimeters.length,n);assert.equal(graph.centroids.length,n);
 const binary=Buffer.concat([Buffer.from('RADJ'),u32(3),u32(n),u32(graph.edges.length),...graph.population.map(i64),...graph.adjacency.flatMap(a=>[u32(a.length),...a.map(u32)]),u32(graph.edges.length),...graph.edges.flatMap(([u,v,w])=>[u32(u),u32(v),f64(w)]),u32(1),...graph.exterior_perimeters.map(f64),...graph.areas.map(f64)]),stem=`V3/data/${graph.year}/adjacency/${graph.state.toLowerCase()}_adjacency_${graph.year}`;
 return new Map([[stem+'.pkl',Buffer.alloc(0)],[stem+'.adj.bin',binary],[stem+'_geoids.json',Buffer.from(JSON.stringify(Object.fromEntries(graph.geoids.map((id,i)=>[String(i),id]))))],[stem+'_centroids.json',Buffer.from(JSON.stringify({centroids:graph.centroids}))]]);
}
