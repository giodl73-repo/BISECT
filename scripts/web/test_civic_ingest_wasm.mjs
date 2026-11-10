import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {createProject,parseProject} from '../../web/lab/project.js';
import {CIVIC_DEFAULTS,civicIngestRequest,civicIngestZip} from '../../web/lab/civic-files.js';
import {countResultTables} from '../../web/lab/count-results.js';
const root='target/civic-ingest-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));let parity=0,cli=0,rejections=0;
const header='geoid,comment_id,label,source,source_url,confidence,submitted_at,extra\n',csv=header+'01001000100,c1,Port,public,,high,recorded,"a,b"\n';
const settings={label:'community_submission',submitter:'Recorded curator',timestamp:'2026-10-07T12:00:00Z'};
const req=(text=csv,mode='strict',set=settings)=>({operation:'ingest-civic-coi',input:{label:set.label,source_name:'original.csv',csv_base64:Buffer.from(text).toString('base64')},state:'AL',year:'2020',validate_mode:mode,settings:set});
async function native(r){await fs.writeFile(root+'/request.json',JSON.stringify(r));return JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8',maxBuffer:40*1024*1024,stdio:['ignore','pipe','pipe']}));}
async function exact(r){const out=engine.execute(r);assert.deepEqual(out,await native(r));parity++;const file=`${root}/source-${parity}.csv`,base=`${root}/cli-${parity}`;await fs.writeFile(file,Buffer.from(r.input.csv_base64,'base64'));const ref=spawnSync('target/debug/examples/civic_reference.exe',['ingest',file,r.settings.label,r.state,r.year,r.validate_mode,base,r.settings.submitter],{encoding:'utf8'});assert.equal(ref.status,0,ref.stderr);cli++;
 const prefix=`v1/civic_inputs/${r.settings.label}`;const rawManifest=await fs.readFile(`${base}/${prefix}/manifest.json`,'utf8');const manifest=JSON.parse(rawManifest);
 // The CLI supplies its real path and wall clock; WASM records the explicit supplied values.
 manifest.input.original_path=r.input.source_name;manifest.submitted_at=r.settings.timestamp;manifest.ingested_at=r.settings.timestamp;assert.deepEqual(manifest,out.manifest);assert.equal(JSON.stringify(manifest,null,2),out.manifest_json);
 for(const name of ['original.csv','normalized.csv','validation_log.txt'])assert.deepEqual(await fs.readFile(`${base}/${prefix}/${name}`),Buffer.from(out.files_base64[`${prefix}/${name}`],'base64'));
 assert.equal(Buffer.from(out.files_base64[`${prefix}/manifest.json`],'base64').toString(),out.manifest_json);return out;
}
const result=await exact(req());for(const text of ['\ufeff'+csv.replaceAll('\n','\r\n'),header,csv.replace('Port','<script>'),csv.replace('"a,b"','=1+1')])await exact(req(text));
for(const mode of ['lenient','advisory'])await exact(req(csv.replace('01001000100','bad').replace('public,,','public,http://127.0.0.1,'),mode));
await exact(req(csv,'strict',{...settings,label:'alternate',submitter:'Curator é',timestamp:'supplied-time-label'}));
const many=header+'01001000100,c,L,s,,h,d,x\n'.repeat(50_000);await exact(req(many));const padded=csv.replace('"a,b"','"'+'x'.repeat(3*1024*1024-Buffer.byteLength(csv)+3)+'"');assert.equal(Buffer.byteLength(padded),3*1024*1024);await exact(req(padded));
for(const r of [req(csv.replace('01001000100','bad')),req(Buffer.from([0xff,0xfe,65,0])),req('wrong,headers\na,b\n'),req(csv,'strict',{...settings,label:'../escape'}),req(csv,'strict',{...settings,label:'constructor'}),req(csv,'strict',{...settings,timestamp:''}),req(csv,'strict',{...settings,submitter:'x'.repeat(201)}),{...req(),extra:true},{...req(),settings:{...settings,extra:true}},req(padded+' '),req(many+'01001000100,c,L,s,,h,d,x\n')]){assert.throws(()=>engine.execute(r));await assert.rejects(()=>native(r));rejections++;}
const sources=[{name:'original.csv',base64:Buffer.from(csv).toString('base64')}],inputs={...CIVIC_DEFAULTS,state:'AL',sources};assert.deepEqual(civicIngestRequest(inputs,settings),req());assert.throws(()=>civicIngestRequest({...inputs,sources:[...sources,{...sources[0],name:'second.csv'}]},settings));
const project=createProject({name:'Native civic ingestion',files:{civicInputs:inputs,civicIngestSettings:settings},operation:'ingest-civic-coi',constraints:[],result,lastOperation:'ingest-civic-coi'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(countResultTables(result)[0].rows[0][1],'ingested');assert.throws(()=>parseProject(JSON.stringify({...project,files:{civicIngestSettings:{...settings,label:'../escape'}}})));
assert.throws(()=>civicIngestZip({...result,files_base64:{'../escape':'QQ=='}}));await fs.writeFile(root+'/browser-project.bisect',JSON.stringify(project));await fs.writeFile(root+'/package.zip',civicIngestZip(result));await fs.writeFile(root+'/evidence.json',JSON.stringify({parity,cli,rejections,scope:result.scope},null,2));console.log(`Civic ingestion: ${parity} native/WASM outputs, ${cli} actual CLI byte checks and ${rejections} matching rejected requests passed.`);
