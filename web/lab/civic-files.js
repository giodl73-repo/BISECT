import {validateCivicSnapshots} from './civic-snapshot-files.js';
import {validateSourceFile,readElectionSource,exactSourceText} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
export const CIVIC_DEFAULTS={state:'RI',year:'2020',validate_mode:'strict',sources:[]};
export const CIVIC_INGEST_DEFAULTS={label:'community_submission',submitter:'',timestamp:''};
const bytes=s=>new TextEncoder().encode(s).length;
export function validateCivicInputs(value){
 if(!value||Array.isArray(value)||Object.keys(value).sort().join()!=='sources,state,validate_mode,year'||typeof value.state!=='string'||!value.state||bytes(value.state)>100||typeof value.year!=='string'||!value.year||bytes(value.year)>20||!['strict','lenient','advisory'].includes(value.validate_mode)||!Array.isArray(value.sources)||value.sources.length>20)throw new Error('Invalid civic input settings.');
 let total=0;const names=new Set();for(const source of value.sources){validateSourceFile(source,{maxBytes:3*1024*1024});if(names.has(source.name)||bytes(source.name)>100)throw new Error('Select unique civic filenames of at most 100 UTF-8 bytes.');names.add(source.name);total+=atob(source.base64).length;}if(total>3*1024*1024)throw new Error('Civic sources exceed 3 MiB.');return value;
}
export function civicRequest(value){validateCivicInputs(value);if(!value.sources.length)throw new Error('Select civic CSV submissions.');return {operation:'inspect-civic-coi',state:value.state,year:value.year,validate_mode:value.validate_mode,inputs:value.sources.map(s=>({label:s.name,source_name:s.name,csv_base64:s.base64}))};}
export async function readCivicSources(selected){
 const list=Array.from(selected);if(list.length>20)throw new Error('Select at most 20 civic CSV files.');let total=0;const names=new Set();for(const f of list){if(typeof f.name!=='string'||bytes(f.name)>100||names.has(f.name)||!Number.isSafeInteger(f.size)||f.size<1)throw new Error('Select uniquely named nonempty civic CSV files.');names.add(f.name);total+=f.size;}if(total>3*1024*1024)throw new Error('Civic sources exceed 3 MiB.');
 const sources=[];for(const f of list){const s=await readElectionSource(f);if(atob(s.base64).length!==f.size)throw new Error('Civic file size changed while reading.');sources.push(s);}validateCivicInputs({...CIVIC_DEFAULTS,sources});return sources;
}
export function civicZip(result){
 if(result?.schema_version!=='bisect-civic-assessment-v1')throw new Error('Run civic assessment first.');const files=Object.create(null);for(const [i,input]of(result.inputs||[]).entries()){if(typeof input.normalized_csv==='string')files[`submission-${i+1}/normalized.csv`]=input.normalized_csv;}files['assessment.json']=JSON.stringify(result,null,2);if(typeof result.conflicts_json==='string')files['conflicts.json']=result.conflicts_json;return storedZipBytes(Object.entries(files).map(([n,t])=>[n,new TextEncoder().encode(t)]));
}
export function validateCivicIngestSettings(value){
 if(!value||Array.isArray(value)||Object.keys(value).filter(k=>k!=='snapshots').sort().join()!=='label,submitter,timestamp'||typeof value.label!=='string'||!/^[A-Za-z0-9_-]{1,100}$/.test(value.label)||['__proto__','constructor','prototype'].includes(value.label)||typeof value.submitter!=='string'||bytes(value.submitter)>200||typeof value.timestamp!=='string'||bytes(value.timestamp)>100)throw new Error('Invalid civic ingestion label, submitter or timestamp.');if(Object.hasOwn(value,'snapshots'))validateCivicSnapshots(value.snapshots);return value;
}
export function civicIngestRequest(inputs,settings){
 const assessment=civicRequest(inputs);validateCivicIngestSettings(settings);if(assessment.inputs.length!==1)throw new Error('Select exactly one civic CSV for ingestion.');if(!settings.timestamp)throw new Error('Supply the civic ingestion timestamp label.');return {operation:'ingest-civic-coi',input:{...assessment.inputs[0],label:settings.label},state:inputs.state,year:inputs.year,validate_mode:inputs.validate_mode,settings};
}
export function civicIngestZip(result){
 if(result?.schema_version!=='bisect-civic-ingest-v1')throw new Error('Run civic ingestion first.');const label=result.manifest?.label;validateCivicIngestSettings({label,submitter:'',timestamp:''});const prefix=`v1/civic_inputs/${label}/`,entries=Object.entries(result.files_base64||{});const expected=['original.csv','normalized.csv','validation_log.txt','manifest.json'].map(n=>prefix+n);
 if(entries.length<4||entries.length>44||expected.some(n=>!Object.hasOwn(result.files_base64,n))||entries.some(([n,s])=>(!expected.includes(n)&&!(n.startsWith(prefix+'snapshots/')&&/^[a-f0-9]{8}\.(body|headers\.txt)$/.test(n.slice((prefix+'snapshots/').length))))||typeof s!=='string'))throw new Error('Invalid civic package files.');return storedZipBytes(entries.map(([n,s])=>[n,Uint8Array.from(atob(s),c=>c.charCodeAt(0))]));
}
export function validateCivicManifestSources(value){
 if(!value||Array.isArray(value)||Object.keys(value).join()!=='sources'||!Array.isArray(value.sources)||value.sources.length>20)throw new Error('Invalid civic manifest sources.');let total=0;const names=new Set();for(const s of value.sources){validateSourceFile(s,{maxBytes:1024*1024});exactSourceText(s);if(names.has(s.name)||bytes(s.name)>200)throw new Error('Civic manifest source names must be unique and at most 200 UTF-8 bytes.');names.add(s.name);total+=atob(s.base64).length;}if(total>8*1024*1024)throw new Error('Civic manifests exceed 8 MiB.');return value;
}
export function civicInventoryRequest(value){validateCivicManifestSources(value);if(!value.sources.length)throw new Error('Select civic manifests first.');return {operation:'inspect-civic-manifests',inputs:value.sources.map(s=>({source_name:s.name,manifest_json:exactSourceText(s)}))};}
export async function readCivicManifests(selected,folder=false){
 const nativeRoot=folder==='native-root';
 const files=Array.from(selected).filter(f=>{
  if(!folder)return true;
  if(f.name!=='manifest.json')return false;
  if(!nativeRoot)return true;
  // A directory input names files relative to the user's selected root.
  // Native List considers only <root>/<label>/manifest.json.
  const parts=f.webkitRelativePath?.split('/')??[];
  return parts.length===3&&parts[2]==='manifest.json'&&parts.slice(0,2).every(p=>p&&p!=='.'&&p!=='..'&&!p.includes('\\'));
 });if(files.length>20)throw new Error('Select at most 20 civic manifests.');let total=0;for(const f of files){if(!Number.isSafeInteger(f.size)||f.size<1||f.size>1024*1024)throw new Error('Select nonempty manifests of at most 1 MiB each.');total+=f.size;}if(total>8*1024*1024)throw new Error('Civic manifests exceed 8 MiB.');const sources=[];
 for(const [i,f]of files.entries()){const s=await readElectionSource(f);s.name=`${i+1}: ${f.webkitRelativePath||f.name}`;if(atob(s.base64).length!==f.size)throw new Error('Civic manifest size changed while reading.');sources.push(s);}validateCivicManifestSources({sources});return sources;
}
export function civicInventoryZip(result){
 if(result?.schema_version!=='bisect-civic-inventory-v1'||typeof result.native_list_text!=='string')throw new Error('Run civic manifest inspection first.');const entries=[['list.txt',result.native_list_text],['inventory.json',JSON.stringify(result,null,2)]];for(const [i,e]of(result.entries||[]).entries())if(typeof e.native_show_json==='string')entries.push([`manifest-${i+1}.json`,e.native_show_json]);return storedZipBytes(entries.map(([n,t])=>[n,new TextEncoder().encode(t)]));
}

const CIVIC_CONFLICT_BYTES=3*1024*1024;
const CIVIC_CONFLICT_COUNT=20;
const conflictLabel=(value,max)=>typeof value==='string'&&value.length>0&&bytes(value)<=max&&!/[\u0000-\u001f\u007f-\u009f]/u.test(value);

export function validateNativeCivicConflictInputs(value){
 if(!value||Array.isArray(value)||Object.keys(value).join()!=='sources'||!Array.isArray(value.sources)||value.sources.length>CIVIC_CONFLICT_COUNT)throw new Error('Invalid normalized civic inputs.');
 let total=0;const labels=new Map();
 for(const item of value.sources){
  if(!item||Array.isArray(item)||Object.keys(item).sort().join()!=='label,source'||!conflictLabel(item.label,100)||!conflictLabel(item.source?.name,200))throw new Error('Use a nonempty desktop label and filename without control characters.');
  validateSourceFile(item.source,{maxBytes:CIVIC_CONFLICT_BYTES});
  const {base64}=item.source;const length=base64.length/4*3-(base64.endsWith('==')?2:base64.endsWith('=')?1:0);
  total+=length;if(length===0||total>CIVIC_CONFLICT_BYTES)throw new Error('Normalized civic inputs total at most 3 MiB.');
  if(labels.has(item.label)&&labels.get(item.label)!==base64)throw new Error('Repeated desktop labels must use identical normalized CSV bytes.');
  labels.set(item.label,base64);
 }
 return value;
}

export async function appendNativeCivicConflictFile(inputs,label,file){
 const {sources}=validateNativeCivicConflictInputs(inputs);
 if(sources.length===CIVIC_CONFLICT_COUNT)throw new Error('Select at most 20 normalized civic inputs.');
 if(!conflictLabel(label,100)||!conflictLabel(file?.name,200))throw new Error('Enter the desktop package label before selecting a normalized CSV.');
 const current=sources.reduce((n,{source:{base64:b}})=>n+b.length/4*3-(b.endsWith('==')?2:b.endsWith('=')?1:0),0);
 if(!Number.isSafeInteger(file.size)||file.size<1||file.size+current>CIVIC_CONFLICT_BYTES)throw new Error('Normalized civic inputs total at most 3 MiB.');
 const source=await readElectionSource(file);
 if(atob(source.base64).length!==file.size)throw new Error('Normalized CSV size changed while reading.');
 return validateNativeCivicConflictInputs({sources:[...sources,{label,source}]});
}

export function nativeCivicConflictRequest(inputs){
 const {sources}=validateNativeCivicConflictInputs(inputs);
 if(sources.length<2)throw new Error('Select at least two normalized civic CSV inputs.');
 return {operation:'inspect-native-civic-conflicts',inputs:sources.map(({label,source})=>({label,source_name:source.name,csv_base64:source.base64}))};
}
