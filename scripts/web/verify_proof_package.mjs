import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';

const manifestPath=new URL('./proof-runtime-dependencies.json',import.meta.url);
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');

export async function verifyProofPackage(directory,{servedAssets=null}={}){
 const root=path.resolve(directory),inventory=JSON.parse(await fs.readFile(manifestPath,'utf8'));
 assert.equal(inventory.schema,'bisect-proof-runtime-dependencies-v1');
 const checked=new Map(),visited=new Set();
 async function read(name){
  assert.equal(typeof name,'string');assert(!name.includes('\\')&&!name.includes('?')&&!name.includes('#')&&!name.includes(':'),'Invalid package path: '+name);
  const target=path.resolve(root,name);assert(target.startsWith(root+path.sep),'Package path escapes root: '+name);
  const bytes=await fs.readFile(target);checked.set(name,{sha256:digest(bytes),bytes:bytes.length});return bytes;
 }
 const json=async name=>JSON.parse((await read(name)).toString('utf8'));
 for(const field of ['entry_points','runtime_files','distribution_files']){
  assert(Array.isArray(inventory[field])&&inventory[field].length>0);
  assert.equal(new Set(inventory[field]).size,inventory[field].length);
  for(const name of inventory[field])await read(name);
 }
 const queue=[...inventory.entry_points,'roundingsat/roundingsat.mjs'];
 while(queue.length){
  const name=queue.shift();if(visited.has(name))continue;visited.add(name);
  const text=(await read(name)).toString('utf8'),refs=[];
  if(name.endsWith('.html'))refs.push(...[...text.matchAll(/(?:src|href)=["']([^"']+\.(?:js|mjs|css)(?:[?#][^"']*)?)["']/g)].map(m=>m[1]));
  else refs.push(...[...text.matchAll(/(?:from\s*|import\s*\(|new URL\s*\()(['"])(\.\.?\/[^'"]+)\1/g)].map(m=>m[2]));
  for(const ref of refs){
   const url=new URL(ref,'https://package.invalid/'+name);assert.equal(url.origin,'https://package.invalid');
   const dependency=decodeURIComponent(url.pathname.slice(1));
   // Directory URL bases are covered by the explicit runtime file inventory.
   if(dependency.endsWith('/')){const directory=path.resolve(root,dependency);assert(directory.startsWith(root+path.sep),'Directory URL escapes package');assert((await fs.stat(directory)).isDirectory());continue;}
   await read(dependency);if(/\.(?:js|mjs|html)$/.test(dependency))queue.push(dependency);
  }
 }
 const solver=await json('roundingsat/build-receipt.json');assert.equal(solver.schema,'bisect-roundingsat-wasm-build-v1');
 assert.deepEqual(Object.keys(solver.files).sort(),['roundingsat.mjs','roundingsat.wasm']);
 for(const [name,expected]of Object.entries(solver.files)){const actual=checked.get('roundingsat/'+name);assert.equal(actual.sha256,expected.sha256);assert.equal(actual.bytes,expected.bytes);}
 const checker=await json('proof-checker/proof-checker-build.json');assert.equal(checker.schema,'bisect-veripb-wasm-build-v1');
 assert.equal(checked.get('proof-checker/bisect_proof_checker.wasm').sha256,checker.wasm_sha256);
 assert.equal(checked.get('proof-checker/bisect_proof_checker.wasm').bytes,checker.wasm_bytes);
 assert.equal(checked.get('proof-checker/provenance.json').sha256,checker.vendor_provenance_sha256);
 for(const receiptName of ['roundingsat/package-receipt.json','proof-checker/package-receipt.json']){
  const receipt=await json(receiptName);
  for(const [name,expected]of Object.entries(receipt.files)){await read(name);const actual=checked.get(name);assert.equal(actual.sha256,expected.sha256,name);assert.equal(actual.bytes,expected.bytes,name);}
 }
 const proof=await json('native-proof-build.json');assert.equal(proof.schema,'bisect-native-proof-preview-v1');
 for(const [name,expected]of Object.entries(proof.files)){await read(name);assert.deepEqual(checked.get(name),{sha256:expected.sha256,bytes:expected.bytes},name);}
 for(const [name,expected]of Object.entries(proof.wasm_dependencies)){await read(name);assert.equal(checked.get(name).sha256,expected,name);}
 if(servedAssets)for(const [name,expected]of Object.entries(servedAssets)){
  const key=name.replaceAll('\\','/');assert(checked.has(key),'Runtime asset is outside declared entry-point closure: '+key);assert.deepEqual(checked.get(key),expected,key);
 }
 return {schema:'bisect-proof-package-check-v1',entry_points:inventory.entry_points,modules_and_pages:visited.size,checked_files:Object.fromEntries(checked),served_assets_checked:servedAssets?Object.keys(servedAssets).length:0,scope:'Declared entry-point closure and exact packaged identities; runtime workflows require separate browser evidence'};
}

if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
 const root=process.argv[2];assert(root,'Specify the package directory.');
 const observed=process.argv[3]?JSON.parse(await fs.readFile(process.argv[3],'utf8')).served_assets:null;
 console.log(JSON.stringify(await verifyProofPackage(root,{servedAssets:observed}),null,2));
}
