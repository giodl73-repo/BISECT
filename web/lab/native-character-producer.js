import {validateNativeCharacterInputs} from './native-character-files.js';
import {validateJsonTree} from './project.js';
export function nativeCharacterProducerSource(plan){
 const p=plan?.provenance?.producer,r=p?.native_character_source??p?.multiscale_input?.request?.native_character_source;if(r==null)return null;
 validateJsonTree(r);const keys=['state','year','data_year','kind','source_label','source_name','source_base64'];if(Object.keys(r).length!==keys.length||keys.some(k=>!Object.hasOwn(r,k)))throw new Error('Invalid retained native source input.');
 const input=validateNativeCharacterInputs({source:{name:r.source_name,base64:r.source_base64},settings:Object.fromEntries(keys.slice(0,5).map(k=>[k,r[k]]))}),o=p.multiscale_input?.request?.options??p.engine_options;
 if(plan.metadata?.jurisdiction!==r.state||String(plan.plan?.units?.year)!==r.year||o?.weights!==r.kind+'-character')throw new Error('Retained native source scope disagrees with plan producer.');return input;
}
export async function verifyNativeCharacterProducerSource(plan,context){
 const input=nativeCharacterProducerSource(plan);if(!input)throw new Error('This plan has no retained native CSV source.');const bytes=Uint8Array.from(atob(input.source.base64),c=>c.charCodeAt(0)),sha=[...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(b=>b.toString(16).padStart(2,'0')).join(''),claimed=plan.provenance.source_hashes?.['bisect.native-character-source'];
 if(sha!==claimed||context&&context.source_hashes?.['bisect.native-character-source']!==sha)throw new Error('Retained original CSV bytes disagree with exported source hashes.');return {input,bytes,sha256:sha};
}
