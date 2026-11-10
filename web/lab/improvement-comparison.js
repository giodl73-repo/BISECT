import {validateSourceFile,encodeBytes} from './package-files.js';
import {validateTextComparisonInputs} from './text-comparison-files.js';
const source=(name,text)=>({name,base64:encodeBytes(new TextEncoder().encode(text))});
export async function improvementComparisonInputs(result,parent){
 if(result?.schema_version!=='bisect-local-improvement-output-v1')throw new Error('Complete a fresh improvement first.');validateSourceFile(parent,{maxBytes:3*1024*1024});
 const outputs={};for(const name of ['improved.rplan','improved.rctx']){const text=result.files?.[name];if(typeof text!=='string')throw new Error('Missing improved plan/context.');const file=source(name,text);validateSourceFile(file,{maxBytes:(name.endsWith('.rctx')?8:3)*1024*1024});const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(text))),b=>b.toString(16).padStart(2,'0')).join('');if(hash!==result.file_sha256?.[name])throw new Error('Improvement source hash mismatch.');outputs[name]=file;}
 return validateTextComparisonInputs({a:{...parent},b:outputs['improved.rplan'],context:outputs['improved.rctx']});
}
