import {validateSourceFile,exactSourceText,readElectionSource} from './package-files.js';
const LIMIT=3*1024*1024;
const nameValid=n=>typeof n==='string'&&n.length>0&&n.length<=120&&/^[A-Za-z0-9_-][A-Za-z0-9_.-]*$/.test(n);
export function validateStabilityInputs(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||Object.keys(value).length!==1||!Object.hasOwn(value,'plans')||!value.plans||typeof value.plans!=='object'||Array.isArray(value.plans)||Object.keys(value.plans).length>32)throw new Error('Supply at most 32 uniquely named plans.');
 let bytes=0;for(const [name,source]of Object.entries(value.plans)){if(!nameValid(name))throw new Error('Plans need unique plain basenames of at most 120 bytes.');validateSourceFile(source);if(source.base64.length>4*1024*1024)throw new Error('Stability sources exceed 3 MiB.');exactSourceText(source);bytes+=name.length+atob(source.base64).length;}
 if(bytes>LIMIT)throw new Error('Stability sources must total at most 3 MiB.');return value;
}
export function stabilityRequest(inputs){validateStabilityInputs(inputs);if(Object.keys(inputs.plans).length<2)throw new Error('Select at least two RPLAN files.');return {operation:'analyze-suite-stability',plans_json:Object.fromEntries(Object.entries(inputs.plans).map(([n,s])=>[n,exactSourceText(s)]))};}
export async function readStabilityPlans(selected){
 const list=Array.from(selected);if(list.length>32)throw new Error('Select at most 32 RPLAN files.');const names=new Set();let bytes=0;
 for(const f of list){if(!nameValid(f.name)||names.has(f.name))throw new Error('Plans need unique plain basenames.');names.add(f.name);if(!Number.isSafeInteger(f.size)||f.size<1)throw new Error('Select nonempty plans with valid sizes.');bytes+=f.size+f.name.length;}
 if(bytes>LIMIT)throw new Error('Stability sources exceed 3 MiB.');const plans=Object.create(null);
 for(const f of list){const source=await readElectionSource(f);if(atob(source.base64).length!==f.size)throw new Error('Plan size changed while reading.');exactSourceText(source);plans[f.name]=source;}return plans;
}
