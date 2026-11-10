import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyProofPackage} from './verify_proof_package.mjs';
assert(process.argv[2],'Specify the toolkit package.');
const root=path.resolve('target/proof-package-tests/run-'+Date.now()),baseline=path.resolve(process.argv[2]),casesRoot=path.join(root,'package-mutation-cases');
await fs.mkdir(root,{recursive:true});
await fs.mkdir(casesRoot);
const browser=process.argv[3]?JSON.parse(await fs.readFile(process.argv[3],'utf8')):{served_assets:null};
const accepted=await verifyProofPackage(baseline,{servedAssets:browser.served_assets}),refusals=[];
async function check(name,mutate,pattern,servedAssets=null){
 const directory=path.join(casesRoot,name);await fs.cp(baseline,directory,{recursive:true,errorOnExist:true,force:false});await mutate(directory);
 let refused=false;
 try{await verifyProofPackage(directory,{servedAssets});}catch(error){assert.match(String(error),pattern);refusals.push({case:name,error:String(error)});refused=true;}
 assert(refused,name+' unexpectedly accepted');
}
await check('missing-dynamic-solver',dir=>fs.unlink(path.join(dir,'roundingsat/roundingsat.wasm')),/ENOENT/);
await check('missing-checker-receipt',dir=>fs.unlink(path.join(dir,'proof-checker/proof-checker-build.json')),/ENOENT/);
await check('missing-license',dir=>fs.unlink(path.join(dir,'roundingsat/LICENSE-Boost.txt')),/ENOENT/);
await check('corrupt-checker-bytes',async dir=>{const p=path.join(dir,'proof-checker/bisect_proof_checker.wasm'),bytes=await fs.readFile(p);bytes[bytes.length-1]^=1;await fs.writeFile(p,bytes);},/AssertionError/);
await check('wrong-native-engine-binding',async dir=>{const p=path.join(dir,'native-proof-build.json'),r=JSON.parse(await fs.readFile(p,'utf8'));r.wasm_dependencies['bisect_wasm.wasm']='0'.repeat(64);await fs.writeFile(p,JSON.stringify(r));},/AssertionError/);
await check('auxiliary-html-missing-script',async dir=>{const p=path.join(dir,'proof-checker.html');await fs.writeFile(p,(await fs.readFile(p,'utf8')).replace('proof-checker-page.js','missing-checker-page.js'));},/ENOENT/);
await check('undeclared-observed-runtime',dir=>fs.writeFile(path.join(dir,'new-runtime.js'),'export const added=true;'),/outside declared entry-point closure/,{...browser.served_assets,'new-runtime.js':{sha256:'0'.repeat(64),bytes:24}});
assert.equal(refusals.length,7);
const after=await verifyProofPackage(baseline,{servedAssets:browser.served_assets});assert.deepEqual(after,accepted);
await fs.writeFile(path.join(root,'dependency-mutation-evidence.json'),JSON.stringify({scope:'Package dependency/refusal checks only; runtime and full acceptance separate',entry_points:accepted.entry_points,checked_files:Object.keys(accepted.checked_files).length,served_assets_checked:accepted.served_assets_checked,refusals,frozen_baseline_unchanged:true},null,2));
console.log(`Proof dependencies: ${accepted.entry_points.length} entry points, ${Object.keys(accepted.checked_files).length} exact files, ${accepted.served_assets_checked} browser-served identities; seven damaged/undeclared packages refused.`);
