import assert from 'node:assert/strict';

import fs from 'node:fs/promises';

import path from 'node:path';

import {spawnSync} from 'node:child_process';

import {createHash} from 'node:crypto';

import {instantiateEngine} from '../../web/lab/wasm-engine.js';

import {WasmCatalog} from '../../web/lab/wasm-catalog.js';
import {createLabProject} from '../../web/lab/laboratory-project.js';
import {exactPlanText,exactSourceText} from '../../web/lab/package-files.js';

import {preparePractitionerProject} from '../../web/lab/laboratory-practitioner-project.js';

import {parseProject,MAX_PROJECT_BYTES} from '../../web/lab/project.js';

import {parseWorkbenchArchiveBytes} from '../../web/lab/workbench-archive.js';



const packageRoot=process.argv[2];if(!packageRoot)throw Error('Pass a laboratory package containing CA.');

await fs.mkdir('target/laboratory-practitioner-archive',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/laboratory-practitioner-archive/run-'));

const hash=b=>createHash('sha256').update(b).digest('hex');

const manifest=JSON.parse(await fs.readFile(path.join(packageRoot,'catalog.json'),'utf8'));

const wasm=await fs.readFile(path.join(packageRoot,'bisect_wasm.wasm')),engine=await instantiateEngine(wasm);

const graph=JSON.parse(await fs.readFile(path.join(packageRoot,manifest.graphs['CA:2020'].graph_ref),'utf8'));

const csv='geoid,c000,cns01,cns02,cns05,cns07,cns08,cns09,cns10,cns11\n'+graph.geoids.map((id,i)=>id+','+(i%3===0?'0,0,0,0,0,0,0,0,0':'1000,10,20,30,'+(i%2?'400,40,100,50,50':'100,200,200,100,100'))).join('\n')+'\n';

const character=engine.execute({operation:'import-character-csv',state:'CA',year:'2020',data_year:'2022',kind:'economic',source_label:'SYNTHETIC character values; not observed LODES/ACS data',source_base64:Buffer.from(csv).toString('base64')});

const fixture={graph,character,options:{character_alpha:.5,metis_objective:'volume',metis_trials:3,structure:'standard-bisect',weights:'economic-character',search:'single',districts:52,seed:'18446744073709551615',seeds:3,steps:4,percentile:.5,alpha_county:0,balance_tolerance:10,area_swing:1.1,iterations:10}};

assert.equal(graph.geoids.length,9129);

const native=path.join(root,'native.exe');await fs.copyFile('target/release/examples/execute_tool_request.exe',native);

const nativeHash=hash(await fs.readFile(native));

const fetcher=async name=>{const bytes=await fs.readFile(path.join(packageRoot,name));return {ok:true,arrayBuffer:async()=>bytes.buffer.slice(bytes.byteOffset,bytes.byteOffset+bytes.length)};};

const operations=[];

class Worker{postMessage(m){setImmediate(()=>{if(this.terminated)return;try{operations.push(m.type==='run'?m.request.operation||'engine':m.type);this.onmessage({data:m.type==='initialize'?{type:'ready'}:{type:'result',id:m.id,result:engine.execute(m.request)}});}catch(e){this.onmessage({data:{type:'error',id:m.id,error:e.message}});}});}terminate(){this.terminated=true;}}

const lab=new WasmCatalog(manifest,fetcher,()=>new Worker());

const config={name:'California character audit export',mode:'state',states:['CA'],year:'2020',chamber:'congressional',timeout_seconds:120,...fixture.options,characters:{CA:fixture.character}};

const job=lab.submit(config);while(lab.active)await new Promise(r=>setTimeout(r,10));

const completed=lab.jobs.get(job.id);assert.equal(completed.status,'completed',JSON.stringify(completed.states));

await fs.writeFile(path.join(root,'CA-laboratory.bisect-lab'),JSON.stringify(await createLabProject(lab,job.id)));
const result=await lab.exportPractitionerProject(job.id,'CA');

assert(result.archive!==undefined,'Full CA character export must use a bounded archive');

const bytes=typeof result.archive==='string'?new TextEncoder().encode(result.archive):result.archive;

const file=path.join(root,'CA-character-audit.bisect-workbench');await fs.writeFile(file,bytes);

assert.deepEqual(await parseWorkbenchArchiveBytes(bytes),result.project);

assert.deepEqual(result.project.files.plan.provenance.producer.character_input,fixture.character);

assert.equal(Buffer.from(result.project.files.auditPlanText.base64,'base64').toString('utf8'),result.exported.document_json);

assert.equal(Buffer.from(result.project.files.auditContextText.base64,'base64').toString('utf8'),result.exported.context_json);

assert.throws(()=>parseProject(JSON.stringify(result.project)),/25 MB import limit/);

assert(bytes.byteLength>MAX_PROJECT_BYTES);
assert.equal(exactPlanText(result.project.files.auditPlanText),result.exported.document_json);
assert.throws(()=>exactSourceText(result.project.files.auditPlanText),/3 MiB/);
assert.throws(()=>exactPlanText({name:'oversized.rplan',base64:Buffer.alloc(8*1024*1024+1,32).toString('base64')}),/byte limit|8 MiB/);

const request={operation:'export-engine-plan',request:{graph:fixture.graph,options:completed.states[0].metrics.recorded_options,character:fixture.character},assignments:lab.outputs.get(`${job.id}:CA`),label:config.name+' · CA',chamber:config.chamber,created_at:new Date(completed.created_unix*1000).toISOString()};

const requestFile=path.join(root,'native-request.json');await fs.writeFile(requestFile,JSON.stringify(request,(_key,value)=>Object.is(value,-0)?'__negative_zero__':value).replaceAll('"__negative_zero__"','-0.0'));

const proc=spawnSync(native,[requestFile],{encoding:'utf8',windowsHide:true,maxBuffer:64*1024*1024});assert.equal(proc.status,0,proc.stderr);assert.deepEqual(result.exported,JSON.parse(proc.stdout));

const small=await preparePractitionerProject({name:'Ordinary project',files:{},operation:'plan-sweep',constraints:[]});assert.equal(small.archive,undefined);assert.deepEqual(parseProject(JSON.stringify(small.project)),small.project);
const manyValues=await preparePractitionerProject({name:'Retained high-value-count result',files:{},operation:'plan-sweep',constraints:[],lastOperation:'plan-sweep',result:{values:Array.from({length:500001},(_,i)=>i)}});assert(manyValues.archive!==undefined);assert.equal(manyValues.project.result.values.length,500001);

await assert.rejects(preparePractitionerProject({name:'Invalid source',files:{unknown:1},operation:'plan-sweep',constraints:[]}),/input|file/i);

const sources=[];for(const name of ['web/lab/wasm-catalog.js','web/lab/lab.js','web/lab/laboratory-practitioner-project.js','web/lab/project.js','web/lab/package-files.js','scripts/web/test_laboratory_practitioner_archive.mjs'])sources.push({path:name,sha256:hash(await fs.readFile(name))});

await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-large-laboratory-practitioner-archive',sources,native:{path:native,sha256:nativeHash},wasm_sha256:hash(wasm),archive:{path:file,sha256:hash(bytes),byte_length:bytes.byteLength},state:'CA',units:9129,districts:52,full_native_export_equal:true,original_plan_context_bytes_retained:true,ordinary_project_limit_unchanged:true,operations,scope:'Actual full CA laboratory character run and native-equal export, bounded archive roundtrip. Browser download/Open and complete builder acceptance remain separate.'},null,2)+'\n');

console.log(JSON.stringify({root,status:'passed-large-laboratory-practitioner-archive',archive_bytes:bytes.byteLength}));

