import {freshNativeAuxiliaryKinds,validateFreshNativeMissingSource} from './fresh-native-state-auxiliary.js';
import {validateSourceFile,encodeBytes} from './package-files.js';

export const freshNativeMultiscaleSamplers=Object.freeze({
 native64:'rand-0.8.6-smallrng-native64-wilson',
 native32:'rand-0.8.6-smallrng-native32-wilson'
});
export const isFreshNativeMultiscaleSampler=value=>Object.values(freshNativeMultiscaleSamplers).includes(value);

const record=value=>value!==null&&typeof value==='object'&&!Array.isArray(value);
const fields=['arguments_json','state','year','label','chamber','created_at'];
const sourceKinds=['adjacency','geoids','centroids','coi','partisan','demographics','economic','housing','fine_adjacency','fine_geoids','fine_centroids'];
const bytes=text=>new TextEncoder().encode(text).length;
function known(value,keys,label){if(!record(value)||Object.keys(value).some(key=>!keys.includes(key)))throw Error('Invalid fresh State '+label+'.');}
function source(value,maxBytes){
 validateSourceFile(value,{maxBytes});
 const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',text=value.base64;
 if(text.endsWith('==')&&(alphabet.indexOf(text.at(-3))&15)!==0||!text.endsWith('==')&&text.endsWith('=')&&(alphabet.indexOf(text.at(-2))&3)!==0)throw Error('Fresh State sources require canonical base64 pad bits.');
}

// This is persistence validation only. Source claims, draft flags, and retained
// records cannot establish engine admissibility, execution, or authenticity.
export function validateFreshNativeStateSession(value){
 known(value,['schema_version','fields','sources','display_geometry_json','retained_run','runtime_artifact','multiscale_sampler','load_c_metis','native_recursive_backend'],'session');
 if(value.schema_version!=='bisect-fresh-native-state-session-v1')throw Error('Unsupported fresh State session format.');
 known(value.fields,fields,'draft fields');
 if(Object.keys(value.fields).length!==fields.length||fields.some(key=>typeof value.fields[key]!=='string'||bytes(value.fields[key])>(key==='arguments_json'?256*1024:800)))throw Error('Fresh State draft fields exceed their text limits.');
 known(value.sources,sourceKinds,'sources');
 for(const [key,file]of Object.entries(value.sources)){if(file&&Object.hasOwn(file,'missing')){if(!freshNativeAuxiliaryKinds.includes(key))throw Error('Only native auxiliary sources accept missing-source claims.');validateFreshNativeMissingSource(file);}else source(file,(key.endsWith('adjacency')?64:['partisan','demographics','economic','housing'].includes(key)?8:16)*1024*1024);}
 if(value.multiscale_sampler!==undefined&&!isFreshNativeMultiscaleSampler(value.multiscale_sampler))throw Error('Select a supported native multiscale sampler.');
 if(value.native_recursive_backend!==undefined&&!['metis-core','c-ffi'].includes(value.native_recursive_backend))throw Error('Invalid fresh State native execution profile.');
 if(value.load_c_metis!==undefined&&typeof value.load_c_metis!=='boolean')throw Error('Invalid fresh State C runtime selection.');
 if(value.display_geometry_json!==null&&(typeof value.display_geometry_json!=='string'||bytes(value.display_geometry_json)>64*1024*1024))throw Error('Fresh State display geometry exceeds 64 MiB.');
 if(value.retained_run!==null)source(value.retained_run,64*1024*1024);
 if(value.runtime_artifact!==null){
  known(value.runtime_artifact,['sha256','byte_length'],'runtime artifact');
  if(!/^[a-f0-9]{64}$/.test(value.runtime_artifact.sha256)||!Number.isSafeInteger(value.runtime_artifact.byte_length)||value.runtime_artifact.byte_length<1||value.runtime_artifact.byte_length>128*1024*1024)throw Error('Invalid retained fresh State engine identity.');
 }
 return value;
}

export function retainFreshNativeStateRun(result){
 const bytes=new TextEncoder().encode(JSON.stringify(result));
 if(bytes.length>64*1024*1024)throw Error('Fresh State run exceeds the 64 MiB retained-record limit. Download its individual output files.');
 return {name:'fresh-native-state-retained-run.json',base64:encodeBytes(bytes)};
}

export function freshNativeStateRetainedText(file){
 source(file,64*1024*1024);
 return new TextDecoder('utf-8',{fatal:true}).decode(Uint8Array.from(atob(file.base64),char=>char.charCodeAt(0)));
}
