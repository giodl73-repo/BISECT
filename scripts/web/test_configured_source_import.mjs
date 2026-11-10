import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {importConfiguredSourceBundle,readConfiguredSourceBundle} from '../../web/lab/configured-source-import.js';
import {parseConfiguredSourceZip} from '../../web/lab/fletch-cache-backup-zip.js';
import {storedZipBytes} from '../../web/lab/reproducibility-files.js';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
import {configuredSourceBundle} from '../../web/lab/configured-source-bundle.js';
import {configRunRequest,configuredRunExportRequest} from '../../web/lab/config-run.js';
const root='target/configured-source-import';await fs.mkdir(root,{recursive:true});let refusals=0;
const sha=b=>createHash('sha256').update(b).digest('hex');
for(const i of [0,1]){const bytes=new Uint8Array(await fs.readFile(`target/configured-source-bundle/case-${i}.zip`)),p=await importConfiguredSourceBundle(bytes),original=JSON.parse(await fs.readFile(`target/native-demographic-adoption/case-${i}.bisect`,'utf8'));assert.equal(p.operation,'audit-rplan-text');assert.equal(p.result,null);assert.equal(p.lastOperation,null);assert.deepEqual(p.files.demographicSource,original.files.demographicSource);assert.deepEqual(p.files.nativeDemographicReceipt,original.files.nativeDemographicReceipt);assert.deepEqual(p.files.configRunInputs.graph,original.files.configRunInputs.graph);await fs.writeFile(`${root}/case-${i}.bisect-workbench`,JSON.stringify(p));}
const bytes=new Uint8Array(await fs.readFile('target/configured-source-bundle/case-1.zip')),members=parseConfiguredSourceZip(bytes);
async function rebuilt(change,{rehash=true}={}){const entries=new Map([...members].map(([name,v])=>[name,v.data.slice()]));change(entries);if(rehash){const manifest=JSON.parse(new TextDecoder().decode(entries.get('bundle.json')));manifest.files=Object.fromEntries([...entries].filter(([n])=>n!=='bundle.json').map(([n,b])=>[n,{bytes:b.length,sha256:sha(b)}]));entries.set('bundle.json',new TextEncoder().encode(JSON.stringify(manifest)));}await assert.rejects(importConfiguredSourceBundle(storedZipBytes([...entries])));refusals++;}
await rebuilt(e=>e.delete('sources/graph.json'));
await rebuilt(e=>e.delete('sources/native-demographics.csv'));
await rebuilt(e=>e.set('sources/config.yml',new TextEncoder().encode('changed')),{rehash:false});
await rebuilt(e=>{const g=JSON.parse(new TextDecoder().decode(e.get('sources/graph.json')));g.state='IA';e.set('sources/graph.json',new TextEncoder().encode(JSON.stringify(g)));});
await rebuilt(e=>{const r=JSON.parse(new TextDecoder().decode(e.get('sources/native-demographic-adoption.json')));r.source_sha256='0'.repeat(64);e.set('sources/native-demographic-adoption.json',new TextEncoder().encode(JSON.stringify(r)));});
await rebuilt(e=>e.set('bundle.json',new TextEncoder().encode('{"__proto__":{}}')),{rehash:false});
await rebuilt(e=>e.set('unexpected.js',new TextEncoder().encode('alert(1)')));
const end=bytes.length-22,central=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength).getUint32(end+16,true);
for(const mutate of [v=>v.setUint16(central+10,8,true),v=>v.setUint16(central+8,0x801,true),v=>v.setUint32(central+38,(0o120777<<16)>>>0,true),v=>v.setUint32(central+42,1,true),v=>v.setUint32(central+24,0xffffffff,true)]){const b=bytes.slice();mutate(new DataView(b.buffer));await assert.rejects(importConfiguredSourceBundle(b));refusals++;}
const corrupt=bytes.slice();corrupt[members.get('plan.rplan').start]^=1;await assert.rejects(importConfiguredSourceBundle(corrupt),/CRC/);refusals++;
let reads=0;await assert.rejects(readConfiguredSourceBundle({size:32*1024*1024+1,arrayBuffer:async()=>{reads++;}}));assert.equal(reads,0);
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm')),fixture=JSON.parse(await fs.readFile('target/native-demographic-adoption/case-1.bisect','utf8')),files=fixture.files,g=JSON.parse(Buffer.from(files.configRunInputs.graph.base64,'base64').toString()),coiText=JSON.stringify({[g.geoids[0]]:2});files.configRunInputs.coi={name:'original-coi.json',base64:Buffer.from(coiText).toString('base64')};const result=engine.execute(configRunRequest(files.configSource,files.configSettings,files.configRunInputs,files.configRunSettings));result.practitioner_export=engine.execute(configuredRunExportRequest(result,{label:'COI source roundtrip',created_at:'2026-10-13T00:00:00Z'}));const coiBundle=await configuredSourceBundle(result,files),coiOpened=await importConfiguredSourceBundle(coiBundle.bytes);assert.deepEqual(coiOpened.files.configRunInputs.coi,files.configRunInputs.coi);await fs.writeFile(root+'/coi.zip',coiBundle.bytes);const bad=structuredClone(files);bad.configRunInputs.coi.base64=Buffer.from('{}').toString('base64');await assert.rejects(configuredSourceBundle(result,bad),/COI source/);refusals++;
await fs.writeFile(root+'/evidence.json',JSON.stringify({exact_archive_imports:3,hostile_package_refusals:refusals,coi_original_name_and_bytes_retained:true,oversize_rejected_before_read:true,original_sources_retained:true,opens_for_audit:true,no_run_or_claimed_result:true},null,2));console.log(`Source ZIP import: three exact source-preserving audit imports including COI, ${refusals} malformed/tampered/hostile refusals and oversized pre-read rejection; no recorded execution result.`);
