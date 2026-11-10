import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import {parseCacheBackupZip} from '../../web/lab/fletch-cache-backup-zip.js';
const index=JSON.parse(await fs.readFile('target/fletch-cache-browser/backup-index.json','utf8'));
const original=new Uint8Array(await fs.readFile('target/fletch-cache-browser/'+index.volumes[0].name)),v=new DataView(original.buffer,original.byteOffset,original.byteLength),end=original.length-22,central=v.getUint32(end+16,true);
const members=parseCacheBackupZip(original);assert.equal(members.size,3);
function reject(change,pattern){const bytes=original.slice(),view=new DataView(bytes.buffer);change(bytes,view);assert.throws(()=>parseCacheBackupZip(bytes),pattern);}
reject((b,v)=>v.setUint16(end+4,1,true),/end record/);
reject((b,v)=>v.setUint16(central+8,0x801,true),/encrypted/);
reject((b,v)=>v.setUint16(central+10,8,true),/compressed/);
reject((b,v)=>v.setUint32(central+38,(0o120777<<16)>>>0,true),/linked/);
reject((b,v)=>v.setUint32(central+42,1,true),/local header/);
reject((b,v)=>v.setUint32(central+24,0xffffffff,true),/oversized/);
reject((b,v)=>v.setUint16(central+30,1,true),/Unsupported/);
const object=[...members.entries()].find(([name])=>name.startsWith('objects/'));
reject(b=>{b[object[1].start]^=1;},/CRC/);
// Replace the first name in both headers so matching local/central names do not
// conceal an unexpected path. Neither checksum covers the name.
reject((b,v)=>{b[30]=46;b[central+46]=46;},/unsafe/);
let second=central+46+v.getUint16(central+28,true);
reject((b,v)=>{v.setUint32(second+42,v.getUint32(central+42,true),true);},/local header|extent|name/);
assert.throws(()=>parseCacheBackupZip(original.subarray(0,original.length-1)),/end record/);
console.log('Cache backup ZIP parser: stored native volume accepted; split/encrypted/compressed/link/offset/size/extra-field/CRC/unsafe-path/overlap/truncation guards passed.');
