import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {LODES_DEFAULTS,lodesRequest,readLodesSource,lodesDownload} from '../../web/lab/lodes-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/lodes-source-wasm';await fs.mkdir(root,{recursive:true});
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const wac='w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS08,CNS09,CNS10,CNS11\n',od='h_geocode,w_geocode,S000\n';
const fixtures=[
 ['wac','workplace',wac+'440010002001001,10,1,2,3,4,5,6,7,8\n440010001001001,20,2,4,6,8,10,12,14,16\n440010002001002,30,3,6,9,12,15,18,21,24\n','ready'],
 ['wac','defaults',wac+'440010001001001,bad,1.5,-2,NaN,4,5,6,7,8\n','ready'],
 ['wac','foreign',wac+'370010001001001,10,1,2,3,4,5,6,7,8\n','unavailable'],
 ['wac','short','CNS08,w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS09,CNS10,CNS11\n1,440010001001001\n','unavailable'],
 ['wac','utf8',wac+'1234567890é,10,1,2,3,4,5,6,7,8\n','unavailable'],
 ['wac','quoted',wac.replaceAll(/([^,\n]+)/g,'"$1"')+'"440010001001001","10","1","2","3","4","5","6","7","8"\r\n','ready'],
 ['wac','header-only',wac,'unavailable'],
 ['od','commuting',od+'440010002001001,370010001001001,12\n440010001001001,440010002001001,5\n440010002001002,370010001001002,8\n','not-applicable'],
 ['od','defaults-od',od+'440010001001001,440010002001001,bad\n440010001001001,440010002001001,1.5\n','not-applicable'],
 ['od','short-od','S000,h_geocode,w_geocode\n1,440010001001001\n','not-applicable'],
 ['od','utf8-od',od+'1234567890é,440010001001001,1\n','not-applicable'],
 ['od','maximum',od+'440010001001001,440010002001001,18446744073709551615\n','not-applicable']
];
for(const [kind,name,raw,status]of fixtures){
 const bytes=new TextEncoder().encode(raw),source=await readLodesSource({name:name+'.csv',size:bytes.length,arrayBuffer:async()=>bytes.buffer}),inputs={source,settings:{...LODES_DEFAULTS,kind}},request=lodesRequest(inputs),result=engine.execute(request);
 assert.equal(request.input.csv_text,raw);await fs.writeFile(root+'/request.json',JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8'})),result);
 await fs.writeFile(root+'/source.csv',raw);const native=spawnSync('target/debug/examples/lodes_reference.exe',[kind,root+'/source.csv',root+'/native.csv'],{encoding:'utf8'});assert.equal(native.status,0,native.stderr);assert.equal(await fs.readFile(root+'/native.csv','utf8'),result.csv_text);
 assert.equal(result.character_status,status);assert.equal(result.execution_performed,false);assert.equal(result.source_sha256,createHash('sha256').update(bytes).digest('hex'));assert.equal(result.csv_sha256,createHash('sha256').update(result.csv_text).digest('hex'));assert.equal(new TextDecoder().decode(lodesDownload(result,'csv').bytes),result.csv_text);
 if(status==='ready')assert.deepEqual(JSON.parse(new TextDecoder().decode(lodesDownload(result,'character').bytes)),result.character_input);else assert.throws(()=>lodesDownload(result,'character'));
 if(name==='workplace')assert.equal(result.csv_text,'geoid,c000,cns07,cns09,cns10,cns11,cns01,cns02,cns05,cns08\n44001000100,20,8,12,14,16,2,4,6,10\n44001000200,40,16,24,28,32,4,8,12,20\n');
 if(name==='commuting')assert.equal(result.csv_text,'home_geoid,work_geoid,s000\n44001000100,44001000200,5\n44001000200,37001000100,20\n');
 if(name.startsWith('short'))assert.equal(result.skipped_short_rows,1);if(name.startsWith('utf8'))assert.equal(result.skipped_geoid_rows,1);
 const project=createProject({name:'LODES '+name,files:{lodesInputs:inputs},operation:'prepare-lodes-source',constraints:[],result,lastOperation:'prepare-lodes-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
for(const [kind,raw]of [['wac',''],['wac','missing\n'],['od','missing\n'],['od',od+'440010001001001,440010002001001,18446744073709551615\n440010001001002,440010002001002,1\n']]){
 assert.throws(()=>engine.execute({operation:'prepare-lodes-source',input:{...LODES_DEFAULTS,kind,source_name:'bad.csv',csv_text:raw}}));await fs.writeFile(root+'/source.csv',raw);const p=spawnSync('target/debug/examples/lodes_reference.exe',[kind,root+'/source.csv',root+'/bad.csv'],{encoding:'utf8'});assert.notEqual(p.status,0);assert.doesNotMatch(p.stderr,/panicked/);
}
let read=false;await assert.rejects(()=>readLodesSource({name:'huge.csv',size:16*1024*1024+1,arrayBuffer(){read=true;}}));assert.equal(read,false);await assert.rejects(()=>readLodesSource({name:'bad.csv',size:1,arrayBuffer:async()=>Uint8Array.of(255).buffer}));
assert.throws(()=>lodesRequest({source:null,settings:LODES_DEFAULTS}));
console.log('LODES source: 12 native-host/WASM/native CSV comparisons, independent WAC/OD sum and sorting oracles, numeric defaults, short/UTF-8 row safety, maximum u64 and overflow rejection, source/hash/project/download preservation passed.');
