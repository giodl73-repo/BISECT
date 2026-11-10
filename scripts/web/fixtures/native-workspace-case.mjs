// Small synthetic RADJ fixture, regenerated without retained target outputs.
import {cffiEngineCase} from './cffi-engine-case.mjs';
export function nativeWorkspaceCase(){
 const u32=value=>{const b=Buffer.alloc(4);b.writeUInt32LE(value);return b;},i64=value=>{const b=Buffer.alloc(8);b.writeBigInt64LE(BigInt(value));return b;},f64=value=>{const b=Buffer.alloc(8);b.writeDoubleLE(value);return b;};
 const ids=['44001000100','44001000200','44001000300','44001000400'],neighbors=[[1],[0,2],[1,3],[2]],edges=[[0,1],[1,2],[2,3]];
 const binary=Buffer.concat([Buffer.from('RADJ'),u32(3),u32(4),u32(3),...ids.map(()=>i64(100)),...neighbors.flatMap(row=>[u32(row.length),...row.map(u32)]),u32(3),...edges.flatMap(([u,v])=>[u32(u),u32(v),f64(100)]),u32(1),...[300,200,200,300,10000,10000,10000,10000].map(f64)]);
 const template=cffiEngineCase(2,2);template.graph=null;template.options.districts=null;template.options.seed='18446744073709551615';
 const settings={label:'Workspace_RI',chamber:'congressional',created_at:'2026-10-23T00:00:00Z',inspect_only:true};
 const geometry={type:'FeatureCollection',state:'RI',year:'2020',features:ids.map((id,i)=>{const x=-71.5+i*.01,y=41.5;return {type:'Feature',id,properties:{geoid:id},geometry:{type:'Polygon',coordinates:[[[x,y],[x+.01,y],[x+.01,y+.01],[x,y+.01],[x,y]]]}};})};
 const stem='V3/data/2020/adjacency/ri_adjacency_2020';
 const files=new Map([[stem+'.pkl',Buffer.alloc(0)],[stem+'.adj.bin',binary],[stem+'_geoids.json',Buffer.from(JSON.stringify(Object.fromEntries(ids.map((id,i)=>[String(i),id]))))],[stem+'_centroids.json',Buffer.from(JSON.stringify({centroids:ids.map((_,i)=>[-71.495+i*.01,41.505])}))],[stem+'.request.json',Buffer.from(JSON.stringify(template))],[stem+'_display.geojson',Buffer.from(JSON.stringify(geometry))],['ignored.txt',Buffer.from('not a partition input')]]);
 return {files,graphInputs:{graph:null,template:null,districts:2,settings},stem,ids};
}
