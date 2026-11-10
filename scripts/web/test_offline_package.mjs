import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import {offlineSelection,offlineDigest,validateOfflineManifest,offlineManifestText} from '../../web/lab/offline-package.js';
const root=path.resolve(process.argv[2]||'dist/wasm-lab-build'),m=JSON.parse(await fs.readFile(root+'/offline-manifest.json','utf8'));
validateOfflineManifest(m);
for(const [name,a]of Object.entries(m.assets)){const file=await fs.readFile(root+'/'+name);assert.equal(file.length,a.bytes,name);assert.equal(await offlineDigest(file),a.sha256,name);}
const all=offlineSelection(m,Object.keys(m.states));assert.equal(new Set(all.paths).size,all.paths.length);
const shell=offlineSelection(m,[]);assert(shell.bytes>23_000_000);assert(!shell.paths.some(p=>p.includes('project')&&p.endsWith('.bisect')));
for(const key of ['toolkit/roundingsat/roundingsat.wasm','toolkit/proof-checker/bisect_proof_checker.wasm'])if(m.assets[key])assert(m.shell.includes(key));
for(const mutate of [d=>d.assets['../escape.js']={sha256:'0'.repeat(64),bytes:1},d=>d.assets['https:evil.example']={sha256:'0'.repeat(64),bytes:1},d=>d.shell.push(d.shell[0]),d=>d.shell.push('missing'),d=>d.states['RI:2020']=['index.html'],d=>d.assets['index.html'].bytes=Infinity,d=>d.extra=true]){const d=structuredClone(m);mutate(d);assert.throws(()=>validateOfflineManifest(d));}
assert.throws(()=>offlineSelection(m,['ZZ:1900']));assert.throws(()=>offlineSelection(m,['RI:2020','RI:2020']));
console.log(`PASS offline package: ${Object.keys(m.assets).length} exact hashes/sizes, ${m.shell.length} shell assets, ${Object.keys(m.states).length} state/year packs, unsafe/unknown/repeated selections refused`);

await assert.rejects(offlineManifestText(new Response(new Uint8Array(2*1024*1024+1))),/2 MiB/);await assert.rejects(offlineManifestText(new Response(new Uint8Array([255]))));
