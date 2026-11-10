const decoder=new TextDecoder('utf-8',{fatal:true});
const crcTable=Uint32Array.from({length:256},(_,n)=>{for(let i=0;i<8;i++)n=n&1?0xedb88320^(n>>>1):n>>>1;return n>>>0;});
function crc32(bytes){let n=0xffffffff;for(const b of bytes)n=crcTable[(n^b)&255]^(n>>>8);return (n^0xffffffff)>>>0;}
// Read only the stored ZIP32 format produced by the cache exporter. Paths never
// become filesystem destinations; unknown members and hidden extents reject.
export function parseCacheBackupZip(bytes){
 return parseStoredZip(bytes,{maxBytes:128*1024*1024,maxEntries:502,maxMemberBytes:64*1024*1024,allowed:name=>['cache-manifest.json','cache-library.json'].includes(name)||/^objects\/sha256\/[0-9a-f]{64}$/.test(name)});
}
export function parseConfiguredSourceZip(bytes){
 const paths=['sources/context-attributes.json','sources/original-context.rctx','sources/context-engine-origin.json','runtime/engine-artifacts.json','bundle.json','plan.rplan','context.rctx','sources/config.yml','sources/run-settings.json','sources/graph.json','sources/coi.json','sources/demographics.json','sources/native-demographics.csv','sources/native-demographic-adoption.json'];
 try{return parseStoredZip(bytes,{maxBytes:32*1024*1024,maxEntries:14,maxMemberBytes:25*1024*1024,allowed:name=>paths.includes(name)});}catch(error){throw new Error(error.message.replaceAll('cache backup ZIP','plan source ZIP').replaceAll('cache ZIP','source ZIP'));}
}
export function parseNativeAggregateBackupZip(bytes){
 try{return parseStoredZip(bytes,{maxBytes:256*1024*1024,maxEntries:20001,minEntries:1,maxMemberBytes:96*1024*1024,allowed:name=>name==='workspace-backup.json'||/^objects\/[0-9a-f]{64}$/.test(name)});}catch(error){throw new Error(error.message.replaceAll('cache backup ZIP','workspace backup ZIP').replaceAll('cache ZIP','workspace ZIP'));}
}
function parseStoredZip(bytes,{maxBytes,maxEntries,minEntries=2,maxMemberBytes,allowed}){
 if(!(bytes instanceof Uint8Array)||bytes.length<22||bytes.length>maxBytes)throw new Error('Invalid cache backup ZIP size.');
 const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),u16=p=>v.getUint16(p,true),u32=p=>v.getUint32(p,true),end=bytes.length-22;
 if(u32(end)!==0x06054b50||u16(end+4)||u16(end+6)||u16(end+20))throw new Error('Unsupported cache ZIP end record.');
 const count=u16(end+10),central=u32(end+16);if(count<minEntries||count>maxEntries||u16(end+8)!==count||central+u32(end+12)!==end)throw new Error('Invalid cache ZIP directory.');
 const entries=new Map(),ranges=[];let offset=central;
 for(let i=0;i<count;i++){
  if(offset+46>end||u32(offset)!==0x02014b50)throw new Error('Invalid cache ZIP member directory.');
  const size=u32(offset+24),packed=u32(offset+20),nl=u16(offset+28),local=u32(offset+42),crc=u32(offset+16),type=(u32(offset+38)>>>16)&0xf000;
  if(offset+46+nl>end||u16(offset+8)!==0x800||u16(offset+10)||u16(offset+30)||u16(offset+32)||u16(offset+34)||size!==packed||size>maxMemberBytes||(type&&type!==0x8000))throw new Error('Unsupported compressed, encrypted, linked or oversized cache ZIP member.');
  const name=decoder.decode(bytes.subarray(offset+46,offset+46+nl));if(!allowed(name))throw new Error('Unknown or unsafe cache ZIP path.');if(entries.has(name))throw new Error('Duplicate cache ZIP path.');
  if(local+30>central||u32(local)!==0x04034b50||u16(local+6)!==0x800||u16(local+8)||u16(local+28)||u32(local+14)!==crc||u32(local+18)!==size||u32(local+22)!==size||u16(local+26)!==nl)throw new Error('Cache ZIP local header disagrees.');
  const start=local+30+nl;if(start+size>central||decoder.decode(bytes.subarray(local+30,start))!==name)throw new Error('Cache ZIP member extent or name disagrees.');
  const data=bytes.subarray(start,start+size);if(crc32(data)!==crc)throw new Error('Cache ZIP member CRC mismatch.');entries.set(name,{start,size,data});ranges.push([local,start+size]);offset+=46+nl;
 }
 if(offset!==end)throw new Error('Cache ZIP directory size disagrees.');ranges.sort((a,b)=>a[0]-b[0]);let cursor=0;for(const [start,end]of ranges){if(start!==cursor)throw new Error('Overlapping or hidden cache ZIP bytes.');cursor=end;}if(cursor!==central)throw new Error('Hidden cache ZIP bytes.');
 return entries;
}
