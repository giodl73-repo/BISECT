import {validateSourceFile,exactSourceText,encodeBytes} from './package-files.js';
export function validateLabelInventoryInputs(v){
 if(!v||Object.keys(v).sort().join()!=='label,registry'||typeof v.label!=='string'||new TextEncoder().encode(v.label).length>200||/[\u0000-\u001f\u007f-\u009f]/.test(v.label))throw new Error('Invalid registry inventory query.');
 if(v.registry!==null){if(v.registry?.base64===''){if(Object.keys(v.registry).sort().join()!=='base64,name'||typeof v.registry.name!=='string'||v.registry.name.length>200)throw new Error('Invalid empty registry source.');}else{validateSourceFile(v.registry,{maxBytes:1024*1024});exactSourceText(v.registry);}}return v;
}
export function nativeRegistryText(source){validateLabelInventoryInputs({label:'',registry:source});return new TextDecoder('utf-8',{fatal:true,ignoreBOM:true}).decode(Uint8Array.from(atob(source.base64),c=>c.charCodeAt(0)));}
export function labelInventoryRequest(v){validateLabelInventoryInputs(v);return {operation:'inspect-label-registry',registry_json:v.registry?nativeRegistryText(v.registry):null,label:v.label||null};}
export async function readLabelInventoryRegistry(f){if(!f||!Number.isSafeInteger(f.size)||f.size<0||f.size>1024*1024)throw new Error('Select native registry up to 1 MiB.');const bytes=new Uint8Array(await f.arrayBuffer());if(bytes.length!==f.size)throw new Error('Registry file size changed while reading.');const registry={name:f.name,base64:encodeBytes(bytes)};validateLabelInventoryInputs({label:'',registry});return registry;}

export const LABEL_COMPARE_DEFAULTS={registry:null,label_a:'',label_b:'',year:'2020'};
export function validateLabelCompareInputs(v){
 if(!v||Object.keys(v).sort().join()!=='label_a,label_b,registry,year'||typeof v.year!=='string'||!/^\d{4}$/.test(v.year))throw new Error('Select a four-digit comparison year.');
 for(const label of [v.label_a,v.label_b])if(typeof label!=='string'||new TextEncoder().encode(label).length>200||label&&!/^[a-zA-Z0-9_-]+$/.test(label))throw new Error('Comparison labels must use letters, digits, underscores or hyphens, up to 200 bytes.');
 validateLabelInventoryInputs({registry:v.registry,label:''});return v;
}
export function labelCompareRequest(v){validateLabelCompareInputs(v);if(!v.label_a||!v.label_b)throw new Error('Select both comparison labels.');return {operation:'inspect-label-comparison',registry_json:v.registry?nativeRegistryText(v.registry):null,label_a:v.label_a,label_b:v.label_b,year:v.year};}
