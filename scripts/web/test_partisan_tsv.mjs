import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validatePartisanInput} from '../../web/lab/partisan-input.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';

const root=process.argv[2]||'dist/wasm-partisan-tsv';
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
await fs.mkdir('target/partisan-tsv-fixtures',{recursive:true});let sequence=0,exact=0,rejected=0,projects=0;
async function native(request,tool=true){const file=`target/partisan-tsv-fixtures/request-${sequence++}.json`;await fs.writeFile(file,JSON.stringify(request));return spawnSync(`target/release/examples/${tool?'execute_tool_request':'execute_request'}.exe`,[file],{encoding:'utf8',maxBuffer:32*1024*1024});}
const base={operation:'import-partisan-shares-tsv',state:'AL',year:'2020',source_label:'SYNTHETIC TSV — not election results'};
const request=(text,change={})=>({...base,source_base64:Buffer.from(text).toString('base64'),...change});
for(const header of ['', 'geoid\tdem_share\n'])for(const ending of ['\n','\r\n']){
 const text='\ufeff# synthetic shares\n'+header+'\n1001020100\t0.25\n# comment\n01001020200\t0.75\n';
 const req=request(text.replaceAll('\n',ending)),ref=await native(req);assert.equal(ref.status,0,ref.stderr);const input=engine.execute(req);assert.deepEqual(input,JSON.parse(ref.stdout));validatePartisanInput(input);assert.deepEqual(input.dem_shares,{'01001020100':.25,'01001020200':.75});exact++;
}
for(const req of [request(''),request('geoid\tdem_share\n'),request('wrong\theader\n01001020100\t.5'),
 ...['01001020100\tNaN','01001020100\t-0','01001020100\t-1','01001020100\tInfinity','01001020100\t1.01','01001020100\t','01001020100','01001020100\t.5\textra','abc\t.5','44001020100\t.5','01001020100\t.5\n1001020100\t.5'].map(row=>request('geoid\tdem_share\n'+row+'\n')),
 request('x',{state:'XX'}),request('x',{year:'2024'}),request('x',{source_label:'é'.repeat(101)}),request('x',{source_base64:'???'}),request('x',{source_base64:Buffer.from([255]).toString('base64')}),
 request('x'.repeat(8*1024*1024+1)),request('geoid\tdem_share\n'+Array.from({length:100001},(_,i)=>`${String(1000000000+i).padStart(11,'0')}\t.5`).join('\n'))
]){assert.throws(()=>engine.execute(req));assert.notEqual((await native(req)).status,0);rejected++;}
const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const fetcher=async ref=>{const b=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>b.buffer.slice(b.byteOffset,b.byteOffset+b.length)};};
let executions=0;
class EngineWorker{postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:message.id,result:engine.execute(message.request)}});}}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}terminate(){this.terminated=true;}}
const graphKeys=['RI:2020','IA:2020','NC:2020'].filter(key=>manifest.graphs[key]);if(!graphKeys.length)graphKeys.push(Object.keys(manifest.graphs)[0]);
for(const graphKey of graphKeys){
 const entry=manifest.graphs[graphKey];assert.ok(entry,'Missing prepared graph');const graph=JSON.parse(await fs.readFile(path.join(root,entry.graph_ref),'utf8')),code=graph.state;
 const tsv='geoid\tdem_share\n'+graph.geoids.map((id,i)=>`${id}\t${i%2?.8:.2}`).join('\n')+'\n';
 await fs.writeFile(`target/partisan-tsv-fixtures/${code}-synthetic.tsv`,tsv);
 const req=request(tsv,{state:code,year:graph.year}),input=engine.execute(req),ref=await native(req);assert.equal(ref.status,0,ref.stderr);assert.deepEqual(input,JSON.parse(ref.stdout));validatePartisanInput(input,graph);exact++;
 for(const search of ['single','multi']){
  const config={name:'SYNTHETIC TSV run',mode:'state',states:[code],year:graph.year,chamber:'congressional',districts:null,structure:'standard-bisect',weights:'partisan',search,seed:42,seeds:3,steps:20,percentile:0,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:30,timeout_seconds:60,partisans:{[code]:input},dem_threshold:.55,rep_threshold:.45};
  const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker()),run=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,5));
  const project=await createLabProject(lab,run.id);assert.equal(project.experiment.status,'completed',JSON.stringify(project.experiment.logs));const state=project.experiment.states[0],engineRequest={graph,options:state.metrics.requested_options,partisan:input};
  const result=engine.execute(engineRequest),reference=await native(engineRequest,false);assert.equal(reference.status,0,reference.stderr);assert.deepEqual(result,JSON.parse(reference.stdout));exact++;
  const before=executions,restorer=new WasmCatalog(manifest,fetcher,()=>{throw new Error('Open must not run the engine');}),opened=await restorer.restoreProject(project);assert.equal(executions,before);assert.deepEqual(opened.config.partisans[code],input);assert.equal((await restorer.api(`/api/runs/${opened.id}/${code}/map`)).features.length,graph.geoids.length);projects++;
  const bad=structuredClone(project);bad.experiment.config.partisans[code].dem_shares[graph.geoids[0]]=.4;await assert.rejects(restorer.restoreProject(bad));rejected++;
 }
}
console.log(`Partisan TSV: ${exact} exact native/WASM comparisons, ${rejected} malformed/tamper rejections, ${projects} ${graphKeys.join(',')} run/Save/Open/map cases; no engine on Open. All shares are synthetic.`);
