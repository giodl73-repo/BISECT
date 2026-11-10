import {validateSourceFile} from './package-files.js';

export const freshNativeAuxiliaryKinds=['coi','partisan','demographics','economic','housing'];
export function validateFreshNativeMissingSource(source){
 if(!source||Array.isArray(source)||Object.keys(source).length!==2||!Object.hasOwn(source,'name')||!Object.hasOwn(source,'missing')||source.missing!==true||typeof source.name!=='string'||!source.name.trim()||source.name.includes('\0')||new TextEncoder().encode(source.name).length>4096)throw Error('A missing-source claim requires an explicit recorded name and missing: true, without file bytes.');
 return source;
}
export function freshNativeAuxiliarySources(input){
 if(typeof input!=='string'||new TextEncoder().encode(input).length>25*1024*1024)throw Error('Native auxiliary inputs must fit 25 MiB.');
 const value=JSON.parse(input),aux=value?.native_auxiliary;
 if(!value||Array.isArray(value)||Object.keys(value).join()!=='native_auxiliary'||!aux||Array.isArray(aux)||typeof aux!=='object'||!Object.keys(aux).length||Object.keys(aux).some(key=>!freshNativeAuxiliaryKinds.includes(key)))throw Error('Select original native auxiliary files only; derived observations and imported execution records are not input authority.');
 const selected={};
 for(const [kind,source] of Object.entries(aux)){
  if(!source||Array.isArray(source)||Object.keys(source).length!==2||!Object.hasOwn(source,'source_name')||!Object.hasOwn(source,'bytes_base64')||!(typeof source.bytes_base64==='string'||source.bytes_base64===null))throw Error('Native auxiliary input requires selected file bytes or an explicit null-byte absence claim.');
  if(source.bytes_base64===null){selected[kind]=validateFreshNativeMissingSource({name:source.source_name,missing:true});continue;}
  const file={name:source.source_name,base64:source.bytes_base64};
  validateSourceFile(file,{maxBytes:(kind==='coi'?16:8)*1024*1024});
  if(!file.name.trim()||file.name.includes('\0'))throw Error('Select a named native auxiliary file.');
  const alphabet='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/',text=file.base64;
  if(text.endsWith('==')&&(alphabet.indexOf(text.at(-3))&15)!==0||!text.endsWith('==')&&text.endsWith('=')&&(alphabet.indexOf(text.at(-2))&3)!==0)throw Error('Native auxiliary files require canonical base64 pad bits.');
  selected[kind]=file;
 }
 return selected;
}

export function freshNativeAuxiliaryJson(sources){
 const auxiliary={};
 for(const kind of freshNativeAuxiliaryKinds)if(sources[kind]){const source=sources[kind];if(Object.hasOwn(source,'missing'))validateFreshNativeMissingSource(source);auxiliary[kind]={source_name:source.name,bytes_base64:source.missing===true?null:source.base64};}
 if(!Object.keys(auxiliary).length)return null;
 const json=JSON.stringify({native_auxiliary:auxiliary});
 freshNativeAuxiliarySources(json);
 return json;
}
