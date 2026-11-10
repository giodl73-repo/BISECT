import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
const LIMIT=3*1024*1024;
const record=v=>v&&typeof v==='object'&&!Array.isArray(v);
const analysisName=n=>typeof n==='string'&&n.length<=120&&/^[A-Za-z0-9_][A-Za-z0-9_.-]*\.json$/.test(n);
export function validateReproducibilityInputs(value){
 if(!record(value)||Object.keys(value).length!==3||!['label','manifest','analysis'].every(k=>Object.hasOwn(value,k)))throw new Error('Invalid reproducibility inputs.');
 if(typeof value.label!=='string'||! /^[A-Za-z0-9_-]{1,120}$/.test(value.label))throw new Error('Package label needs 1–120 letters, digits, underscores or hyphens.');
 if(!record(value.analysis)||Object.keys(value.analysis).length>64)throw new Error('Select at most 64 analysis JSON files.');
 let bytes=0;for(const [name,source] of Object.entries(value.analysis)){if(!analysisName(name))throw new Error('Analysis needs a plain JSON basename.');validateSourceFile(source);bytes+=new TextEncoder().encode(exactSourceText(source)).length+name.length;}
 if(value.manifest!==null){validateSourceFile(value.manifest);bytes+=new TextEncoder().encode(exactSourceText(value.manifest)).length;}
 if(bytes>LIMIT)throw new Error('Reproducibility inputs exceed 3 MiB.');return value;
}
export function reproducibilityRequest(files,inputs){
 validateReproducibilityInputs(inputs);if(!files.auditPlanText)throw new Error('Select an exact RPLAN file.');
 const document_json=exactSourceText(files.auditPlanText),manifest_json=inputs.manifest?exactSourceText(inputs.manifest):null;
 const analysis_files=Object.fromEntries(Object.entries(inputs.analysis).map(([name,source])=>[name,exactSourceText(source)]));
 const bytes=new TextEncoder().encode(document_json+(manifest_json||'')).length+Object.entries(analysis_files).reduce((n,[k,v])=>n+k.length+new TextEncoder().encode(v).length,0);
 if(bytes>LIMIT)throw new Error('Plan, manifest and analysis must total at most 3 MiB.');
 return {operation:'export-reproducibility-package',document_json,manifest_json,analysis_files,label:inputs.label};
}
export async function readReproducibilityAnalysis(selected){
 const list=Array.from(selected);if(list.length>64)throw new Error('Select at most 64 analysis JSON files.');
 let total=0;const names=new Set();for(const f of list){if(!analysisName(f.name)||names.has(f.name))throw new Error('Analysis file names must be unique plain JSON basenames.');names.add(f.name);if(!Number.isSafeInteger(f.size)||f.size<0)throw new Error('Invalid file size.');total+=f.size+f.name.length;}
 if(total>LIMIT)throw new Error('Analysis input exceeds 3 MiB.');
 const sources=Object.create(null);for(const f of list){const source=await readElectionSource(f);if(atob(source.base64).length!==f.size)throw new Error('Analysis file size changed while reading.');sources[f.name]=source;}return sources;
}
function generatedFiles(result){
 if(result?.schema_version!=='bisect-reproducibility-package-v1'||!record(result.files))throw new Error('Run native package export first.');
 const entries=Object.entries(result.files);if(entries.length<3||entries.length>68)throw new Error('Invalid package output count.');
 let bytes=0;for(const [name,text] of entries){if(typeof text!=='string'||!(name==='manifest.json'||name==='audit.json'||name==='verify.sh'||/^[A-Za-z0-9_-]{1,120}\.rplan$/.test(name)||name.startsWith('analysis/')&&analysisName(name.slice(9))))throw new Error('Invalid package output path.');bytes+=new TextEncoder().encode(text).length;}
 if(bytes>25*1024*1024)throw new Error('Package output exceeds 25 MiB.');return entries;
}
export function reproducibilityOutputNames(result){return generatedFiles(result).map(([name])=>name);}
// Portable ZIP32 with stored UTF-8 entries, fixed timestamps and CRC-32. No scripts execute.
export function reproducibilityZip(result){
 return storedZipBytes(generatedFiles(result).map(([name,text])=>[name,new TextEncoder().encode(text)]),{executablePaths:['verify.sh']});
}
export function storedZipBytes(entries,{executablePaths=[],maxSourceBytes=25*1024*1024,maxPathLength=240,maxEntries=2000}={}){
 if(!Number.isSafeInteger(maxPathLength)||maxPathLength<1||maxPathLength>512)throw new Error('Invalid ZIP path length limit.');
 if(!Number.isSafeInteger(maxSourceBytes)||maxSourceBytes<1||maxSourceBytes>256*1024*1024)throw new Error('Invalid ZIP source size limit.');
 if(!Number.isSafeInteger(maxEntries)||maxEntries<1||maxEntries>20000)throw new Error('Invalid ZIP entry-count limit.');
 if(!Array.isArray(entries)||!entries.length||entries.length>maxEntries)throw new Error('Invalid ZIP entry count.');
 const paths=new Set();let total=0;
 for(const [name,data]of entries){const directory=typeof name==='string'&&name.endsWith('/');const path=directory?name.slice(0,-1):name;if(typeof name!=='string'||name.length>maxPathLength||!/^[A-Za-z0-9_./ -]+$/.test(path)||path.split('/').some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p))||paths.has(name)||!(data instanceof Uint8Array)||(directory&&data.length))throw new Error('Invalid ZIP entry.');paths.add(name);total+=data.length;}
 if(total>maxSourceBytes)throw new Error('ZIP source exceeds its configured size limit.');
 const parts=[],central=[];let offset=0,centralSize=0;
 const block=(length,fields)=>{const b=new Uint8Array(length),v=new DataView(b.buffer);for(const [at,size,value] of fields)size===2?v.setUint16(at,value,true):v.setUint32(at,value,true);return b;};
 const crc=data=>{let c=0xffffffff;for(const byte of data){c^=byte;for(let i=0;i<8;i++)c=(c>>>1)^((c&1)?0xedb88320:0);}return(c^0xffffffff)>>>0;};
 for(const [name,data] of entries){const path=new TextEncoder().encode(name),checksum=crc(data);
  const local=block(30,[[0,4,0x04034b50],[4,2,20],[6,2,0x800],[12,2,33],[14,4,checksum],[18,4,data.length],[22,4,data.length],[26,2,path.length]]);
  const directory=block(46,[[0,4,0x02014b50],[4,2,788],[6,2,20],[8,2,0x800],[14,2,33],[16,4,checksum],[20,4,data.length],[24,4,data.length],[28,2,path.length],[38,4,name.endsWith('/')?((0o40755<<16)|0x10)>>>0:((executablePaths.includes(name)?0o100755:0o100644)<<16)>>>0],[42,4,offset]]);
  parts.push(local,path,data);central.push(directory,path);offset+=local.length+path.length+data.length;centralSize+=directory.length+path.length;
 }
 const end=block(22,[[0,4,0x06054b50],[8,2,entries.length],[10,2,entries.length],[12,4,centralSize],[16,4,offset]]);
 const out=new Uint8Array(offset+centralSize+end.length);let cursor=0;for(const part of [...parts,...central,end]){out.set(part,cursor);cursor+=part.length;}return out;
}
