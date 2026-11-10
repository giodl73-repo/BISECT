import {validateSourceFile,encodeBytes} from './package-files.js';
import {storedZipBytes} from './reproducibility-files.js';
import {NATIVE_AGGREGATE_TYPES,NATIVE_AGGREGATE_ALL} from './native-aggregate-catalog.js';
export {NATIVE_AGGREGATE_TYPES};
export const resolvedNativeAggregateTypes=types=>types.includes('all')?[...NATIVE_AGGREGATE_ALL]:[...types];
export const NATIVE_AGGREGATE_DEFAULTS={settings:{year:'2020',types:['all'],format:'csv',csv:false},reports:{}};
export function validateAggregateCompletion(c,year){
 if(!c||Object.keys(c).sort().join()!=='existing_outputs,force,year'||c.year!==year||typeof c.force!=='boolean'||!Array.isArray(c.existing_outputs)||c.existing_outputs.length>10||new Set(c.existing_outputs).size!==c.existing_outputs.length||c.existing_outputs.some(p=>!NATIVE_AGGREGATE_TYPES.some(t=>p===`us_${t}.json`)))throw new Error('Output observation must match the selected year and contain distinct native JSON output names. Clear or reselect it after changing year.');
 return c;
}
export function readAggregateCompletion(files,year,force=false){
 const list=[...files];if(!list.length||list.length>10000)throw new Error('Select one national output folder with up to 10,000 entries.');
 const roots=new Set(),names=new Set();
 for(const file of list){
  const parts=(file.webkitRelativePath||'').split('/');
  if(parts.length<2||parts.some(p=>!p||['.','..','__proto__','constructor','prototype'].includes(p)||/[\\\u0000-\u001f]/.test(p)))throw new Error('Unsafe output observation path.');
  roots.add(parts[0]);if(roots.size>1)throw new Error('Select one national output root.');
  if(NATIVE_AGGREGATE_TYPES.some(t=>parts[1]===`us_${t}.json`))names.add(parts[1]);
 }
 return validateAggregateCompletion({year,existing_outputs:[...names].sort(),force},year);
}
export function validateNativeAggregateInputs(v){
 if(!v||!['reports','settings'].every(k=>Object.hasOwn(v,k))||Object.keys(v).some(k=>!['reports','settings','completion','deferred_types'].includes(k))||!v.settings||Object.keys(v.settings).sort().join()!=='csv,format,types,year')throw new Error('Invalid native aggregate inputs.');const s=v.settings;
 if(Object.hasOwn(v,'completion'))validateAggregateCompletion(v.completion,s.year);
 if(Object.hasOwn(v,'deferred_types')&&(!Array.isArray(v.deferred_types)||v.deferred_types.length>10||new Set(v.deferred_types).size!==v.deferred_types.length||v.deferred_types.some(t=>!NATIVE_AGGREGATE_TYPES.includes(t))))throw new Error('Invalid deferred aggregate source types.');
 if(!['2000','2010','2020'].includes(s.year)||!['csv','json','none','parquet'].includes(s.format)||typeof s.csv!=='boolean'||!Array.isArray(s.types)||!s.types.length||s.types.length>11||s.types.some(t=>!['all',...NATIVE_AGGREGATE_TYPES].includes(t)))throw new Error('Choose native year, analyzer types and output formats.');
 if(!v.reports||Array.isArray(v.reports)||typeof v.reports!=='object'||Object.keys(v.reports).length>60)throw new Error('Choose at most 60 state folders.');let bytes=0;const states=new Set();
 for(const [state,reports]of Object.entries(v.reports)){if(!/^[A-Za-z0-9_ -]{1,60}$/.test(state)||states.has(state.toUpperCase())||!reports||Array.isArray(reports)||typeof reports!=='object'||Object.keys(reports).length>10)throw new Error('Invalid or duplicate state folder scope.');states.add(state.toUpperCase());for(const [type,source]of Object.entries(reports)){if(!NATIVE_AGGREGATE_TYPES.includes(type))throw new Error('Unknown analyzer source.');validateSourceFile(source,{maxBytes:3*1024*1024,allowEmpty:true});const raw=Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0));new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(raw);bytes+=state.length+type.length+raw.length;}}
 if(bytes>8*1024*1024)throw new Error('Aggregate input exceeds 8 MiB.');return v;
}
export function nativeAggregateRequest(v){validateNativeAggregateInputs(v);const needed=(v.deferred_types||[]).filter(t=>resolvedNativeAggregateTypes(v.settings.types).includes(t)&&(!v.completion||v.completion.force||!v.completion.existing_outputs.includes(`us_${t}.json`)));if(needed.length)throw new Error('Reload or reselect state sources before regenerating deferred analyzers: '+needed.join(', '));if(!Object.keys(v.reports).length)throw new Error('Select raw state analysis folders.');return {operation:'aggregate-native-analysis',input:{...v.settings,...(v.completion?{completion:v.completion}:{}),reports:Object.fromEntries(Object.entries(v.reports).map(([state,reports])=>[state,Object.fromEntries(Object.entries(reports).filter(([type])=>resolvedNativeAggregateTypes(v.settings.types).includes(type)).map(([type,s])=>[type,new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(s.base64),c=>c.charCodeAt(0)))]))]))}};}
export async function readNativeAggregateFolder(files,settings,completion=null){
 const list=[...files];
 if(!list.length||list.length>10000)throw new Error('Select a states collection with up to 10,000 entries.');
 const active=new Set(resolvedNativeAggregateTypes(settings.types)),observed=new Set(),foldedStates=new Map();
 if(completion){validateAggregateCompletion(completion,settings.year);if(!completion.force)for(const type of active)if(completion.existing_outputs.includes(`us_${type}.json`))active.delete(type);}
 const pending=[],seen=new Set(),roots=new Set(),deferred=new Set();let total=0;
 for(const file of list){
  const p=(file.webkitRelativePath||'').split('/');
  if(p.length<2||p.some(s=>!s||['.','..','__proto__','constructor','prototype'].includes(s)||/[\\\u0000-\u001f]/.test(s)))throw new Error('Unsafe collection folder path.');
  roots.add(p[0]);if(roots.size>1)throw new Error('Select one collection root.');
  if(p.length<3)continue;
  const state=p[1];
  if(!/^[A-Za-z0-9_ -]{1,60}$/.test(state))throw new Error('Invalid state folder label.');
  const folded=state.toUpperCase();
  if(foldedStates.has(folded)&&foldedStates.get(folded)!==state)throw new Error('Duplicate state folder scope.');
  foldedStates.set(folded,state);observed.add(state);
  if(observed.size>60)throw new Error('Too many state folders.');
  // Native discovery opens only states/<state>/analysis/<type>.json.
  // Other files reveal a state directory without being read or size-gated.
  if(p.length!==4||p[2]!=='analysis')continue;
  const name=p[3],type=name.replace(/\.json$/,'');
  if(!name.endsWith('.json')||!NATIVE_AGGREGATE_TYPES.includes(type))continue;
  if(!active.has(type)){if(resolvedNativeAggregateTypes(settings.types).includes(type))deferred.add(type);continue;}
  const key=state+'/'+type;if(seen.has(key))throw new Error('Duplicate state analysis file.');seen.add(key);
  if(!Number.isSafeInteger(file.size)||file.size<0||file.size>3*1024*1024)throw new Error('Analysis file exceeds 3 MiB.');
  total+=file.size+state.length+type.length;if(total>8*1024*1024)throw new Error('Collection exceeds 8 MiB.');
  pending.push({state,type,file});
 }
 const reports=Object.fromEntries([...observed].sort().map(state=>[state,Object.create(null)]));
 for(const {state,type,file}of pending){
  const bytes=new Uint8Array(await file.arrayBuffer());if(bytes.length!==file.size)throw new Error('Analysis file changed while reading.');
  new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(bytes);reports[state][type]={name:file.name,base64:encodeBytes(bytes)};
 }
 return validateNativeAggregateInputs({settings,reports,...(completion?{completion}: {}),...(deferred.size?{deferred_types:[...deferred].sort()}: {})});
}
export async function nativeAggregateZip(result){if(result?.schema_version!=='bisect-native-aggregate-v1'||!result.files||!Object.keys(result.files).length)throw new Error('Run aggregation with available output first.');const entries=[];for(const [name,text]of Object.entries(result.files)){if(!/^us_[a-z-]+(?:_(?:states|districts))?\.(?:json|csv)$/.test(name)||typeof text!=='string')throw new Error('Invalid native aggregate output.');const bytes=new TextEncoder().encode(text),hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==result.file_sha256?.[name])throw new Error('Aggregate output hash mismatch.');entries.push([name,bytes]);}return storedZipBytes(entries);}
