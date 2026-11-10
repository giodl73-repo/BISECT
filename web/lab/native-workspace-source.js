import {selectedFolderEntries,portableRelativePath} from './run-target-files.js';
import {encodeBytes} from './package-files.js';

const sources=new WeakMap(),MAX_BYTES=64*1024*1024,MAX_ENTRIES=10000;
const hash=async bytes=>Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
const check=signal=>signal?.throwIfAborted();
function pathOK(path){
 if(!portableRelativePath(path)||path.split('/').length>32||path.split('/').some(p=>['__proto__','constructor','prototype'].includes(p)||p.endsWith('.')||/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(p)))throw new Error('Unsafe native workspace path.');
 return path;
}
function pathAliases(path,seen){const parts=path.split('/');for(let n=1;n<=parts.length;n++){const prefix=parts.slice(0,n).join('/'),key=prefix.toLowerCase(),previous=seen.get(key);if(previous!==undefined&&previous!==prefix)throw new Error('Case-colliding native workspace path components.');seen.set(key,prefix);}}
function validateFiles(files){
 const {root,entries}=selectedFolderEntries(files),seen=new Set(),aliases=new Map(),filePaths=new Set(entries.map(e=>e.path.toLowerCase()));
 for(const {path,file}of entries){pathOK(path);pathAliases(path,aliases);const key=path.toLowerCase();if(seen.has(key))throw new Error('Case-colliding native workspace paths.');seen.add(key);if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid native workspace file size.');const parts=key.split('/');for(let n=1;n<parts.length;n++)if(filePaths.has(parts.slice(0,n).join('/')))throw new Error('Native workspace file/directory collision.');}
 return {root,entries};
}
function register(kind,files,extra={}){
 const {root}=validateFiles(files),selection=Object.freeze({kind,root,display_name:extra.directory?.name??extra.archiveSource?.name??root,file_count:files.length,...(extra.archiveSource?{archive_source:extra.archiveSource,archive_sha256:extra.archiveSha}: {})});
 sources.set(selection,{files,...extra});return selection;
}
export function selectNativeWorkspaceFiles(files){return register('uploaded-folder',[...files]);}
export function retireNativeWorkspaceSource(selection){if(selection)sources.delete(selection);}
async function permission(root){if(typeof root.queryPermission==='function'&&await root.queryPermission({mode:'read'})!=='granted')throw new Error('Input folder permission is no longer granted. Select the folder again.');}
async function directoryFiles(root,{signal}={}){
 if(root?.kind!=='directory')throw new Error('Select a local input directory.');check(signal);await permission(root);check(signal);
 const files=[];let count=0;
 async function visit(dir,prefix='',depth=0){
  if(depth>32)throw new Error('Native workspace directory exceeds 32 levels.');
  for await(const [name,entry]of dir.entries()){
   check(signal);if(++count>MAX_ENTRIES)throw new Error('Native workspace exceeds 10,000 entries.');const path=pathOK(prefix+name);
   if(entry.kind==='directory')await visit(entry,path+'/',depth+1);
   else if(entry.kind==='file'){
    const file=await entry.getFile();check(signal);const size=file.size,modified=file.lastModified;
    files.push({name,webkitRelativePath:'workspace/'+path,size,lastModified:modified,async arrayBuffer(){check(signal);await permission(root);const current=await entry.getFile();check(signal);if(current.size!==size||current.lastModified!==modified)throw new Error('Native workspace source changed. Select and inspect the folder again.');const bytes=await current.arrayBuffer();check(signal);if(bytes.byteLength!==size)throw new Error('Native workspace source changed during reading.');return bytes;}});
   }else throw new Error('Unsupported native workspace entry.');
  }
 }
 await visit(root);files.sort((a,b)=>a.webkitRelativePath.localeCompare(b.webkitRelativePath,'en'));validateFiles(files);return files;
}
const fingerprint=files=>JSON.stringify(files.map(f=>[f.webkitRelativePath,f.size,f.lastModified??null]));
export async function selectNativeWorkspaceDirectory(root,options){const files=await directoryFiles(root,options);return register('live-directory',files,{directory:root,fingerprint:fingerprint(files)});}
export async function nativeWorkspaceFiles(selection,{signal}={}){
 check(signal);const state=sources.get(selection);if(!state)throw new Error('Select and inspect this native workspace again. Imported metadata grants no folder authority.');
 if(state.directory){const files=await directoryFiles(state.directory,{signal});if(fingerprint(files)!==state.fingerprint)throw new Error('Native workspace source changed. Select and inspect the folder again.');return files;}
 return state.files.slice();
}

const crcTable=Uint32Array.from({length:256},(_,n)=>{for(let i=0;i<8;i++)n=n&1?0xedb88320^(n>>>1):n>>>1;return n>>>0;});
function crc32(bytes){let crc=0xffffffff;for(const b of bytes)crc=crcTable[(crc^b)&255]^(crc>>>8);return (crc^0xffffffff)>>>0;}
async function inflate(bytes,size,signal){
 const reader=new Blob([bytes]).stream().pipeThrough(new DecompressionStream('deflate-raw')).getReader(),chunks=[];let total=0;
 try{for(;;){check(signal);const {done,value}=await reader.read();check(signal);if(done)break;total+=value.length;if(total>size)throw new Error('Native workspace ZIP expansion exceeds declared size.');chunks.push(value);}}catch(error){await reader.cancel().catch(()=>{});throw error;}
 const result=new Uint8Array(total);let offset=0;for(const part of chunks){result.set(part,offset);offset+=part.length;}return result;
}
// Only the explicit import action reads this bounded archive. Members stay in
// memory, never become extraction destinations, and are never executed.
export async function decodeNativeWorkspaceArchive(bytes,{signal,layout='native-inputs'}={}){
 if(!['native-inputs','output-package','fetch-source'].includes(layout))throw new Error('Choose a supported native archive layout.');
 const expandedLimit=layout==='fetch-source'?128*1024*1024:MAX_BYTES;
 check(signal);if(!(bytes instanceof Uint8Array)||bytes.length<22||bytes.length>MAX_BYTES)throw new Error('Select a native workspace ZIP up to 64 MiB.');
 bytes=bytes.slice(); // Hold one private input snapshot across async inflation.
 const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),u16=p=>view.getUint16(p,true),u32=p=>view.getUint32(p,true),decoder=new TextDecoder('utf-8',{fatal:true});
 let end=-1;for(let p=bytes.length-22;p>=Math.max(0,bytes.length-65557);p--)if(u32(p)===0x06054b50&&p+22+u16(p+20)===bytes.length){end=p;break;}
 if(end<0)throw new Error('Invalid native workspace ZIP end record.');
 const count=u16(end+10),central=u32(end+16);if(u16(end+4)||u16(end+6)||u16(end+8)!==count||!count||count>MAX_ENTRIES||central+u32(end+12)!==end)throw new Error('Unsupported split, ZIP64 or oversized native workspace ZIP.');
 let offset=central,total=0;const entries=[],ranges=[],names=new Set(),aliases=new Map();
 for(let i=0;i<count;i++){
  check(signal);if(offset+46>end||u32(offset)!==0x02014b50)throw new Error('Invalid native workspace ZIP directory.');
  const flags=u16(offset+8),method=u16(offset+10),crc=u32(offset+16),packed=u32(offset+20),size=u32(offset+24),nl=u16(offset+28),xl=u16(offset+30),cl=u16(offset+32),local=u32(offset+42),type=(u32(offset+38)>>>16)&0xf000;
  if(offset+46+nl+xl+cl>end||flags&~(0x800|8|6)||![0,8].includes(method)||u16(offset+34)||[size,packed,local].includes(0xffffffff))throw new Error('Unsupported encrypted, ZIP64 or compressed native workspace ZIP.');
  const name=decoder.decode(bytes.subarray(offset+46,offset+46+nl)),directory=name.endsWith('/'),path=pathOK(directory?name.slice(0,-1):name),folded=path.toLowerCase();
  pathAliases(path,aliases);
  if(names.has(folded))throw new Error('Duplicate or case-colliding native workspace ZIP path.');names.add(folded);
  if(type&&type!==(directory?0x4000:0x8000)||directory&&(packed||size))throw new Error('Native workspace ZIP links/special files are unsupported.');
  total+=size;if(size>expandedLimit||total>expandedLimit)throw new Error('Native workspace ZIP expanded data exceeds '+(expandedLimit/1024/1024)+' MiB.');
  if(local+30>central||u32(local)!==0x04034b50||u16(local+6)!==flags||u16(local+8)!==method)throw new Error('Native workspace ZIP local header disagrees.');
  const lnl=u16(local+26),lxl=u16(local+28),start=local+30+lnl+lxl;
  if(start+packed>central||decoder.decode(bytes.subarray(local+30,local+30+lnl))!==name)throw new Error('Native workspace ZIP local name/extent disagrees.');
  if(!(flags&8)&&(u32(local+14)!==crc||u32(local+18)!==packed||u32(local+22)!==size))throw new Error('Native workspace ZIP local sizes/CRC disagree.');
  let extent=start+packed;
  if(flags&8){const signature=extent+4<=central&&u32(extent)===0x08074b50?4:0,p=extent+signature;if(p+12>central||u32(p)!==crc||u32(p+4)!==packed||u32(p+8)!==size)throw new Error('Native workspace ZIP data descriptor disagrees.');extent=p+12;}
  ranges.push([local,extent]);entries.push({path,directory,method,crc,packed,size,start});offset+=46+nl+xl+cl;
 }
 if(offset!==end)throw new Error('Native workspace ZIP directory size disagrees.');ranges.sort((a,b)=>a[0]-b[0]);let cursor=0;for(const [start,finish]of ranges){if(start!==cursor)throw new Error('Overlapping or hidden native workspace ZIP bytes.');cursor=finish;}if(cursor!==central)throw new Error('Hidden native workspace ZIP bytes.');
 const filePaths=new Set(entries.filter(e=>!e.directory).map(e=>e.path.toLowerCase()));
 for(const e of entries){const parts=e.path.toLowerCase().split('/');for(let n=1;n<parts.length;n++)if(filePaths.has(parts.slice(0,n).join('/')))throw new Error('Native workspace ZIP file/directory collision.');}
 const files=[];
 for(const entry of entries){check(signal);if(entry.directory)continue;let data=bytes.subarray(entry.start,entry.start+entry.packed);data=entry.method===8?await inflate(data,entry.size,signal):data.slice();if(data.length!==entry.size||crc32(data)!==entry.crc)throw new Error('Native workspace ZIP member size/CRC mismatch.');files.push({path:entry.path,bytes:data});}
 if(!files.length)throw new Error('Native workspace ZIP contains no files.');
 if(layout==='output-package'||layout==='fetch-source')return {files,archive_sha256:await hash(bytes),original_bytes:bytes};
 // Accept either V3/V4 at archive root, or one enclosing folder. Never search
 // arbitrary nested roots or change the native V3/V4 precedence policy.
 const bare=files.some(f=>/^(V3|V4)\//.test(f.path));
 if(!bare){const roots=new Set(files.map(f=>f.path.split('/')[0]));if(roots.size!==1||files.some(f=>!f.path.includes('/')))throw new Error('ZIP must contain V3/V4 or one enclosing native output root.');const prefix=[...roots][0]+'/';for(const file of files)file.path=file.path.slice(prefix.length);}
 if(!files.some(f=>/^(V3|V4)\//.test(f.path)))throw new Error('ZIP has no native V3/V4 input store.');
 return {files,archive_sha256:await hash(bytes),original_bytes:bytes};
}
export async function selectNativeWorkspaceArchive(bytes,name='native-inputs.zip',options){
 const decoded=await decodeNativeWorkspaceArchive(bytes,options);return archiveSelection(decoded,decoded.original_bytes,name);
}
function archiveSelection(decoded,bytes,name){
 const files=decoded.files.map(({path,bytes})=>({name:path.split('/').at(-1),webkitRelativePath:'workspace/'+path,size:bytes.length,async arrayBuffer(){return bytes.slice().buffer;}}));
 return register('zip',files,{archiveSha:decoded.archive_sha256,archiveSource:Object.freeze({name,base64:encodeBytes(bytes)})});
}
export function readNativeWorkspaceArchive(file,{signal}={}){
 if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>MAX_BYTES)return Promise.reject(new Error('Select a native workspace ZIP up to 64 MiB.'));
 return new Promise((resolve,reject)=>{
  check(signal);const worker=new Worker(new URL('./native-workspace-archive-worker.js',import.meta.url),{type:'module'});let finished=false;
  const finish=(error,value)=>{if(finished)return;finished=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(value);},abort=()=>finish(signal.reason??new Error('Native workspace ZIP import cancelled.')),timer=setTimeout(()=>finish(new Error('Native workspace ZIP import timed out.')),60000);
  signal?.addEventListener('abort',abort,{once:true});worker.onerror=worker.onmessageerror=()=>finish(new Error('Native workspace ZIP import failed.'));
  worker.onmessage=({data})=>{try{if(!data?.ok)throw new Error(data?.error||'Native workspace ZIP import failed.');check(signal);finish(null,archiveSelection(data.result,new Uint8Array(data.original),file.name));}catch(error){finish(error);}};
  try{worker.postMessage({file});}catch(error){finish(error);}
 });
}
