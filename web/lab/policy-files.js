import {validateSourceFile,exactNativeUtf8Text,readElectionSource} from './package-files.js';
import {validatePolicyFallback} from './policy-fallback.js';
export const MAX_POLICY_BYTES=512*1024;
export function validatePolicyInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).some(k=>!['query','source','fallback'].includes(k))||!Object.hasOwn(value,'query')||!Object.hasOwn(value,'source')||typeof value.query!=='string'||new TextEncoder().encode(value.query).length>200)throw new Error('Supply a location query of at most 200 UTF-8 bytes.');
 if(value.fallback!=null){validatePolicyFallback(value.fallback);if(value.source!==null)throw Error('Choose explicit inspection or native fallback sources, not both.');}
 if(value.source!==null){validateSourceFile(value.source,{maxBytes:MAX_POLICY_BYTES});exactNativeUtf8Text(value.source);}
 return value;
}
export function policyRequest(value){
 validatePolicyInputs(value);if(!value.query.trim())throw new Error('Supply a location code or name.');
 if(value.fallback!=null)return {operation:'inspect-location-policy',query:value.query,source_name:null,policy_json:null,fallback:structuredClone(value.fallback)};
 return {operation:'inspect-location-policy',query:value.query,source_name:value.source?.name??null,policy_json:value.source?exactNativeUtf8Text(value.source):null};
}
export async function readPolicySource(file){
 if(!file||!Number.isSafeInteger(file.size)||file.size<1||file.size>MAX_POLICY_BYTES)throw new Error('Select nonempty policy JSON of at most 512 KiB.');
 const source=await readElectionSource(file);if(atob(source.base64).length!==file.size)throw new Error('Policy source size changed while reading.');
 validatePolicyInputs({query:'',source});return source;
}
