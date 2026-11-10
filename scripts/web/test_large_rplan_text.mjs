import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const option=n=>process.argv.find(a=>a.startsWith('--'+n+'='))?.slice(n.length+3);
const wasm=option('wasm'),native=option('native');if(!wasm||!native)throw Error('Pass explicit --wasm and --native CLI.');
await fs.mkdir('target/laboratory-practitioner-archive',{recursive:true});
const root=await fs.mkdtemp(path.resolve('target/laboratory-practitioner-archive/text-')),hash=b=>createHash('sha256').update(b).digest('hex');
const fixture='crates/rplan-audit/fixtures/grid3x3-valid.rplan',text=await fs.readFile(fixture,'utf8'),engine=await instantiateEngine(await fs.readFile(wasm));
const baseline=engine.execute({operation:'validate-rplan-text',document_json:text}),records=[];
for(const [name,bytes,accepted]of [['above-old-limit',4*1024*1024,true],['exact-limit',8*1024*1024,true],['above-limit',8*1024*1024+1,false]]){
 const source=text+' '.repeat(bytes-Buffer.byteLength(text)),file=path.join(root,name+'.rplan');await fs.writeFile(file,source);
 const cli=spawnSync(native,['validate','--file',file],{encoding:'utf8',windowsHide:true});assert.equal(cli.status,0,cli.stderr);
 if(accepted)assert.deepEqual(engine.execute({operation:'validate-rplan-text',document_json:source}),baseline);
 else assert.throws(()=>engine.execute({operation:'validate-rplan-text',document_json:source}),/Plan exceeds 8 MiB/);
 records.push({name,source:{path:file,sha256:hash(source),byte_length:bytes},native:{status:cli.status,stdout:cli.stdout,stderr:cli.stderr},wasm:accepted?'same-native-shape-result':'explicit-portable-boundary-native-accepted'});
}
const bom='\ufeff'+text;assert.throws(()=>engine.execute({operation:'validate-rplan-text',document_json:bom}),/Invalid exact artifact/);
await fs.writeFile(path.join(root,'bom.rplan'),bom);const cli=spawnSync(native,['validate','--file',path.join(root,'bom.rplan')],{encoding:'utf8',windowsHide:true});assert.equal(cli.status,1);records.push({name:'BOM',native:{status:cli.status,stdout:cli.stdout,stderr:cli.stderr},wasm:'rejected-without-normalization'});
const sources=[];for(const p of ['crates/bisect-wasm/src/plan_audit_text.rs','scripts/web/test_large_rplan_text.mjs',fixture])sources.push({path:p,sha256:hash(await fs.readFile(p))});
await fs.writeFile(path.join(root,'evidence.json'),JSON.stringify({status:'passed-large-rplan-text',sources,wasm:{path:wasm,sha256:hash(await fs.readFile(wasm))},native:{path:native,sha256:hash(await fs.readFile(native))},baseline,records,scope:'Typed RPLAN shape inspection with actual native CLI validity checks. The above-eight-MiB case is explicitly native-accepted/browser-unsupported; no native failure or complete workflow is inferred.'},null,2)+'\n');
console.log(JSON.stringify({root,status:'passed-large-rplan-text',cases:records.length}));
