import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const root='target/config-area-wasm';await fs.mkdir(root,{recursive:true});
const adjacency=[[1,4],[0,2,5],[1,3,6],[2,7],[0,5],[1,4,6],[2,5,7],[3,6]];
const edges=[];for(let u=0;u<8;u++)for(const v of adjacency[u])if(u<v)edges.push([u,v,(u===1&&v===2)||(u===5&&v===6)?1:v-u===4?100:1000]);
const graph={schema_version:1,state:'IA',year:'2020',geoids:Array.from({length:8},(_,i)=>`19001${String(i).padStart(6,'0')}`),adjacency,population:Array(8).fill(100),edges,areas:Array(8).fill(1000000),exterior_perimeters:Array(8).fill(0),centroids:Array.from({length:8},(_,i)=>[-93+i%4*.01,42+Math.floor(i/4)*.01])};
const run=(exe,path)=>JSON.parse(execFileSync(exe,[path],{encoding:'utf8',maxBuffer:32*1024*1024,stdio:['ignore','pipe','pipe']}));
const evidence=[];
for(const search of ['single','multi']) for(const ratio of [null,1,1.25,1.9,2]) {
 const yaml_text=`name: area\nalgorithm:\n  structure: ratio-optimal-area\n  engine: metis-core\n  weights: geographic\n  search: ${search}\n  seeds: 64\n  seed: 42\n  balance_tolerance: 5\n${ratio==null?'':`  area_swing: ${ratio}\n`}`;
 const config=engine.execute({operation:'validate-algorithm-config',yaml_text,source_name:'area.yml',profile:'open'});
 const area_swing=config.resolution.split.area_swing;assert.equal(area_swing,ratio??1.1);
 const options={structure:'ratio-optimal-area',weights:'geographic',search,districts:2,seed:42,seeds:64,steps:20,percentile:0,alpha_county:0,balance_tolerance:5,area_swing,iterations:100};
 const result=engine.execute({graph,options});
 const path=`${root}/case-${evidence.length}.json`;await fs.writeFile(path,JSON.stringify({graph,yaml_text}));
 const native=run('target/debug/examples/config_area_reference.exe',path);
 assert.deepEqual(result.assignments,native.assignments);assert.equal(native.area_swing,area_swing);
 const areaLeft=graph.geoids.reduce((sum,id,i)=>sum+(result.assignments[id]<=native.left?graph.areas[i]:0),0);
 const total=graph.areas.reduce((a,b)=>a+b,0);
 assert.ok(areaLeft<=total*.5*area_swing+1e-6);assert.ok(total-areaLeft<=total*.5*area_swing+1e-6);
 assert.equal(result.metrics.root_split.area_left_m2,areaLeft);assert.equal(result.metrics.root_split.weighted_boundary,native.cut);
 evidence.push({search,declared_ratio:ratio,effective_ratio:area_swing,area_left:areaLeft,total_area:total,assignments:result.assignments});
}
for(const ratio of [.1,.25,.99,2.01]) assert.throws(()=>engine.execute({operation:'validate-algorithm-config',yaml_text:`name: bad\nalgorithm:\n  structure: ratio-optimal-area\n  area_swing: ${ratio}\n`,source_name:'bad.yml',profile:'open'}),/multiplier/);
await fs.writeFile(root+'/evidence.json',JSON.stringify(evidence,null,2));
console.log(`Area YAML execution: ${evidence.length} actual native YAML/AreaSection and WASM assignment matches, independent area bounds, ratio endpoints, default and four margin/range rejections passed.`);
