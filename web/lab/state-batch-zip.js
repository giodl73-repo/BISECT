import {storedZipBytes} from './reproducibility-files.js';

// Output-only national bundle. Input archive and generic ZIP limits are unchanged.
const MAX_EXPANDED=256*1024*1024,MAX_PACKED=128*1024*1024;
export async function compressedStateBatchZip(entries){
 if(!Array.isArray(entries)||entries.length>2000)throw new Error('Invalid State batch ZIP entries.');
 let expanded=0;
 for(const entry of entries){if(!Array.isArray(entry)||entry.length!==2||!(entry[1] instanceof Uint8Array))throw new Error('Invalid State batch ZIP data.');expanded+=entry[1].length;if(expanded>MAX_EXPANDED)throw new Error('State batch ZIP expanded output exceeds 256 MiB.');}
 // Reuse path/duplicate/count validation before invoking a compressor.
 storedZipBytes(entries.map(([name])=>[name,new Uint8Array()]));
 if(typeof CompressionStream!=='function')throw new Error('This larger State batch ZIP requires browser compression support. Save a workbench archive to retain the run.');
 const table=new Uint32Array(256);for(let n=0;n<256;n++){let c=n;for(let k=0;k<8;k++)c=(c>>>1)^((c&1)?0xedb88320:0);table[n]=c>>>0;}
 const crc=bytes=>{let c=0xffffffff;for(const b of bytes)c=table[(c^b)&255]^(c>>>8);return(c^0xffffffff)>>>0;};
 const packed=[],metadata=[];let total=0;
 for(const [name,data]of entries){
  const reader=new Blob([data]).stream().pipeThrough(new CompressionStream('deflate-raw')).getReader(),chunks=[];let size=0;
  try{while(true){const next=await reader.read();if(next.done)break;size+=next.value.length;if(total+size>MAX_PACKED){await reader.cancel();throw new Error('State batch ZIP compressed output exceeds 128 MiB.');}chunks.push(next.value);}}finally{reader.releaseLock();}
  const bytes=new Uint8Array(size);let cursor=0;for(const chunk of chunks){bytes.set(chunk,cursor);cursor+=chunk.length;}
  packed.push([name,bytes]);metadata.push({size:data.length,crc:crc(data)});total+=size;
 }
 // Assemble validated standard ZIP records, then set deflate's uncompressed
 // sizes and CRCs. File names, offsets and packed sizes stay as assembled.
 const zip=storedZipBytes(packed,{maxSourceBytes:MAX_PACKED}),view=new DataView(zip.buffer,zip.byteOffset,zip.byteLength);
 const end=zip.length-22;let central=view.getUint32(end+16,true);
 for(const m of metadata){
  if(view.getUint32(central,true)!==0x02014b50)throw new Error('Invalid State batch ZIP assembly.');
  const local=view.getUint32(central+42,true);view.setUint16(local+8,8,true);view.setUint32(local+14,m.crc,true);view.setUint32(local+22,m.size,true);
  view.setUint16(central+10,8,true);view.setUint32(central+16,m.crc,true);view.setUint32(central+24,m.size,true);
  central+=46+view.getUint16(central+28,true)+view.getUint16(central+30,true)+view.getUint16(central+32,true);
 }
 return zip;
}
