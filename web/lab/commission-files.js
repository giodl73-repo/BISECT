import {validateSourceFile,encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
import {inspectJsonText} from './project.js?v=commission-report-v1';
const TOTAL=25*1024*1024,PER=16*1024*1024;
const names=['summary.json','contiguity.json','compactness.json','partisan.json','splits.json','vra_analysis.json','comparison.json','nesting.json'];
const maps=['districts.png','district_map.png','political.png','compactness.png'];
const safe=p=>typeof p==='string'&&p.length>0&&p.length<=240&&!/[\\:]/.test(p)&&!p.startsWith('/')&&p.split('/').every(s=>s&&!['.','..','__proto__','constructor','prototype'].includes(s));
function raw(source,{allowEmpty=false}={}){validateSourceFile(allowEmpty&&source?.base64===''?{...source,base64:'AA=='}:source,{maxBytes:PER});return Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));}
function recordedJson(source){const text=new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(raw(source));inspectJsonText(text);return JSON.parse(text);}
function allowed(manifest){const refs=[manifest?.rplan_path,manifest?.rctx_path,manifest?.audit_certificate_path].filter(p=>p!=null);if(refs.some(p=>!safe(p)))throw new Error('Report references must be safe relative package paths.');return ['manifest.json',...names.map(n=>'analysis/'+n),...maps.map(n=>'maps/'+n),...refs];}
function manifestInfo(source,opaque){try{return recordedJson(source);}catch(error){if(opaque)return null;throw error;}}
export const COMMISSION_FALLBACK_DEFAULTS={enabled:false,label:'',year:'2020'};
export function validateCommissionFallbackSettings(v){
 if(!v||Object.keys(v).sort().join()!=='enabled,label,year'||typeof v.enabled!=='boolean'||['label','year'].some(k=>typeof v[k]!=='string'||new TextEncoder().encode(v[k]).length>200))throw new Error('Invalid commission fallback settings.');return v;
}
export function commissionFallback(v){validateCommissionFallbackSettings(v);if(!v.enabled)return undefined;if(!v.label.trim()||!v.year.trim())throw new Error('Record a fallback label and year.');return {label:v.label,year:v.year};}
function validateFallback(v){if(v!==undefined&&(!v||Object.keys(v).sort().join()!=='label,year'||['label','year'].some(k=>typeof v[k]!=='string'||new TextEncoder().encode(v[k]).length>200)))throw new Error('Invalid native report fallback.');}
export function validateCommissionOpaqueSource(source){raw(source,{allowEmpty:true});return source;}
export function validateCommissionInputs(v){
 if(!v||!['files,generated_at','files,generated_at,opaque_manifest'].includes(Object.keys(v).sort().join())||(Object.hasOwn(v,'opaque_manifest')&&v.opaque_manifest!==true)||!v.files||typeof v.files!=='object'||Array.isArray(v.files)||Object.keys(v.files).length>16||typeof v.generated_at!=='string'||new TextEncoder().encode(v.generated_at).length>100)throw new Error('Invalid commission report inputs.');
 let total=0;for(const [path,source]of Object.entries(v.files)){if(!safe(path))throw new Error('Unsafe commission report path.');const bytes=raw(source,{allowEmpty:path==='manifest.json'&&v.opaque_manifest});total+=bytes.length;if(total>TOTAL)throw new Error('Commission sources exceed 25 MiB.');}
 const accept=allowed(v.files['manifest.json']?manifestInfo(v.files['manifest.json'],v.opaque_manifest):null);for(const [path,source]of Object.entries(v.files)){if(!accept.includes(path))throw new Error('Unexpected commission report file.');if(!path.startsWith('maps/')&&!(v.opaque_manifest&&!path.startsWith('analysis/')))recordedJson(source);}
 if(!v.files['manifest.json']&&Object.keys(v.files).length&&!v.opaque_manifest)throw new Error('Select a package containing manifest.json.');
 return v;
}
export function commissionRequest(v,fallback,delivery){validateCommissionInputs(v);validateFallback(fallback);if(delivery!==undefined)validateCommissionDeliverySettings(delivery);if((!v.files['manifest.json']&&!fallback)||!v.generated_at)throw new Error('Select a report package or explicit fallback and record its generation timestamp.');return {operation:'assemble-commission-report',generated_at:v.generated_at,files:Object.fromEntries(Object.entries(v.files).map(([p,s])=>[p,Array.from(raw(s,{allowEmpty:p==='manifest.json'&&v.opaque_manifest}))])),...(fallback?{fallback}:{}),...(delivery?{delivery}:{})};}
export async function readCommissionFolder(selected,{allowFallback=false}={}){
 const entries=Array.from(selected);if(!entries.length||entries.length>2000)throw new Error('Select a report folder with at most 2,000 entries.');
 const paths=entries.map(f=>f.webkitRelativePath||f.name);const nested=paths.every(p=>p.includes('/')&&p.split('/')[0]===paths[0].split('/')[0]);
 const byPath=new Map();for(let i=0;i<entries.length;i++){const p=nested?paths[i].slice(paths[i].indexOf('/')+1):paths[i];if(!safe(p)||byPath.has(p))throw new Error('Unsafe or duplicate report folder path.');byPath.set(p,entries[i]);}
 async function read(path){const f=byPath.get(path);if(!f||!Number.isSafeInteger(f.size)||f.size<(path==='manifest.json'&&allowFallback?0:1)||f.size>PER)throw new Error('Report sources must be nonempty and at most 16 MiB each.');const b=new Uint8Array(await f.arrayBuffer());if(b.length!==f.size)throw new Error('Report source size changed while reading.');return {name:f.name,base64:encodeBytes(b)};}
 const manifest=byPath.has('manifest.json')?await read('manifest.json'):allowFallback?null:await read('manifest.json'),accept=allowed(manifest?manifestInfo(manifest,allowFallback):null),chosen=accept.filter(p=>byPath.has(p));let total=0;for(const p of chosen){const f=byPath.get(p);if(!Number.isSafeInteger(f.size)||f.size<(p==='manifest.json'&&allowFallback?0:1)||f.size>PER)throw new Error('Report sources must be nonempty and at most 16 MiB each.');total+=f.size;}if(total>TOTAL)throw new Error('Commission sources exceed 25 MiB.');
 const files=Object.create(null);if(manifest)files['manifest.json']=manifest;for(const p of chosen)if(p!=='manifest.json')files[p]=await read(p);
 return validateCommissionInputs({files,generated_at:'',...(allowFallback?{opaque_manifest:true}:{})});
}
// Numeric tokens are display strings in this view; original output bytes remain untouched.
export function commissionReportView(text){
 if(typeof text!=='string'||new TextEncoder().encode(text).length>TOTAL)throw new Error('Report view exceeds its byte limit.');
 inspectJsonText(text);JSON.parse(text);
 return JSON.parse(text.replace(/"(?:\\.|[^"\\])*"|-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?/g,token=>token.startsWith('"')?token:JSON.stringify(token)));
}

export function validateCommissionAuditSource(source){recordedJson(source);return source;}
export function commissionAuditRequest(source,fallback,{opaque=false}={}){validateFallback(fallback);if(!source){if(!fallback)throw new Error('Select a manifest or enable explicit fallback.');return {operation:'assemble-commission-audit',fallback};}if(opaque)validateCommissionOpaqueSource(source);else validateCommissionAuditSource(source);return {operation:'assemble-commission-audit',...(opaque?{manifest_bytes:Array.from(raw(source,{allowEmpty:true}))}:{manifest_json:new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(raw(source))}),...(fallback?{fallback}:{})};}
export async function readCommissionAuditSource(file,{allowOpaque=false}={}){
 if(!file||!Number.isSafeInteger(file.size)||file.size<(allowOpaque?0:1)||file.size>PER)throw new Error('Commission manifest must be nonempty and at most 16 MiB.');
 const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Commission manifest size changed while reading.');
 return (allowOpaque?validateCommissionOpaqueSource:validateCommissionAuditSource)({name:file.name,base64:encodeBytes(bytes)});
}

export const COMMISSION_DELIVERY_DEFAULTS={label:'commission-report',formats:['html','json'],audit_with_report:true};
export function validateCommissionDeliverySettings(v,{requireComplete=true}={}){
 if(!v||Object.keys(v).sort().join()!=='audit_with_report,formats,label'||typeof v.label!=='string'||new TextEncoder().encode(v.label).length>100||(requireComplete&&!/^[A-Za-z0-9_-]{1,100}$/.test(v.label))||typeof v.audit_with_report!=='boolean'||!Array.isArray(v.formats)||(requireComplete&&!v.formats.length)||v.formats.length>8||v.formats.some(f=>!['html','json'].includes(f)))throw new Error('Choose an output label and HTML/JSON formats.');return v;
}
export function commissionFormatAvailable(result,kind){
 if(result?.schema_version!=='bisect-commission-report-v1')return false;
 if(result.delivery_settings){try{validateCommissionDeliverySettings(result.delivery_settings);}catch{return false;}if(kind==='audit'?!result.delivery_settings.audit_with_report:!result.delivery_settings.formats.includes(kind))return false;}
 return typeof result[kind==='audit'?'audit_json':kind==='html'?'report_html':'report_json']==='string';
}
export function commissionReportFiles(result){
 let files;
 if(result?.schema_version==='bisect-commission-audit-v1'&&typeof result.audit_json==='string')files={'audit.json':result.audit_json};
 else {
  if(result?.schema_version!=='bisect-commission-report-v1')throw new Error('Run report delivery before downloading.');
  const settings=validateCommissionDeliverySettings(result.delivery_settings),expected={};
  for(const format of new Set(settings.formats)){if(!commissionFormatAvailable(result,format))throw new Error('Selected report format is missing.');expected[settings.label+'_report.'+format]=result[format==='html'?'report_html':'report_json'];}
  if(settings.audit_with_report){if(!commissionFormatAvailable(result,'audit'))throw new Error('Companion audit is missing.');expected['audit.json']=result.audit_json;}
  if(!result.report_files||Object.keys(result.report_files).sort().join()!==Object.keys(expected).sort().join()||Object.entries(expected).some(([p,text])=>result.report_files[p]!==text))throw new Error('Report filenames or bytes differ from the selected native delivery.');files=expected;
 }
 return files;
}

export function commissionReportZip(result){return storedZipBytes(Object.entries(commissionReportFiles(result)).map(([p,text])=>[p,new TextEncoder().encode(text)]),{maxSourceBytes:64*1024*1024});}
