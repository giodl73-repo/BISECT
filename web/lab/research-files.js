import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
const LIMIT=3*1024*1024;
const record=v=>v&&typeof v==='object'&&!Array.isArray(v);
const nameBytes=n=>new TextEncoder().encode(n);
const chainName=n=>typeof n==='string'&&n!=='.jsonl'&&n.endsWith('.jsonl')&&nameBytes(n).length<=120&&!/[\/\\:\u0000-\u001f\u007f-\u009f]/.test(n)&&new TextDecoder('utf-8',{fatal:true}).decode(nameBytes(n))===n;
export function validateResearchInputs(value){
 if(!record(value)||![4,5].includes(Object.keys(value).length)||!['chains','enacted','rhat_threshold','ess_min'].every(k=>Object.hasOwn(value,k))||Object.keys(value).some(k=>!['chains','enacted','rhat_threshold','ess_min','strict'].includes(k))||(Object.hasOwn(value,'strict')&&typeof value.strict!=='boolean'))throw new Error('Invalid research inputs.');
 if(!Number.isFinite(value.rhat_threshold)||value.rhat_threshold<=0||!Number.isFinite(value.ess_min)||value.ess_min<0)throw new Error('Supply a positive R-hat threshold and nonnegative ESS minimum.');
 if(!record(value.chains)||Object.keys(value.chains).length>32)throw new Error('Select at most 32 parallel chains.');
 let bytes=0;for(const [name,source] of Object.entries(value.chains)){if(!chainName(name))throw new Error('Chains need unique plain JSONL names.');validateSourceFile(source);bytes+=nameBytes(name).length+new TextEncoder().encode(exactSourceText(source)).length;}
 if(value.enacted!==null){validateSourceFile(value.enacted);bytes+=new TextEncoder().encode(exactSourceText(value.enacted)).length;}
 if(bytes>LIMIT)throw new Error('Research sources must total at most 3 MiB.');return value;
}
export function researchRequest(inputs){
 validateResearchInputs(inputs);if(Object.keys(inputs.chains).length<4)throw new Error('Supply at least four independent parallel chain files.');
 return {operation:'validate-research-ensemble',chains_jsonl:Object.fromEntries(Object.entries(inputs.chains).map(([n,s])=>[n,exactSourceText(s)])),enacted_json:inputs.enacted?exactSourceText(inputs.enacted):null,rhat_threshold:inputs.rhat_threshold,ess_min:inputs.ess_min,strict:inputs.strict??false};
}
export async function readResearchChains(selected){
 const list=Array.from(selected);if(list.length>32)throw new Error('Select at most 32 parallel chains.');let bytes=0;const names=new Set();
 for(const f of list){if(!chainName(f.name)||names.has(f.name))throw new Error('Chains need unique plain JSONL names.');names.add(f.name);if(!Number.isSafeInteger(f.size)||f.size<1)throw new Error('Select nonempty chains with valid file sizes.');bytes+=f.size+nameBytes(f.name).length;}
 if(bytes>LIMIT)throw new Error('Research sources exceed 3 MiB.');const sources=Object.create(null);
 for(const f of list){const source=await readElectionSource(f);if(atob(source.base64).length!==f.size)throw new Error('Chain size changed while reading.');exactSourceText(source);sources[f.name]=source;}return sources;
}
