// ZIP bytes stay in memory: paths are identifiers, never filesystem destinations.
import {encodeBytes,validatePackagePath} from './package-files.js';
export const MAX_CENSUS_ZIP_BYTES=64*1024*1024;
export function censusArchiveLimit(settings){if(settings?.archive_profile!==undefined&&settings.archive_profile!=='large-state')throw new Error('Invalid Census archive profile.');return settings?.kind==='pl94'&&settings?.archive_profile==='large-state'?128*1024*1024:settings?.kind==='tiger'&&settings?.unit_kind==='counties'?96*1024*1024:MAX_CENSUS_ZIP_BYTES;}
const MAX_MEMBER=128*1024*1024,MAX_TOTAL=256*1024*1024,MAX_SOURCE=16*1024*1024;
const crcTable=Uint32Array.from({length:256},(_,n)=>{for(let i=0;i<8;i++)n=n&1?0xedb88320^(n>>>1):n>>>1;return n>>>0;});
function crc32(bytes){let n=0xffffffff;for(const b of bytes)n=crcTable[(n^b)&255]^(n>>>8);return (n^0xffffffff)>>>0;}
async function transform(bytes,format,compress=false,limit=MAX_MEMBER){
 const codec=compress?new CompressionStream(format):new DecompressionStream(format);
 const reader=new Blob([bytes]).stream().pipeThrough(codec).getReader();let total=0;const chunks=[];
 try{for(;;){const {done,value}=await reader.read();if(done)break;total+=value.length;if(total>limit)throw new Error('Census archive expansion exceeds its limit.');chunks.push(value);}}
 catch(error){await reader.cancel().catch(()=>{});throw error;}
 const out=new Uint8Array(total);let offset=0;for(const chunk of chunks){out.set(chunk,offset);offset+=chunk.length;}return out;
}
export async function extractCensusZip(bytes,{kind,year=2020,unit_kind='tracts',archive_profile}={}){
 if(!(bytes instanceof Uint8Array)||bytes.length<22||bytes.length>censusArchiveLimit({kind,unit_kind,archive_profile})||!['tiger','pl94'].includes(kind)||![2000,2010,2020].includes(year))throw new Error('Select a supported Census ZIP within the selected source size limit.');
 const memberLimit=kind==='pl94'?512*1024*1024:MAX_MEMBER,totalLimit=kind==='pl94'?1024*1024*1024:MAX_TOTAL;
 const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength),u16=p=>view.getUint16(p,true),u32=p=>view.getUint32(p,true);
 let end=-1;for(let p=bytes.length-22;p>=Math.max(0,bytes.length-65557);p--)if(u32(p)===0x06054b50&&p+22+u16(p+20)===bytes.length){end=p;break;}
 if(end<0)throw new Error('ZIP end record is missing or truncated.');
 const count=u16(end+10),central=u32(end+16),centralSize=u32(end+12);
 if(u16(end+4)||u16(end+6)||u16(end+8)!==count||count===65535||!count||count>2000||central+centralSize!==end)throw new Error('Unsupported split, ZIP64 or oversized ZIP directory.');
 const decoder=new TextDecoder('utf-8',{fatal:true}),names=new Set(),ranges=[],entries=[];let offset=central,total=0;
 for(let i=0;i<count;i++){
  if(offset+46>end||u32(offset)!==0x02014b50)throw new Error('Invalid ZIP directory entry.');
  const flags=u16(offset+8),method=u16(offset+10),crc=u32(offset+16),packed=u32(offset+20),size=u32(offset+24),nl=u16(offset+28),xl=u16(offset+30),cl=u16(offset+32),local=u32(offset+42),attrs=u32(offset+38);
  if(offset+46+nl+xl+cl>end||flags&~(0x800|8|6)||![0,8].includes(method)||u16(offset+34)||size===0xffffffff||packed===0xffffffff||local===0xffffffff)throw new Error('Unsupported encrypted, ZIP64 or compressed ZIP entry.');
  const name=decoder.decode(bytes.subarray(offset+46,offset+46+nl)),directory=name.endsWith('/');validatePackagePath(directory?name.slice(0,-1):name);
  const folded=name.toLowerCase();if(names.has(folded))throw new Error('Duplicate ZIP path.');names.add(folded);
  const unixType=(attrs>>>16)&0xf000;if(unixType&&unixType!==(directory?0x4000:0x8000))throw new Error('ZIP links and special files are unsupported.');
  total+=size;if(size>memberLimit||total>totalLimit)throw new Error('Census archive expansion exceeds its limit.');
  if(local+30>central||u32(local)!==0x04034b50||u16(local+6)!==flags||u16(local+8)!==method)throw new Error('ZIP local header disagrees with directory.');
  const lnl=u16(local+26),lxl=u16(local+28),start=local+30+lnl+lxl;
  if(start+packed>central||decoder.decode(bytes.subarray(local+30,local+30+lnl))!==name)throw new Error('ZIP local name or data extent disagrees.');
  if(!(flags&8)&&(u32(local+14)!==crc||u32(local+18)!==packed||u32(local+22)!==size))throw new Error('ZIP local sizes or checksum disagree.');
  ranges.push([local,start+packed]);entries.push({name,method,crc,packed,size,start,directory});offset+=46+nl+xl+cl;
 }
 if(offset!==end)throw new Error('ZIP directory size disagrees.');
 ranges.sort((a,b)=>a[0]-b[0]);for(let i=1;i<ranges.length;i++)if(ranges[i][0]<ranges[i-1][1])throw new Error('Overlapping ZIP members.');
 const chosen=new Map();let stem=null;
 for(const entry of entries){if(entry.directory)continue;const base=entry.name.split('/').at(-1),lower=base.toLowerCase();let role;
  if(kind==='tiger'){role=/\.(shp|dbf|shx|prj)$/.exec(lower)?.[1];if(role){const s=lower.slice(0,-4);if(stem!==null&&s!==stem)throw new Error('Select a ZIP with one matching TIGER dataset.');stem=s;}}
  else if(new RegExp('geo'+year+'\\.pl$','i').test(base)||year===2000&&/geo\.(uf1|upl)$/i.test(base))role='geography';
  else if(new RegExp('00001'+year+'\\.pl$','i').test(base))role='population';
  if(role){if(chosen.has(role))throw new Error('Ambiguous Census ZIP source.');chosen.set(role,entry);}
 }
 const required=kind==='tiger'?['shp','dbf']:year===2000?['geography']:['geography','population'];for(const role of required)if(!chosen.has(role))throw new Error('Census ZIP is missing '+role+'.');
 const sources={},members=[];
 for(const [role,entry] of chosen){let data=bytes.subarray(entry.start,entry.start+entry.packed);if(entry.method===8)data=await transform(data,'deflate-raw',false,entry.size);else data=new Uint8Array(data);
  if(data.length!==entry.size||crc32(data)!==entry.crc)throw new Error('ZIP member size or CRC mismatch.');
  let name=entry.name.split('/').at(-1);const sourceLimit=kind==='tiger'&&role!=='prj'?censusArchiveLimit({kind,unit_kind,archive_profile}):kind==='pl94'?64*1024*1024:MAX_SOURCE;if(kind==='pl94'||kind==='tiger'&&role!=='prj'&&data.length>MAX_SOURCE){data=await transform(data,'gzip',true,sourceLimit);name+='.gz';}
  if(data.length>sourceLimit||role==='prj'&&data.length>65536)throw new Error('Selected Census part exceeds its source limit.');
  sources[role]={name,base64:encodeBytes(data)};members.push({path:entry.name,decoded_bytes:entry.size,crc32:entry.crc});
 }
 return {kind,year,sources,members,archive_sha256:Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join(''),execution_performed:false};
}
export function readCensusArchive(file,settings,{signal}={}){
 if(!file||!Number.isSafeInteger(file.size)||file.size<22||file.size>censusArchiveLimit(settings))return Promise.reject(new Error('Select a Census ZIP within the selected source size limit.'));
 return new Promise((resolve,reject)=>{if(signal?.aborted){reject(new Error('Archive import cancelled.'));return;}const worker=new Worker(new URL('./census-archive-worker.js',import.meta.url),{type:'module'});let settled=false;
  const finish=(error,result)=>{if(settled)return;settled=true;clearTimeout(timer);signal?.removeEventListener('abort',abort);worker.terminate();error?reject(error):resolve(result);},abort=()=>finish(new Error('Archive import cancelled.')),timer=setTimeout(()=>finish(new Error('Archive import timed out.')),60000);
  signal?.addEventListener('abort',abort,{once:true});worker.onmessage=({data})=>data?.ok?finish(null,data.result):finish(new Error(data?.error||'Archive import failed.'));worker.onerror=worker.onmessageerror=()=>finish(new Error('Archive import failed.'));worker.postMessage({file,settings});
 });
}
