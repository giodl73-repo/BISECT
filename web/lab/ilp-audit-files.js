import {parseSafeJson} from './project.js';
import {encodeBytes,isBase64Encoding} from './package-files.js';
const MiB=1024*1024,record=v=>v&&typeof v==='object'&&!Array.isArray(v);
function safePath(path){return typeof path==='string'&&path.length>0&&path.length<=240&&/^[A-Za-z0-9_./-]+$/.test(path)&&!path.split('/').some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p));}
function sourceBytes(source,limit){
 if(!record(source)||Object.keys(source).sort().join()!=='base64,name'||typeof source.name!=='string'||source.name.length>240||typeof source.base64!=='string'||source.base64.length>Math.ceil(limit/3)*4||!isBase64Encoding(source.base64))throw new Error('Invalid bounded ILP source encoding.');
 const bytes=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));if(bytes.length>limit||encodeBytes(bytes)!==source.base64)throw new Error('ILP source exceeds its byte limit or has noncanonical encoding.');return bytes;
}
const text=(source,limit)=>new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(sourceBytes(source,limit));
export function validateIlpAuditInputs(v){
 if(!record(v)||Object.keys(v).sort().join()!=='models,reports,summary'||!record(v.reports)||!record(v.models)||Object.keys(v.reports).length>512||Object.keys(v.models).length>512)throw new Error('Select at most 512 reports and 512 model artifacts.');
 const paths=new Set();let reports=0,total=0;
 for(const [kind,files]of [['reports',v.reports],['models',v.models]])for(const [path,source]of Object.entries(files)){
  if(!safePath(path)||paths.has(path.toLowerCase()))throw new Error('Unsafe, duplicate or case-colliding ILP artifact path.');paths.add(path.toLowerCase());
  const bytes=sourceBytes(source,8*MiB);total+=bytes.length;if(kind==='reports')reports+=bytes.length;
 }
 if(reports>8*MiB||total>16*MiB)throw new Error('Reports are limited to 8 MiB and combined reports/models to 16 MiB.');
 if(v.summary!==null){const raw=text(v.summary,MiB);parseSafeJson(raw);}
 return v;
}
export function ilpAuditScopeSettings(settings){
 if(!record(settings)||Object.keys(settings).join()!=='mode'||!['all-selected','summary-declared'].includes(settings.mode))throw new Error('Invalid ILP audit scope setting.');return settings;
}
export function ilpAuditScopeSelection(v,settings={mode:'all-selected'}){
 validateIlpAuditInputs(v);ilpAuditScopeSettings(settings);const available=Object.keys(v.reports);
 if(settings.mode==='all-selected')return {reports:available,excluded:[]};
 if(v.summary===null)throw new Error('Select a saved summary with an explicit report list.');const summary=parseSafeJson(text(v.summary,MiB)),paths=summary.report_selection;
 if(!Array.isArray(paths)||!Number.isSafeInteger(summary.checked)||summary.checked<1||summary.checked>512||paths.length!==summary.checked||paths.some(p=>!safePath(p)||!Object.hasOwn(v.reports,p))||JSON.stringify(paths)!==JSON.stringify([...new Set(paths)].sort()))throw new Error('The summary report list must be sorted, unique, safe and fully present in selected reports.');
 return {reports:paths,excluded:available.filter(p=>!paths.includes(p))};
}
export function ilpAuditRequest(v,settings={mode:'all-selected'}){
 const selected=ilpAuditScopeSelection(v,settings);if(!selected.reports.length)throw new Error('Select at least one ILP solve report.');
 let reports,byteReports=false;try{reports=Object.fromEntries(selected.reports.map(p=>[p,text(v.reports[p],8*MiB)]));}catch{byteReports=true;reports={};}
 return {operation:'audit-ilp-artifacts',input:{reports,...(byteReports?{reports_base64:Object.fromEntries(selected.reports.map(p=>[p,v.reports[p].base64]))}:{}),models_base64:Object.fromEntries(Object.entries(v.models).map(([p,s])=>[p,s.base64])),summary_json:v.summary===null?null:text(v.summary,MiB)}};
}
function selection(files,folder){
 const entries=Array.from(files);if(!entries.length)throw new Error('Select ILP source files.');
 const prefix=folder?(entries[0].webkitRelativePath?.split('/')[0]+'/'):'';
 if(folder&&(!entries[0].webkitRelativePath||entries.some(f=>!f.webkitRelativePath?.startsWith(prefix))))throw new Error('Select one dedicated audit folder.');
 const seen=new Set();return entries.map(file=>{const path=folder?file.webkitRelativePath.slice(prefix.length):file.name;if(!safePath(path)||seen.has(path.toLowerCase()))throw new Error('Unsafe, duplicate or case-colliding ILP source path.');seen.add(path.toLowerCase());return {file,path};});
}
async function readEntries(entries,kind){
 if(entries.length>512)throw new Error('Select at most 512 files per ILP source group.');let total=0;
 for(const {file}of entries){if(!Number.isSafeInteger(file.size)||file.size<0||file.size>8*MiB)throw new Error('Each model/report file is limited to 8 MiB.');total+=file.size;}
 if(total>(kind==='reports'?8:16)*MiB)throw new Error('Selected ILP sources exceed their aggregate byte limit.');
 const out=Object.create(null);for(const {file,path}of entries){const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('ILP source size changed while reading.');out[path]={name:file.name,base64:encodeBytes(bytes)};}return out;
}
export async function readIlpAuditFiles(files,kind){if(!['reports','models'].includes(kind))throw new Error('Invalid ILP source group.');return readEntries(selection(files,false),kind);}
export async function readIlpAuditSummary(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MiB)throw new Error('Select a nonempty audit summary up to 1 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Audit summary size changed while reading.');const source={name:file.name,base64:encodeBytes(bytes)};parseSafeJson(text(source,MiB));return source;
}
export async function readIlpAuditFolder(files){
 const entries=selection(files,true),reports=[],models=[],summaries=[];let total=0;
 for(const entry of entries){const {file,path}=entry;if(!Number.isSafeInteger(file.size)||file.size<0)throw new Error('Invalid ILP file size.');total+=file.size;if(path.split('/').at(-1)==='audit-summary.json')summaries.push(entry);else if(path.endsWith('.json'))reports.push(entry);else models.push(entry);}
 if(total>17*MiB||summaries.length>1||reports.length>512||models.length>512)throw new Error('Select a bounded audit folder with at most one saved audit summary.');
 // Check every group before the first file read.
 if(reports.reduce((n,e)=>n+e.file.size,0)>8*MiB||reports.concat(models).reduce((n,e)=>n+e.file.size,0)>16*MiB||reports.concat(models).some(e=>e.file.size>8*MiB)||summaries.some(e=>e.file.size>MiB))throw new Error('Audit folder exceeds report, model or summary byte limits.');
 const v={reports:await readEntries(reports,'reports'),models:await readEntries(models,'models'),summary:summaries.length?await readIlpAuditSummary(summaries[0].file):null};return validateIlpAuditInputs(v);
}
export function ilpAuditSummaryBytes(result){
 if(result?.schema_version!=='bisect-ilp-audit-v1'||typeof result.summary_json!=='string'||new TextEncoder().encode(result.summary_json).length>16*MiB)throw new Error('Run an ILP artifact audit first.');
 const canonical=value=>JSON.stringify(value,(_key,v)=>record(v)?Object.fromEntries(Object.keys(v).sort().map(k=>[k,v[k]])):v);
 const native=parseSafeJson(result.summary_json);if(canonical(native)!==canonical(result.summary))throw new Error('Native ILP summary bytes and displayed records differ.');return new TextEncoder().encode(result.summary_json);
}
