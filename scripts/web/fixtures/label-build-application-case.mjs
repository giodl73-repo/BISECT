import fs from 'node:fs/promises';import path from 'node:path';import assert from 'node:assert/strict';
import {nativeWorkspaceCase} from './native-workspace-case.mjs';
export async function labelBuildApplicationCase(engine,{label='build_fixture',years=['2020'],states=['RI'],force=false,registry=null,failed=null,seed='42'}={}){
 const source=(name,bytes)=>({name,base64:Buffer.from(bytes).toString('base64')}),fixture=nativeWorkspaceCase(),nativeFiles=new Map(),jobs=[],manifest=JSON.parse(await fs.readFile('data/manifest.json','utf8'));
 for(const year of years)for(const state of states){const base=fixture.stem,stem=`V3/data/${year}/adjacency/${state.toLowerCase()}_adjacency_${year}`,fips=manifest.states[state].fips,binary=Buffer.from(fixture.files.get(base+'.adj.bin'));if(failed===state)for(let i=0;i<4;i++)binary.writeBigInt64LE(BigInt(i===0?100000:1),16+i*8);const ids=Object.fromEntries(fixture.ids.map((id,i)=>[String(i),fips+id.slice(2)])),mapping=Buffer.from(JSON.stringify(ids)),centroids=fixture.files.get(base+'_centroids.json');for(const [suffix,bytes]of [['.pkl',Buffer.alloc(0)],['.adj.bin',binary],['_geoids.json',mapping],['_centroids.json',centroids]])nativeFiles.set(stem+suffix,bytes);
  const prepared=await engine.execute({operation:'prepare-native-engine-input',input:{state,year,sources:{adjacency:source(path.basename(stem+'.adj.bin'),binary),geoids:source(path.basename(stem+'_geoids.json'),mapping),centroids:source(path.basename(stem+'_centroids.json'),centroids),geometry:null}}});jobs.push({graph:JSON.parse(prepared.graph_json),districts:manifest.states[state].districts[year],chamber:'congressional',demographics:null});
 }
 const yaml=`name: ${label}
algorithm:
  structure: standard-bisect
  weights: geographic
  engine: metis-core
  seed: ${seed}
  search: single
  balance_tolerance: 25
years: [${years.map(y=>JSON.stringify(y)).join(', ')}]
workers: 1
`;
 return {label,years,states,manifest,nativeFiles,yaml,jobs,inputs:{config:source(label+'.yml',yaml),jobs:source('jobs.json',JSON.stringify(jobs)),registry:registry===null?null:source('.bisect',registry),settings:{label,profile:'open',years,force,created_at:'2026-10-23T00:00:00Z'}}};
}
export async function writeNativeBuildFixture(fixture,root){assert(path.isAbsolute(root)&&root.startsWith(path.resolve('target')+path.sep));const manifest={...fixture.manifest,local_outputs_dir:path.join(root,'inputs'),local_data_dir:path.join(root,'data')};await fs.mkdir(manifest.local_data_dir,{recursive:true});for(const [p,b]of fixture.nativeFiles){const target=path.resolve(manifest.local_outputs_dir,p);assert(target.startsWith(root+path.sep));await fs.mkdir(path.dirname(target),{recursive:true});await fs.writeFile(target,b);}await fs.mkdir(path.join(root,'configs'),{recursive:true});await fs.writeFile(path.join(root,'configs',fixture.label+'.yml'),fixture.yaml);const file=path.join(root,'manifest.json');await fs.writeFile(file,JSON.stringify(manifest));return file;}
