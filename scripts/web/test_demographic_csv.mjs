import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {validateDemographicInput,demographicIdentity} from '../../web/lab/demographic-input.js';
import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject,parseLabProject} from '../../web/lab/laboratory-project.js';

const root=process.argv[2]||'dist/wasm-demographic-csv';
const engine=await instantiateEngine(await fs.readFile(path.join(root,'bisect_wasm.wasm')));
const fixtures='target/demographic-csv-fixtures';await fs.mkdir(fixtures,{recursive:true});let sequence=0;
async function native(request,tool=true){const file=path.join(fixtures,`request-${sequence++}.json`);await fs.writeFile(file,JSON.stringify(request));return spawnSync(`target/release/examples/${tool?'execute_tool_request':'execute_request'}.exe`,[file],{encoding:'utf8',maxBuffer:32*1024*1024});}
const base={operation:'import-demographic-csv',state:'AL',year:'2020',basis:'total-population',source_label:'SYNTHETIC CSV fixture'};
const request=(text,change={})=>({...base,source_base64:Buffer.from(text).toString('base64'),...change});let exact=0,rejections=0;
for(const [column,basis]of [['total_pop','total-population'],['total_vap','voting-age-population'],['vap','voting-age-population'],['cvap','citizen-voting-age-population'],['total_vap','citizen-voting-age-population']])for(const ending of ['\n','\r\n']){
  const white=column==='total_pop',text=`\ufeffGEOID,${column},${white?'white_non_hispanic':'minority_vap'},note${ending}1001020100,800,500,"quoted, note"${ending}01001020200,0,0,"multiline${ending}text"${ending}`;
  const req=request(text,{basis}),reference=await native(req);assert.equal(reference.status,0,reference.stderr);const imported=engine.execute(req);assert.deepEqual(imported,JSON.parse(reference.stdout));validateDemographicInput(imported);
  assert.equal(imported.counts['01001020100'].minority,white?300:500);assert.equal(imported.minority_fractions['01001020200'],0);
  const fractions={...imported,schema_version:'bisect-demographic-fractions-v1'};delete fractions.counts;assert.equal(await demographicIdentity(imported),await demographicIdentity(fractions));exact++;
}
for(const req of [
 request(''),request('wrong,headers\n1,2'),request('GEOID,total_pop,white_non_hispanic\n'),
 ...['01001020100,10,11','01001020100,NaN,1','01001020100,10,-0','01001020100,nope,1','01001020100,10,','01001020100,10,1,extra','44001020100,10,1','abc,10,1','01001020100,10,1\n1001020100,10,1'].map(row=>request(`GEOID,total_pop,white_non_hispanic\n${row}\n`)),
 request('GEOID,total_pop,total_pop,white_non_hispanic\n01001020100,10,10,1\n'),
 request('GEOID,total_vap,vap,minority_vap\n01001020100,10,10,1\n',{basis:'voting-age-population'}),
 request('GEOID,cvap,minority_vap\n01001020100,10,1\n',{basis:'voting-age-population'}),
 request('x',{source_base64:'not base64'}),request('x',{source_base64:Buffer.from([0xff]).toString('base64')}),request('x',{source_label:'é'.repeat(101)}),request('x',{state:'XX'}),request('x',{year:'2020x'}),request('x',{basis:'unknown'}),
]){assert.throws(()=>engine.execute(req));assert.notEqual((await native(req)).status,0);rejections++;}

const manifest=JSON.parse(await fs.readFile(path.join(root,'catalog.json'),'utf8'));
const fetcher=async ref=>{const bytes=await fs.readFile(path.join(root,ref));return{ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};
const graphKey=manifest.graphs['RI:2020']?'RI:2020':Object.keys(manifest.graphs).find(key=>key.endsWith(':2020'))||Object.keys(manifest.graphs)[0];
const graph=JSON.parse(await fs.readFile(path.join(root,manifest.graphs[graphKey].graph_ref),'utf8')),code=graph.state;let executions=0;
class EngineWorker{postMessage(message){setImmediate(()=>{if(this.terminated)return;try{if(message.type==='initialize')this.onmessage({data:{type:'ready'}});else{executions++;this.onmessage({data:{type:'result',id:message.id,result:engine.execute(message.request)}});}}catch(error){this.onmessage({data:{type:'error',id:message.id,error:error.message}});}});}terminate(){this.terminated=true;}}
let projectCases=0;
for(const basis of ['total-population','voting-age-population','citizen-voting-age-population']){
 const header=basis==='total-population'?'GEOID,total_pop,white_non_hispanic':`GEOID,${basis==='voting-age-population'?'total_vap':'cvap'},minority_vap`;
 const text=header+'\n'+graph.geoids.map((id,i)=>`${id},1000,${i%2?100:900}`).join('\n')+'\n';
 const req=request(text,{state:code,year:graph.year,basis,source_label:`SYNTHETIC ${basis} CSV — not Census data`}),input=engine.execute(req),reference=await native(req);assert.equal(reference.status,0,reference.stderr);assert.deepEqual(input,JSON.parse(reference.stdout));exact++;
 await fs.writeFile(path.join(fixtures,`${code}-${basis}-synthetic.csv`),text);
 for(const structure of basis==='total-population'?['standard-bisect','ratio-optimal-vra']:['standard-bisect']){
  const config={name:`CSV ${basis} test`,mode:'state',states:[code],year:graph.year,chamber:'congressional',districts:null,structure,weights:'geographic',search:structure==='standard-bisect'?'vra-recom':'single',seed:42,seeds:4,steps:2,percentile:0,alpha_county:0,balance_tolerance:0.5,area_swing:1.1,iterations:100,timeout_seconds:60,demographics:{[code]:input},...(structure==='standard-bisect'?{vra_threshold:0.5}:{w_vra:0.4})};
  const lab=new WasmCatalog(manifest,fetcher,()=>new EngineWorker()),run=lab.submit(config);while(lab.active)await new Promise(resolve=>setTimeout(resolve,5));
  const project=await createLabProject(lab,run.id);assert.equal(project.experiment.status,'completed',JSON.stringify(project.experiment.logs));const state=project.experiment.states[0],engineRequest={graph,options:state.metrics.requested_options,demographics:input};
  const nativeRun=await native(engineRequest,false);assert.equal(nativeRun.status,0,nativeRun.stderr);assert.deepEqual(engine.execute(engineRequest),JSON.parse(nativeRun.stdout));
  const before=executions,restorer=new WasmCatalog(manifest,fetcher,()=>{throw new Error('No engine on Open');}),opened=await restorer.restoreProject(parseLabProject(JSON.stringify(project)));assert.equal(executions,before);assert.deepEqual(opened.config.demographics[code].counts,input.counts);assert.equal((await restorer.api(`/api/runs/${opened.id}/${code}/map`)).features.length,graph.geoids.length);
  for(const mutate of [p=>p.experiment.config.demographics[code].counts[graph.geoids[0]].minority++,p=>delete p.experiment.config.demographics[code].counts[graph.geoids[0]]]){const bad=structuredClone(project);mutate(bad);const count=restorer.jobs.size;await assert.rejects(restorer.restoreProject(bad),/counts|count/);assert.equal(restorer.jobs.size,count);rejections++;}
  const bad={...engineRequest,demographics:structuredClone(input)};bad.demographics.counts[graph.geoids[0]].minority++;assert.throws(()=>engine.execute(bad),/counts/);assert.notEqual((await native(bad,false)).status,0);rejections++;
  await fs.writeFile(path.join(fixtures,`${structure}-${basis}.bisect`),JSON.stringify(project));projectCases++;exact++;
 }
}
await fs.writeFile('target/demographic-csv-verification.json',JSON.stringify({exact_native_wasm_cases:exact,rejections,state:code,year:graph.year,real_graph_project_cases:projectCases,synthetic_demographics:true,counts_retained:true,engine_on_open:false},null,2));
console.log(`Demographic CSV: ${exact} exact native/WASM comparisons, ${projectCases} real-${code} engine/Save/Open/map cases, ${rejections} malformed/count-tamper rejections; counts retained, no engine on Open.`);
