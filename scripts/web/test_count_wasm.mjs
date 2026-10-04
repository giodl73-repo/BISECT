import fs from 'node:fs/promises';
import path from 'node:path';
import assert from 'node:assert/strict';
import {instantiateEngine} from '../../web/lab/wasm-engine.js';
const engine=await instantiateEngine(await fs.readFile('target/wasm32-unknown-unknown/release/bisect_wasm.wasm'));
async function filesAt(dir,prefix='') {
 const files={};
 for(const entry of await fs.readdir(dir,{withFileTypes:true})){
  const key=prefix+entry.name, full=path.join(dir,entry.name);
  if(entry.isDirectory())Object.assign(files,await filesAt(full,key+'/'));
  else files[key]=[...await fs.readFile(full)];
 }
 return files;
}
const files=await filesAt('docs/examples/rcount-golden-packages/summary-basic');
const result=engine.execute({operation:'verify-count-files',files});
const expected=JSON.parse(await fs.readFile('docs/examples/rcount-golden-packages/summary-basic/transcripts/verify-transcript.json','utf8'));
assert.deepEqual(result,expected);
const source=Object.keys(files).find(key=>key.startsWith('sources/')&&key!=='sources/source-index.json');
assert.ok(source);files[source][0]^=1;
const tampered=engine.execute({operation:'verify-count-files',files});
assert.equal(tampered.status,'fail');assert.ok(tampered.checks.some(c=>c.equation_id==='source_hash_match'&&c.status==='fail'));
assert.equal(engine.execute({operation:'replay-count-audits',files}).status,'fail');
console.log('WASM package verification: native golden transcript parity and modified-source rejection passed.');

for(const name of ['comparison','kaplan-markov','minerva','athena-boundary','bad-comparison']) {
 const dir='target/rcount-wasm-fixtures/'+name, bytes=await filesAt(dir);
 const native=JSON.parse(await fs.readFile(dir+'/native-verification.json','utf8'));
 assert.deepEqual(engine.execute({operation:'verify-count-files',files:bytes}),native,name+' verification');
 const replay=engine.execute({operation:'replay-count-audits',files:bytes});
 if(native.status==='pass') {
  assert.deepEqual(replay.runs,JSON.parse(await fs.readFile(dir+'/native-replay.json','utf8')),name+' replay');
  if(name==='athena-boundary')assert.equal(replay.status,'boundary');
  else assert.equal(replay.status,'pass',name);
 } else {assert.equal(replay.status,'fail');assert.deepEqual(replay.runs,JSON.parse(await fs.readFile(dir+'/native-replay.json','utf8')));}
}
console.log('WASM replay: native comparison, Kaplan-Markov, Minerva and explicit Athena boundary parity passed; invalid package stays failed while diagnostic replay matches native.');

const portable=await filesAt('docs/examples/rcount-golden-packages/summary-basic');
const index=JSON.parse(Buffer.from(portable['sources/source-index.json']).toString('utf8'));
for(const source of index.sources)source.path=source.path.replaceAll('/','\\');
portable['sources/source-index.json']=[...Buffer.from(JSON.stringify(index))];
assert.equal(engine.execute({operation:'verify-count-files',files:portable}).status,'pass');
index.sources[0].path='sources\\..\\manifest.json';
portable['sources/source-index.json']=[...Buffer.from(JSON.stringify(index))];
assert.equal(engine.execute({operation:'verify-count-files',files:portable}).status,'fail');
console.log('Package source paths: native Windows separator portability and traversal rejection passed.');
