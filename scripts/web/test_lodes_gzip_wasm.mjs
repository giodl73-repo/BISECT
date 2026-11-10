import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {gzipSync} from 'node:zlib';
import {execFileSync,spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {LODES_DEFAULTS,lodesRequest,readLodesSource,lodesDownload} from '../../web/lab/lodes-files.js';
import {createProject,parseProject} from '../../web/lab/project.js';
const root='target/lodes-gzip-wasm';await fs.mkdir(root,{recursive:true});const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const rawWac='w_geocode,C000,CNS01,CNS02,CNS05,CNS07,CNS08,CNS09,CNS10,CNS11\n440010001001001,10,1,2,3,4,5,6,7,8\n';
const rawOd='h_geocode,w_geocode,S000\n440010001001001,370010002001001,18446744073709551615\n';
const fixtures=[['workplace','wac',gzipSync(rawWac),rawWac],['commuting','od',gzipSync(rawOd),rawOd],['trailing-member','wac',Buffer.concat([gzipSync(rawWac),gzipSync(rawWac)]),rawWac],['large-expanded','wac',gzipSync(rawWac+(' '.repeat(1023)+'\n').repeat(20*1024)),rawWac+(' '.repeat(1023)+'\n').repeat(20*1024)]];
for(const [name,kind,bytes,raw]of fixtures){
 const source=await readLodesSource({name:name+'.csv.gz',size:bytes.length,arrayBuffer:async()=>Uint8Array.from(bytes).buffer});const inputs={source,settings:{...LODES_DEFAULTS,kind}},request=lodesRequest(inputs);assert.equal(request.input.source_base64,bytes.toString('base64'));assert.equal(request.input.csv_text,undefined);const result=engine.execute(request);
 await fs.writeFile(root+'/request.json',JSON.stringify(request));assert.deepEqual(JSON.parse(execFileSync('target/release/examples/execute_tool_request.exe',[root+'/request.json'],{encoding:'utf8'})),result);
 await fs.writeFile(root+'/source.csv.gz',bytes);const p=spawnSync('target/debug/examples/lodes_reference.exe',[kind+'-gzip',root+'/source.csv.gz',root+'/native.csv'],{encoding:'utf8'});assert.equal(p.status,0,p.stderr);assert.equal(await fs.readFile(root+'/native.csv','utf8'),result.csv_text);
 assert.equal(result.source_encoding,'gzip');assert.equal(result.source_sha256,hash(bytes));assert.equal(result.source_byte_length,bytes.length);assert.equal(result.decoded_sha256,hash(raw));assert.equal(result.decoded_byte_length,Buffer.byteLength(raw));assert.equal(new TextDecoder().decode(lodesDownload(result,'csv').bytes),result.csv_text);
 const project=createProject({name:'LODES gzip '+name,files:{lodesInputs:inputs},operation:'prepare-lodes-source',constraints:[],result,lastOperation:'prepare-lodes-source'});assert.deepEqual(parseProject(JSON.stringify(project)),project);assert.equal(project.files.lodesInputs.source.base64,bytes.toString('base64'));await fs.writeFile(root+'/'+name+'.bisect',JSON.stringify(project));await fs.writeFile(root+'/'+name+'-result.json',JSON.stringify(result));
}
const damaged=Buffer.from(gzipSync(rawWac));damaged[damaged.length-8]^=1;
for(const bytes of [damaged,gzipSync(rawWac).subarray(0,16),Buffer.from([31,139,0,0])]){
 const request={operation:'prepare-lodes-source',input:{...LODES_DEFAULTS,source_name:'bad.gz',source_base64:bytes.toString('base64')}};assert.throws(()=>engine.execute(request));await fs.writeFile(root+'/source.csv.gz',bytes);const p=spawnSync('target/debug/examples/lodes_reference.exe',['wac-gzip',root+'/source.csv.gz',root+'/bad.csv'],{encoding:'utf8'});assert.notEqual(p.status,0);assert.doesNotMatch(p.stderr,/panicked/);
}
for(const change of [{csv_text:rawWac,source_base64:gzipSync(rawWac).toString('base64')},{source_base64:'not base64!'},{}])assert.throws(()=>engine.execute({operation:'prepare-lodes-source',input:{...LODES_DEFAULTS,source_name:'bad.gz',...change}}));
const tooLarge=gzipSync(rawWac+(' '.repeat(1023)+'\n').repeat(128*1024));assert.throws(()=>engine.execute({operation:'prepare-lodes-source',input:{...LODES_DEFAULTS,source_name:'expansion.gz',source_base64:tooLarge.toString('base64')}}),/128 MiB/);
const disguised=await readLodesSource({name:'plain.csv.gz',size:Buffer.byteLength(rawWac),arrayBuffer:async()=>new TextEncoder().encode(rawWac).buffer});assert.equal(lodesRequest({source:disguised,settings:LODES_DEFAULTS}).input.csv_text,rawWac);
console.log('LODES gzip: four native-host/WASM/native gzip CSV cases, >16 MiB expanded source, single-member parity, distinct byte hashes, raw compressed project persistence, CRC/truncation/header failures, ambiguous/invalid source rejection and 128 MiB expansion rejection passed.');
